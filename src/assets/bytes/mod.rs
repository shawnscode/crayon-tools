use std::fs;

use assets::{AssetImporter, AssetParams, ResourceType};
use workspace::database::{AssetIntermediateGenerator, AssetMetadataGenerator};

pub type Result<T> = ::std::result::Result<T, ::failure::Error>;

pub struct BytesImporter {}

impl AssetImporter for BytesImporter {
    fn compile(&self, _: &mut AssetIntermediateGenerator) -> Result<()> {
        Ok(())
    }

    fn compile_metadata(&self, db: &mut AssetMetadataGenerator) -> Result<()> {
        let name = db.name().to_owned();
        db.add(&name, ResourceType::Bytes);

        if db.params() != AssetParams::Bytes {
            db.update_params(AssetParams::Bytes);
        }

        Ok(())
    }

    fn import(&self, db: &mut AssetIntermediateGenerator) -> Result<()> {
        if !db.modified() {
            return Ok(());
        }

        info!("Imports bytes {}.", db.name().display());

        let name = db.name().to_owned();
        fs::copy(&db.path(), db.resource(&name, true))?;
        Ok(())
    }
}
