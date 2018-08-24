use std::fs;
use std::path::{Path, PathBuf};

use toml;
use uuid::Uuid;

use super::Result;
use assets::{AssetParams, ResourceType};

pub const EXTENSION: &str = ".meta.toml";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AssetMetadata {
    pub name: PathBuf,
    pub uuid: Uuid,
    pub resources: Vec<ResourceMetadata>,
    pub params: AssetParams,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResourceMetadata {
    #[serde(rename = "type")]
    pub tp: ResourceType,
    pub name: PathBuf,
    pub uuid: Uuid,
}

impl ResourceMetadata {
    pub fn new(name: PathBuf, tp: ResourceType) -> Self {
        ResourceMetadata {
            tp: tp,
            name: name,
            uuid: Uuid::new_v4(),
        }
    }
}

impl AssetMetadata {
    pub fn import_from<T: AsRef<Path>>(assets_dir: T, filename: T) -> Result<AssetMetadata> {
        let filename = filename.as_ref();
        let file = assets_dir.as_ref().join(filename);
        let metafile = Self::with_extension(&file);

        if metafile.exists() {
            let bytes = fs::read(&metafile).unwrap();
            match toml::de::from_slice::<AssetMetadata>(&bytes) {
                Ok(mut metadata) => {
                    metadata.name = filename.to_owned();
                    return Ok(metadata);
                }
                Err(err) => {
                    warn!(
                        "A meta data file ({:?}) seems broken. {}",
                        filename.display(),
                        err
                    );
                }
            }
        }

        let metadata = AssetMetadata {
            name: filename.to_owned(),
            uuid: Uuid::new_v4(),
            params: AssetParams::Bytes,
            resources: Vec::new(),
        };

        let contents = toml::ser::to_string_pretty(&metadata).unwrap();
        fs::write(metafile, contents)?;
        Ok(metadata)
    }

    pub fn save<T: AsRef<Path>>(&self, assets_dir: T, filename: T) -> Result<()> {
        let metafile = Self::with_extension(&assets_dir.as_ref().join(filename.as_ref()));
        let contents = toml::ser::to_string_pretty(self).unwrap();
        fs::write(metafile, contents)?;
        Ok(())
    }

    pub fn with_extension<T: AsRef<Path>>(path: T) -> PathBuf {
        let path = path.as_ref();
        if let Some(v) = path.extension() {
            path.with_extension(v.to_str().unwrap().to_owned() + EXTENSION)
        } else {
            path.with_extension(EXTENSION)
        }
    }
}
