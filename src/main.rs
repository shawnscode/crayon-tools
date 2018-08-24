#[macro_use]
extern crate crayon;
extern crate crayon_3d;
#[macro_use]
extern crate failure;
extern crate env_logger;

#[macro_use]
extern crate serde;
extern crate serde_json;
extern crate toml;

extern crate byteorder;
extern crate crc;
extern crate uuid;
extern crate walkdir;

extern crate clap;

pub mod assets;
pub mod platform;
pub mod workspace;

use crayon::errors::*;

use clap::{App, Arg, ArgMatches, SubCommand};
use std::path::Path;

fn main() -> Result<()> {
    env_logger::Builder::new()
        .filter_level(crayon::LevelFilter::Info)
        .init();

    let matches = App::new("crayon-tools")
        .version("0.0.1")
        .subcommand(
            SubCommand::with_name("build")
                .about("Builds assets into platform-dependent resources.")
                .arg(
                    Arg::with_name("path")
                        .short("p")
                        .help("Sets the root path of workspace.")
                        .takes_value(true),
                ),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("build") {
        return cmd_build(matches);
    }

    Ok(())
}

fn cmd_build<'a>(matches: &ArgMatches<'a>) -> Result<()> {
    let path = match matches.value_of("path") {
        Some(v) => Path::new(v).into(),
        None => ::std::env::current_dir().unwrap(),
    };

    let mut ws = workspace::Workspace::new(&path, platform::RuntimePlatform::Macos, true)?;
    ws.import_all()?;
    return Ok(());
}
