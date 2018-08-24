/// The runtime platform that we should supports.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimePlatform {
    Macos,
    Windows,
    Ios,
    Android,
}

impl ::std::fmt::Display for RuntimePlatform {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            RuntimePlatform::Macos => write!(f, "macOS"),
            RuntimePlatform::Windows => write!(f, "Windows"),
            RuntimePlatform::Ios => write!(f, "iOS"),
            RuntimePlatform::Android => write!(f, "Android"),
        }
    }
}

/// Specified the compression ratio which will be resolved to concrete format
/// during building process.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compression {
    None,
    LowQuality,
    HighQuality,
}

#[cfg(target_os = "macos")]
impl Default for RuntimePlatform {
    fn default() -> Self {
        RuntimePlatform::Macos
    }
}

#[cfg(target_os = "windows")]
impl Default for RuntimePlatform {
    fn default() -> Self {
        RuntimePlatform::Windows
    }
}
