use assets::AssetParams;
use crayon::video::assets::texture::*;
use platform::{Compression, RuntimePlatform};

/// Settings of importing texture assets.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
pub struct TextureImportParams {
    /// Select this to enable mip-map generation. Mip maps are smaller versions
    /// of the Texture that get used when the Texture is very small on screen.
    pub mipmap: bool,
    /// Premultiplys the RGB with A, enable this to prefilter the color to avoid
    /// filtering artifacts.
    pub alpha_premultiply: bool,
    /// Texture coordinate wrapping mode.
    pub wrap: TextureWrap,
    /// Filtering mode of the texture.
    pub filter: TextureFilter,
    /// Compression level of imported texture.
    pub compression: Compression,
}

impl Default for TextureImportParams {
    fn default() -> Self {
        TextureImportParams {
            mipmap: true,
            alpha_premultiply: false,
            wrap: TextureWrap::Clamp,
            filter: TextureFilter::Linear,
            compression: Compression::HighQuality,
        }
    }
}

impl From<AssetParams> for TextureImportParams {
    fn from(params: AssetParams) -> Self {
        match params {
            AssetParams::Texture(params) => params,
            _ => TextureImportParams::default(),
        }
    }
}

impl TextureImportParams {
    pub fn format(&self, platform: RuntimePlatform) -> TextureFormat {
        match platform {
            RuntimePlatform::Macos => match self.compression {
                Compression::None => TextureFormat::RGBA8,
                Compression::LowQuality => TextureFormat::S3tcDxt5RGBA8BPP,
                Compression::HighQuality => TextureFormat::S3tcDxt5RGBA8BPP,
            },
            RuntimePlatform::Windows => match self.compression {
                Compression::None => TextureFormat::RGBA8,
                Compression::LowQuality => TextureFormat::S3tcDxt5RGBA8BPP,
                Compression::HighQuality => TextureFormat::S3tcDxt5RGBA8BPP,
            },
            RuntimePlatform::Ios => match self.compression {
                Compression::None => TextureFormat::RGBA8,
                Compression::LowQuality => TextureFormat::PvrtcRGBA2BPP,
                Compression::HighQuality => TextureFormat::PvrtcRGBA4BPP,
            },
            RuntimePlatform::Android => match self.compression {
                Compression::None => TextureFormat::RGBA8,
                Compression::LowQuality => TextureFormat::Etc2RGBA8BPP,
                Compression::HighQuality => TextureFormat::Etc2RGBA8BPP,
            },
        }
    }
}
