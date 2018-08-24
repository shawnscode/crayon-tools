pub mod cache;
pub mod settings;
pub mod utils;

pub mod metadata;
pub use self::metadata::AssetMetadata;

pub mod database;
pub use self::database::AssetDatabase;

use std::fs;
use std::path::Path;

use failure::ResultExt;
use toml;

use platform::RuntimePlatform;

pub type Result<T> = ::std::result::Result<T, ::failure::Error>;

pub struct Workspace {
    database: AssetDatabase,
}

impl Workspace {
    pub fn new<T: AsRef<Path>>(root: T, platform: RuntimePlatform, strict: bool) -> Result<Self> {
        // Gets canonicalized and absolute path to root folder.
        let mut root = root.as_ref().to_owned();
        if !root.is_absolute() {
            root = ::std::env::current_dir().unwrap().join(root);
        }

        if !root.is_dir() {
            bail!("The path of {:?} is not a directory.", root);
        }

        root = root.canonicalize().unwrap();

        // Checks if we have configs file, generates one if not exists.
        let file = root.join(settings::NAME);
        if !file.exists() {
            bail!(
                "Can not find workspace.toml configuration file at {:?}.",
                root
            );
        }

        let params: settings::Settings = toml::de::from_str(&fs::read_to_string(&file).unwrap())
            .context("The configs file workspace.toml is broken.")?;

        let workspace = Workspace {
            database: AssetDatabase::new(&root, params.assets, platform, strict)?,
        };

        Ok(workspace)
    }

    pub fn import_all(&mut self) -> Result<()> {
        self.database.import_all()
    }
}

impl Workspace {}
