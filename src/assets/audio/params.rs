use assets::AssetParams;
use platform::Compression;

/// Settings of importing sound effect assets.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
pub struct AudioImportParams {
    /// The optional override sample rate of imported sound effect.
    pub sample_rate: Option<SampleRate>,
    /// Compression level of imported sound effect.
    pub compression: Compression,
}

/// List of common sample rates.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
pub enum SampleRate {
    /// Adequate for human speech but without sibilance. Used in telephone/walkie-talkie.
    Hz8000,
    /// Used for lower-quality PCM, MPEG audio and for audio analysis of subwoofer bandpasses.
    Hz11025,
    /// Used for lower-quality PCM and MPEG audio and for audio analysis of low frequency energy.
    Hz22050,
    /// Audio CD, most commonly used rate with MPEG-1 audio (VCD, SVCD, MP3). Covers the 20 kHz bandwidth.
    Hz44100,
    /// Standard sampling rate used by professional digital video equipment, could reconstruct frequencies up to 22 kHz.
    Hz48000,
    /// DVD-Audio, LPCM DVD tracks, Blu-ray audio tracks, HD DVD audio tracks.
    Hz96000,
}

impl Default for AudioImportParams {
    fn default() -> Self {
        AudioImportParams {
            sample_rate: None,
            compression: Compression::HighQuality,
        }
    }
}

impl From<AssetParams> for AudioImportParams {
    fn from(params: AssetParams) -> Self {
        match params {
            AssetParams::Audio(params) => params,
            _ => AudioImportParams::default(),
        }
    }
}
