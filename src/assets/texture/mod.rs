mod params;
pub use self::params::TextureImportParams;

mod ktx;
use self::ktx::Ktx;

use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use crayon::bincode;
use crayon::video::assets::texture::*;
use crayon::video::assets::texture_loader;

use super::{AssetImporter, AssetParams, ResourceType};

use workspace::database::{AssetIntermediateGenerator, AssetMetadataGenerator};
use workspace::utils;

pub type Result<T> = ::std::result::Result<T, ::failure::Error>;

pub struct TextureImporter {}

impl AssetImporter for TextureImporter {
    fn compile(&self, db: &mut AssetIntermediateGenerator) -> Result<()> {
        if !db.modified()
            && !db.intermediate_modified("source.ktx")
            && !db.intermediate_modified("compressed.ktx")
        {
            return Ok(());
        }

        let name = db.name().to_owned();
        info!("Compiles texture {}.", name.display());

        let params: TextureImportParams = db.params().into();
        let png = db.intermediate("source.ktx", true);
        let ktx = db.intermediate("compressed.ktx", true);

        // Pass 1: Converts texture format to png, since `PvrTexTool` does not supports `.PSD` yet.
        Self::convert(&db.path(), &png)?;

        // Pass 2: Converts texture to compressed format.
        let format = params.format(db.platform());
        Self::compress(&png, &ktx, &params, format)?;
        Ok(())
    }

    fn compile_metadata(&self, db: &mut AssetMetadataGenerator) -> Result<()> {
        let name = db.name().to_owned();
        db.add(&name, ResourceType::Texture);

        match db.params() {
            AssetParams::Texture(_) => {}
            _ => db.update_params(AssetParams::Texture(TextureImportParams::default())),
        }

        Ok(())
    }

    fn import(&self, db: &mut AssetIntermediateGenerator) -> Result<()> {
        let name = db.name().to_owned();
        if !db.intermediate_modified("compressed.ktx") && !db.resource_modified(&name) {
            return Ok(());
        }

        let params: TextureImportParams = db.params().into();
        let format = params.format(db.platform());

        let ktx_path = db.intermediate("compressed.ktx", false);
        let ktx = Ktx::parse(&mut fs::File::open(ktx_path).unwrap()).unwrap();

        let mut tex = TextureParams::default();
        tex.format = TextureFormat::RGBA8;
        tex.dimensions = (ktx.pixel_width, ktx.pixel_height).into();
        tex.filter = params.filter;
        tex.wrap = params.wrap;
        tex.format = format;

        info!(
            "Imports resource {}.  Size: {:?}.",
            name.display(),
            ktx.textures.iter().map(|v| v.len()).collect::<Vec<_>>()
        );

        //
        let data = TextureData {
            bytes: ktx.textures,
        };

        let mut file = fs::File::create(db.resource(&name, true))?;
        file.write_all(&texture_loader::MAGIC)?;
        bincode::serialize_into(&mut file, &tex)?;
        bincode::serialize_into(&mut file, &data)?;
        Ok(())
    }
}

impl TextureImporter {
    fn crunch_compress(src: &Path, dst: &Path, params: &TextureImportParams) -> Command {
        let mut cmd = Command::new(utils::current_exe_dir().join("utilities/crunch"));
        cmd.arg("-fileformat ktx");
        cmd.arg(&src);
        cmd.arg("-out");
        cmd.arg(&dst);

        if params.alpha_premultiply {
            // FIXME
        }

        if params.mipmap {
            cmd.arg("-mipMode UseSourceOrGenerate");
        } else {
            cmd.arg("-mipMode None");
        }

        cmd
    }

    fn pvrtc_compress(src: &Path, dst: &Path, params: &TextureImportParams) -> Command {
        let excutable = utils::current_exe_dir().join("utilities/PVRTexToolCLI");
        let mut cmd = Command::new(excutable);
        cmd.arg("-i");
        cmd.arg(&src);
        cmd.arg("-o");
        cmd.arg(&dst);

        if params.alpha_premultiply {
            cmd.arg("-p");
        }

        if params.mipmap {
            cmd.arg("-m");
        }

        cmd
    }

    fn convert(src: &Path, dst: &Path) -> Result<()> {
        let mut cmd = Command::new(utils::current_exe_dir().join("utilities/crunch"));
        cmd.arg("-fileformat ktx");
        cmd.arg(src);
        cmd.arg("-out");
        cmd.arg(dst);
        cmd.arg("-rescalemode nearest");
        cmd.arg("-yflip");
        cmd.arg("-mipMode None");
        cmd.arg("-A8R8G8B8");

        let output = cmd.output().expect("Texture compiler not found.");
        if !output.status.success() {
            bail!(String::from_utf8(output.stderr.to_owned()).unwrap());
        }

        Ok(())
    }

    fn compress(
        src: &Path,
        dst: &Path,
        params: &TextureImportParams,
        format: TextureFormat,
    ) -> Result<()> {
        let mut cmd = match format {
            TextureFormat::S3tcDxt1RGB4BPP => {
                let mut cmd = Self::crunch_compress(src, dst, params);
                cmd.arg("-DXT1");
                cmd
            }
            TextureFormat::S3tcDxt5RGBA8BPP => {
                let mut cmd = Self::crunch_compress(src, dst, params);
                cmd.arg("-DXT5");
                cmd
            }
            TextureFormat::Etc2RGB4BPP => {
                let mut cmd = Self::pvrtc_compress(src, dst, params);
                cmd.arg("-f ETC2_RGB");
                cmd
            }
            TextureFormat::Etc2RGBA8BPP => {
                let mut cmd = Self::pvrtc_compress(src, dst, params);
                cmd.arg("-f ETC2_RGBA");
                cmd
            }
            TextureFormat::PvrtcRGB4BPP => {
                let mut cmd = Self::pvrtc_compress(src, dst, params);
                cmd.arg("-f PVRTC1_4_RGB");
                cmd
            }
            TextureFormat::PvrtcRGB2BPP => {
                let mut cmd = Self::pvrtc_compress(src, dst, params);
                cmd.arg("-f PVRTC1_2_RGB");
                cmd
            }
            TextureFormat::PvrtcRGBA4BPP => {
                let mut cmd = Self::pvrtc_compress(src, dst, params);
                cmd.arg("-f PVRTC1_4");
                cmd
            }
            TextureFormat::PvrtcRGBA2BPP => {
                let mut cmd = Self::pvrtc_compress(src, dst, params);
                cmd.arg("-f PVRTC1_2");
                cmd
            }
            TextureFormat::R8 => {
                let mut cmd = Self::crunch_compress(src, dst, params);
                cmd.arg("-L8");
                cmd
            }
            TextureFormat::RGB8 => {
                let mut cmd = Self::crunch_compress(src, dst, params);
                cmd.arg("-R8G8B8");
                cmd
            }
            TextureFormat::RGBA8 => {
                let mut cmd = Self::crunch_compress(src, dst, params);
                cmd.arg("-A8R8G8B8");
                cmd
            }
            _ => unimplemented!(),
        };

        let output = cmd.output().expect("Texture compiler not found.");
        if !output.status.success() {
            bail!(String::from_utf8(output.stdout.to_owned()).unwrap());
        }

        Ok(())
    }
}
