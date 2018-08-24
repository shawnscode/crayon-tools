mod assbin;
use self::assbin::{Assbin, AssbinMetadata};

mod params;
pub use self::params::{MeshImportParams, TransmissionImportParams};

use std::fs;
use std::io::Write;
use std::process::Command;

use crayon::bincode;
use crayon::video::assets::mesh_loader;
use crayon_3d::assets::prefab::Prefab;
use crayon_3d::assets::prefab_loader;

use super::{AssetImporter, AssetParams, ResourceType};

use workspace::database::{AssetIntermediateGenerator, AssetMetadataGenerator};
use workspace::utils;

pub type Result<T> = ::std::result::Result<T, ::failure::Error>;

pub struct TransmissionImporter {}

impl AssetImporter for TransmissionImporter {
    fn compile(&self, db: &mut AssetIntermediateGenerator) -> Result<()> {
        if !db.modified() && !db.intermediate_modified("transmission.assbin") {
            return Ok(());
        }

        info!("Compiles transmission file {}.", db.name().display());

        let params: TransmissionImportParams = db.params().into();

        let mut cmd = Command::new(utils::current_exe_dir().join("utilities/assimp"));
        cmd.arg("export");
        cmd.arg(&db.path());
        cmd.arg(&db.intermediate("transmission.assbin", true));
        cmd.arg("--triangulate");

        if params.mesh.optimize {
            cmd.arg("--join-identical-vertices");
            cmd.arg("--optimize-meshes");
            cmd.arg("--improve-cache-locality");
        }

        if params.mesh.calculate_normals {
            cmd.arg("--gen-normals");
        }

        if params.mesh.calculate_tangents {
            cmd.arg("--calc-tangent-space");
        }

        if params.mesh.calculate_texcoord {
            cmd.arg("--gen-uvcoords");
        }

        let output = cmd.output().expect("Assimp not found.");
        if !output.status.success() {
            bail!(String::from_utf8(output.stdout.to_owned()).unwrap());
        }

        Ok(())
    }

    fn compile_metadata(&self, db: &mut AssetMetadataGenerator) -> Result<()> {
        let mut file = fs::File::open(db.intermediate("transmission.assbin", false))?;
        let assbin = AssbinMetadata::load(&mut file)?;

        let name = db.name().to_owned();
        db.add(&name, ResourceType::Prefab);
        for v in assbin.meshes {
            db.add(&v, ResourceType::Mesh);
        }

        match db.params() {
            AssetParams::Transmission(_) => {}
            _ => db.update_params(AssetParams::Transmission(
                TransmissionImportParams::default(),
            )),
        }

        Ok(())
    }

    fn import(&self, db: &mut AssetIntermediateGenerator) -> Result<()> {
        if !db.intermediate_modified("transmission.assbin") {
            let mut file = fs::File::open(db.intermediate("transmission.assbin", false))?;
            let assbin = AssbinMetadata::load(&mut file)?;

            let mut modified = false;
            let name = db.name().to_owned();
            if db.resource_modified(&name) {
                modified = true;
            }

            if !modified {
                for v in assbin.meshes {
                    if db.resource_modified(&v) {
                        modified = true;
                        break;
                    }
                }
            }

            if !modified {
                return Ok(());
            }
        }

        let mut file = fs::File::open(db.intermediate("transmission.assbin", false))?;
        let mut assbin = Assbin::load(&mut file)?;

        let mut prefab = Prefab {
            nodes: assbin.nodes,
            universe_meshes: Vec::new(),
            meshes: Vec::new(),
        };

        let name = db.name().to_owned();
        info!("Imports prefab {}.", name.display());

        for v in assbin.meshes.drain(..) {
            info!("Imports mesh {}/{}.", db.name().display(), v.0);

            let mut res = fs::File::create(db.resource(&v.0, true))?;
            res.write_all(&mesh_loader::MAGIC)?;
            bincode::serialize_into(&mut res, &v.1)?;
            bincode::serialize_into(&mut res, &v.2)?;

            prefab.universe_meshes.push(db.uuid(&v.0).unwrap());
        }

        let mut res = fs::File::create(db.resource(name, true))?;
        res.write_all(&prefab_loader::MAGIC)?;
        bincode::serialize_into(&mut res, &prefab)?;
        Ok(())
    }
}
