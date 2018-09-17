pub mod texture;
pub use self::texture::{TextureImportParams, TextureImporter};

pub mod transmission;
pub use self::transmission::{MeshImportParams, TransmissionImportParams, TransmissionImporter};

pub mod bytes;
pub use self::bytes::BytesImporter;

pub mod audio;
pub use self::audio::{AudioImportParams, AudioImporter};

use workspace::database::{AssetIntermediateGenerator, AssetMetadataGenerator};

pub type Result<T> = ::std::result::Result<T, ::failure::Error>;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetType {
    Texture,
    Transmission,
    Bytes,
    Audio,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceType {
    Texture,
    Prefab,
    Mesh,
    Bytes,
    AudioClip,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum AssetParams {
    Bytes,
    Texture(TextureImportParams),
    Transmission(TransmissionImportParams),
    Audio(AudioImportParams),
}

pub trait AssetImporter {
    /// Compiles the assets into some kind of intermediate files for importing process.
    fn compile(&self, db: &mut AssetIntermediateGenerator) -> Result<()>;

    /// Compiles the metadata of asset.
    fn compile_metadata(&self, db: &mut AssetMetadataGenerator) -> Result<()>;

    /// Imports resource files, which could be loaded at runtime.
    fn import(&self, db: &mut AssetIntermediateGenerator) -> Result<()>;
}
