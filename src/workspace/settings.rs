use std::path::PathBuf;

use assets::AssetType;

pub const NAME: &str = "workspace.toml";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    pub assets: AssetSettings,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AssetSettings {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub importers: Vec<AssetExtensions>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AssetExtensions {
    #[serde(rename = "type")]
    pub tp: AssetType,
    pub extensions: Vec<String>,
}
