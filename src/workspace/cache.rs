use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use crc::crc32;
use toml;
use uuid::Uuid;
use walkdir::WalkDir;

use super::metadata::AssetMetadata;
use super::utils;

pub type Result<T> = ::std::result::Result<T, ::failure::Error>;
pub const NAME: &str = "intermediates.toml";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AssetCacheItem {
    pub file: Metadata,
    pub metafile: Metadata,
    pub resources: HashMap<PathBuf, Metadata>,
    pub intermediates: HashMap<PathBuf, Metadata>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AssetCache {
    strict: bool,
    assets_dir: PathBuf,
    dir: PathBuf,
    assets: HashMap<PathBuf, AssetCacheItem>,
}

impl AssetCache {
    pub fn new(assets_dir: &Path, strict: bool) -> Result<AssetCache> {
        let mut hasher = DefaultHasher::new();
        assets_dir.hash(&mut hasher);

        let dir = utils::current_exe_dir()
            .join("intermediates")
            .join(format!("{:X}/", hasher.finish()));

        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }

        let manifest = dir.join(NAME);
        let assets = fs::read(&manifest)
            .ok()
            .and_then(|v| toml::de::from_slice(&v).ok())
            .unwrap_or_default();

        info!("Generates cahce of intermediate files at {:?}.", dir);

        let mut cache = AssetCache {
            assets_dir: assets_dir.to_owned(),
            dir: dir,
            assets: assets,
            strict: strict,
        };

        cache.scan()?;
        Ok(cache)
    }

    // Imports files.
    fn scan(&mut self) -> Result<()> {
        for (n, v) in &mut self.assets {
            v.file.check_file(&self.assets_dir, n, self.strict)?;
            v.metafile.check_metafile(&self.assets_dir, n, self.strict)?;

            for (k, metadata) in &mut v.intermediates {
                metadata.check_intermediate(&self.dir, k, self.strict)?;
            }

            for (_, metadata) in &mut v.resources {
                metadata.check_resource(&self.dir, self.strict)?;
            }
        }

        Ok(())
    }

    pub fn strip(&mut self, assets: &HashMap<PathBuf, AssetMetadata>) -> Result<()> {
        // Strips deprecated entries in this cache.
        self.assets.retain(|k, v| {
            if let Some(w) = assets.get(k) {
                v.resources.retain(|kk, _| {
                    for r in &w.resources {
                        if &r.name == kk {
                            return true;
                        }
                    }

                    false
                });

                true
            } else {
                false
            }
        });

        //
        let mut intermediates = HashSet::new();
        for (_, v) in &mut self.assets {
            for (k, metadata) in &mut v.intermediates {
                intermediates.insert(metadata.path_with_extension(&self.dir, k));
            }

            for (_, metadata) in &mut v.resources {
                intermediates.insert(metadata.path(&self.dir));
            }
        }

        //
        for e in WalkDir::new(&self.dir).into_iter().filter_map(|e| e.ok()) {
            if e.file_type().is_dir() {
                continue;
            }

            if e.path().file_name().unwrap() == NAME {
                continue;
            }

            if !intermediates.contains(e.path()) {
                fs::remove_file(e.path())?;
            }
        }

        Ok(())
    }

    pub fn save(&mut self) -> Result<()> {
        for (n, v) in &mut self.assets {
            if v.file.modified {
                v.file.check_file(&self.assets_dir, n, self.strict)?;
            }

            if v.metafile.modified {
                v.metafile.check_metafile(&self.assets_dir, n, self.strict)?;
            }

            for (k, metadata) in &mut v.intermediates {
                if metadata.modified {
                    metadata.check_intermediate(&self.dir, k, self.strict)?;
                }
            }

            for (_, metadata) in &mut v.resources {
                if metadata.modified {
                    metadata.check_resource(&self.dir, self.strict)?;
                }
            }
        }

        let contents = toml::ser::to_string_pretty(&self.assets)?;
        let manifest = self.dir.join(NAME);
        fs::write(manifest, contents)?;
        Ok(())
    }

    #[inline]
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn item<T: AsRef<Path>>(&mut self, filename: T) -> &mut AssetCacheItem {
        self.assets
            .entry(filename.as_ref().to_owned())
            .or_insert_with(|| AssetCacheItem {
                file: Metadata::new(Uuid::new_v4()),
                metafile: Metadata::new(Uuid::new_v4()),
                resources: HashMap::new(),
                intermediates: HashMap::new(),
            })
    }

    pub fn file<T: AsRef<Path>>(&mut self, filename: T, writable: bool) -> Metadata {
        let item = self.item(filename);
        if writable {
            item.file.modified = true;
        }

        item.file
    }

    pub fn metafile<T: AsRef<Path>>(&mut self, filename: T, writable: bool) -> Metadata {
        let item = self.item(filename);
        if writable {
            item.metafile.modified = true;
        }

        item.metafile
    }

    pub fn intermediate<T1, T2>(&mut self, filename: T1, name: T2, writable: bool) -> Metadata
    where
        T1: AsRef<Path>,
        T2: AsRef<Path>,
    {
        let name = utils::canonicalize(name.as_ref());
        let item = self.item(filename);
        let metadata = item.intermediates
            .entry(name)
            .or_insert_with(|| Metadata::new(Uuid::new_v4()));

        if writable {
            metadata.modified = true;
        }

        *metadata
    }

    pub fn resource<T1, T2>(&mut self, filename: T1, name: T2, writable: bool) -> Metadata
    where
        T1: AsRef<Path>,
        T2: AsRef<Path>,
    {
        let name = utils::canonicalize(name.as_ref());
        let item = self.item(filename);
        let metadata = item.resources
            .entry(name)
            .or_insert_with(|| Metadata::new(Uuid::new_v4()));

        if writable {
            metadata.modified = true;
        }

        *metadata
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Metadata {
    pub uuid: Uuid,
    pub checksum: u32,
    pub last_modified_time: u32,

    #[serde(skip)]
    pub modified: bool,
}

impl Metadata {
    pub fn new(uuid: Uuid) -> Self {
        Metadata {
            uuid: uuid,
            checksum: 0,
            last_modified_time: 0,
            modified: true,
        }
    }

    pub fn check_intermediate(&mut self, dir: &Path, k: &Path, strict: bool) -> Result<()> {
        let path = self.path_with_extension(dir, k);
        self.check(&path, strict)
    }

    pub fn check_resource(&mut self, dir: &Path, strict: bool) -> Result<()> {
        let path = self.path(dir);
        self.check(&path, strict)
    }

    pub fn path(&self, dir: &Path) -> PathBuf {
        dir.join(format!("{:X}", self.uuid.simple()))
    }

    pub fn path_with_extension(&self, dir: &Path, filename: &Path) -> PathBuf {
        if let Some(extension) = filename.extension() {
            self.path(dir).with_extension(extension)
        } else {
            self.path(dir)
        }
    }

    pub fn check_file(&mut self, dir: &Path, filename: &Path, strict: bool) -> Result<()> {
        let path = dir.join(filename);
        self.check(&path, strict)
    }

    pub fn check_metafile(&mut self, dir: &Path, filename: &Path, strict: bool) -> Result<()> {
        let path = AssetMetadata::with_extension(&dir.join(filename));
        self.check(&path, strict)
    }

    fn check(&mut self, path: &Path, strict: bool) -> Result<()> {
        if path.exists() {
            let file_md = fs::metadata(&path).unwrap();
            let wtime = file_md.modified().unwrap();
            let wtime = wtime.duration_since(UNIX_EPOCH).unwrap().as_secs() as u32;

            if strict {
                let crc = crc32::checksum_ieee(&fs::read(&path).unwrap());
                self.modified = crc != self.checksum;
                self.checksum = crc;
            } else {
                self.modified = self.last_modified_time != wtime;
            }

            self.last_modified_time = wtime;
        } else {
            self.checksum = 0;
            self.last_modified_time = 0;
            self.modified = true;
        }

        Ok(())
    }
}
