use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crayon::bincode;
use crayon::res::vfs::manifest::{self, Manifest, ManifestItem};
use failure::ResultExt;
use uuid::Uuid;
use walkdir::WalkDir;

use super::cache::AssetCache;
use super::metadata::{AssetMetadata, ResourceMetadata, EXTENSION};
use super::settings::AssetSettings;

use assets::*;
use platform::RuntimePlatform;

pub type Result<T> = ::std::result::Result<T, ::failure::Error>;

pub struct AssetDatabase {
    assets_dir: PathBuf,
    resources_dir: PathBuf,
    platform: RuntimePlatform,
    cache: AssetCache,
    assets: BTreeMap<PathBuf, AssetMetadata>,
    exts: HashMap<String, AssetType>,
    importers: HashMap<AssetType, Box<AssetImporter>>,
}

impl AssetDatabase {
    pub fn new<T: AsRef<Path>>(
        root: T,
        params: AssetSettings,
        platform: RuntimePlatform,
        strict: bool,
    ) -> Result<AssetDatabase> {
        let root = root.as_ref();

        // Makes sure that assets and resources folder exist.
        let assets_dir = root.join(&params.source);
        let resources_dir = root.join(&params.destination);

        if resources_dir.starts_with(&assets_dir) || assets_dir.starts_with(&resources_dir) {
            bail!("The assets folder can not be located under resources folder.");
        }

        if !assets_dir.exists() {
            bail!("The assets folder ({:?}) is not exists.", assets_dir);
        }

        if !resources_dir.exists() {
            fs::create_dir_all(&resources_dir)
                .context("Failed to create resource folder at destination")?;
        }

        let cache = AssetCache::new(&assets_dir, strict)?;
        let mut database = AssetDatabase {
            assets_dir: assets_dir,
            resources_dir: resources_dir,
            platform: platform,
            cache: cache,
            assets: BTreeMap::new(),
            exts: HashMap::new(),
            importers: HashMap::new(),
        };

        let ti = TextureImporter {};
        database.importers.insert(AssetType::Texture, Box::new(ti));

        let ti = TransmissionImporter {};
        database
            .importers
            .insert(AssetType::Transmission, Box::new(ti));

        let bi = BytesImporter {};
        database.importers.insert(AssetType::Bytes, Box::new(bi));

        let ai = AudioImporter {};
        database.importers.insert(AssetType::Audio, Box::new(ai));

        for v in params.importers {
            for e in v.extensions {
                let ext = e.trim_left_matches('.');
                if database.exts.contains_key(ext) {
                    bail!("File extension {:?} has more than one importer.", e);
                }

                database.exts.insert(ext.to_owned(), v.tp);
            }
        }

        database.scan()?;
        Ok(database)
    }

    pub fn import_all(&mut self) -> Result<()> {
        fs::remove_dir_all(&self.resources_dir)?;
        fs::create_dir_all(&self.resources_dir)?;

        let mut manifest = Manifest::new();
        for (k, v) in &self.assets {
            if let Some(i) = Self::importer(&self.exts, &self.importers, &k) {
                let mut db = AssetIntermediateGenerator {
                    assets_dir: &self.assets_dir,
                    cache: &mut self.cache,
                    metadata: v,
                    platform: self.platform,
                };

                i.import(&mut db)?;
            }

            for r in &v.resources {
                let cmd = self.cache.resource(k, &r.name, false);
                let src = cmd.path(self.cache.dir());
                let dst = self.resources_dir.join(format!("{:X}", r.uuid.simple()));

                if src.exists() {
                    let location = if k == &r.name {
                        k.clone()
                    } else {
                        k.join(&r.name)
                    };

                    fs::copy(&src, &dst)?;
                    manifest.items.push(ManifestItem {
                        filename: manifest.buf.extend_from_str(location.to_str().unwrap()),
                        dependencies: manifest.buf.extend_from_slice(&[]),
                        uuid: r.uuid,
                    });
                } else {
                    println!("NOT EXITS {}", r.name.display());
                }
            }
        }

        // Generates a manifest to locate resources at runtime.
        let mut o = fs::File::create(&self.resources_dir.join(manifest::NAME))?;
        o.write_all(&manifest::MAGIC)?;
        bincode::serialize_into(&mut o, &manifest)?;

        self.cache.save()?;
        Ok(())
    }

    fn scan(&mut self) -> Result<()> {
        let mut metafiles: HashSet<PathBuf> = HashSet::new();
        let mut files = HashSet::new();

        for e in WalkDir::new(&self.assets_dir).into_iter().filter_map(|e| {
            if let Ok(e) = e {
                if e.path()
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| !s.starts_with("."))
                    .unwrap_or(false)
                {
                    return Some(e);
                }
            }

            None
        }) {
            if e.file_type().is_dir() {
                continue;
            }

            let relative = e.path().strip_prefix(&self.assets_dir).unwrap().to_owned();

            {
                let relative_str = relative.to_str().unwrap();
                if relative_str.ends_with(EXTENSION) {
                    let file = relative_str.trim_right_matches(EXTENSION);
                    metafiles.insert(file.into());
                    continue;
                }
            }

            files.insert(relative);
        }

        // Imports meta-files into database.
        self.assets.clear();
        for v in files {
            metafiles.remove(&v);

            let mut metadata = AssetMetadata::import_from(&self.assets_dir, &v)?;

            // Generates essential intermediate files.
            if let Some(i) = Self::importer(&self.exts, &self.importers, &v) {
                let mut db = AssetIntermediateGenerator {
                    assets_dir: &self.assets_dir,
                    cache: &mut self.cache,
                    metadata: &metadata,
                    platform: self.platform,
                };

                i.compile(&mut db)?;
            }

            // Updates meta-file.
            let modified = {
                let mut db = AssetMetadataGenerator::new(&mut self.cache, &mut metadata);
                if let Some(i) = Self::importer(&self.exts, &self.importers, &v) {
                    i.compile_metadata(&mut db)?;
                }
                db.modified()
            };

            if modified {
                self.cache.metafile(&v, true);
                metadata.save(&self.assets_dir, &v)?;
            }

            self.assets.insert(v, metadata);
        }

        // Removes all the un-associated meta-files.
        for v in metafiles.drain() {
            let n = AssetMetadata::with_extension(&v);
            let metafile = self.assets_dir.join(&n);
            fs::remove_file(metafile)?;

            warn!("A meta data file ({:?}) exists but its asset can't be found. When moving \
                or deleting files, please ensure that the corresponding .meta.toml file is moved or \
                deleted along with it.", n);
        }

        self.cache.strip(&self.assets)?;
        Ok(())
    }

    fn importer<'a, T: AsRef<Path>>(
        exts: &'a HashMap<String, AssetType>,
        importers: &'a HashMap<AssetType, Box<AssetImporter>>,
        name: T,
    ) -> Option<&'a AssetImporter> {
        let tp = name
            .as_ref()
            .extension()
            .and_then(|e| e.to_str())
            .and_then(|e| exts.get(e))
            .unwrap_or(&AssetType::Bytes);
        importers.get(tp).map(|i| i.as_ref())
    }
}

pub struct AssetMetadataGenerator<'a> {
    cache: &'a mut AssetCache,
    metadata: &'a mut AssetMetadata,
    table: HashMap<PathBuf, ResourceType>,
    modified: bool,
}

impl<'a> AssetMetadataGenerator<'a> {
    pub fn new(cache: &'a mut AssetCache, metadata: &'a mut AssetMetadata) -> Self {
        let mut table = HashMap::new();
        let mut modified = false;

        for i in (0..metadata.resources.len()).rev() {
            if table.contains_key(&metadata.resources[i].name) {
                metadata.resources.remove(i);
                modified = true;
            } else {
                let v = &metadata.resources[i];
                table.insert(v.name.clone(), v.tp);
            }
        }

        AssetMetadataGenerator {
            cache: cache,
            metadata: metadata,
            table: table,
            modified: modified,
        }
    }

    /// Gets the short-name of this asset.
    pub fn name(&self) -> &Path {
        &self.metadata.name
    }

    /// Gets the parameters of resource entry.
    pub fn params(&self) -> AssetParams {
        self.metadata.params
    }

    /// Adds a new resource entry with parameters.
    pub fn update_params(&mut self, params: AssetParams) {
        self.modified = true;
        self.metadata.params = params;
    }

    pub fn add<T: AsRef<Path>>(&mut self, name: T, tp: ResourceType) {
        let name = name.as_ref();

        if let Some(&v) = self.table.get(name) {
            if v == tp {
                return;
            }
        }

        let rmd = ResourceMetadata::new(name.to_owned(), tp);
        self.metadata.resources.push(rmd);
        self.table.insert(name.to_owned(), tp);
        self.modified = true;
    }

    /// Gets the full path to specified intermediate file.
    pub fn intermediate<T: AsRef<Path>>(&mut self, name: T, modified: bool) -> PathBuf {
        let name = name.as_ref();
        let md = self.cache.intermediate(&self.metadata.name, name, modified);
        let mut path = self.cache.dir().join(format!("{:X}", md.uuid.simple()));
        if let Some(extension) = name.extension() {
            path = path.with_extension(extension);
        }

        path
    }

    fn modified(&self) -> bool {
        self.modified
    }
}

pub struct AssetIntermediateGenerator<'a> {
    assets_dir: &'a Path,
    cache: &'a mut AssetCache,
    metadata: &'a AssetMetadata,
    platform: RuntimePlatform,
}

impl<'a> AssetIntermediateGenerator<'a> {
    pub fn name(&self) -> &Path {
        &self.metadata.name
    }

    pub fn platform(&self) -> RuntimePlatform {
        self.platform
    }

    /// Gets the importing params for resource.
    pub fn params(&self) -> AssetParams {
        self.metadata.params
    }

    // Gets the universal-uniqued identifier for resource.
    pub fn uuid<T: AsRef<Path>>(&self, name: T) -> Option<Uuid> {
        let name = name.as_ref();
        for v in &self.metadata.resources {
            if v.name == name {
                return Some(v.uuid);
            }
        }

        None
    }

    /// Gets the full path to asset file.
    pub fn path(&self) -> PathBuf {
        self.assets_dir.join(&self.metadata.name)
    }

    /// Checks if the asset or its metadat has been modified.
    pub fn modified(&mut self) -> bool {
        self.cache.file(&self.metadata.name, false).modified
            || self.cache.metafile(&self.metadata.name, false).modified
    }

    /// Checks if the intermediate file has been modified.
    pub fn intermediate_modified<T: AsRef<Path>>(&mut self, name: T) -> bool {
        let name = name.as_ref();
        self.cache
            .intermediate(&self.metadata.name, name, false)
            .modified
    }

    /// Gets the full path to specified intermediate file.
    pub fn intermediate<T: AsRef<Path>>(&mut self, name: T, modified: bool) -> PathBuf {
        let name = name.as_ref();
        let md = self.cache.intermediate(&self.metadata.name, name, modified);
        let mut path = self.cache.dir().join(format!("{:X}", md.uuid.simple()));
        if let Some(extension) = name.extension() {
            path = path.with_extension(extension);
        }

        path
    }

    /// Checks if the resource file has been modified.
    pub fn resource_modified<T: AsRef<Path>>(&mut self, name: T) -> bool {
        let name = name.as_ref();
        self.cache
            .resource(&self.metadata.name, name, false)
            .modified
    }

    /// Gets the full path to specified resource file.
    pub fn resource<T: AsRef<Path>>(&mut self, name: T, modified: bool) -> PathBuf {
        let name = name.as_ref();
        let md = self.cache.resource(&self.metadata.name, name, modified);
        self.cache.dir().join(format!("{:X}", md.uuid.simple()))
    }
}
