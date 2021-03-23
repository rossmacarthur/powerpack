mod cargo;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use clap::{AppSettings, Clap};

fn update_cargo_manifest(manifest_dir: &Path) -> anyhow::Result<()> {
    use toml_edit::{Document, Item::Value};

    let manifest_path = manifest_dir.join("Cargo.toml");
    let contents = fs::read_to_string(&manifest_path)?;
    let mut doc = Document::from_str(&contents)?;
    let table = &mut doc["dependencies"];
    table["anyhow"] = Value("1.0".into());
    table["powerpack"] = Value(env!("CARGO_PKG_VERSION").into());
    fs::write(&manifest_path, &doc.to_string_in_original_order())?;
    Ok(())
}

fn write_main(manifest_dir: &Path) -> io::Result<()> {
    let path = manifest_dir.join("src").join("main.rs");
    fs::write(path, include_str!("main.rs.template"))
}

#[derive(Debug, Clap)]
enum Command {
    /// Create a new Rust alfred workflow.
    New { path: PathBuf },
    /// Create a new Rust alfred workflow in an existing directory.
    Init,
}

#[derive(Debug, Clap)]
#[clap(
    author,
    about,
    global_setting = AppSettings::DeriveDisplayOrder,
    global_setting = AppSettings::DisableHelpSubcommand,
    global_setting = AppSettings::GlobalVersion,
    global_setting = AppSettings::VersionlessSubcommands,
    setting = AppSettings::SubcommandRequiredElseHelp,
)]
struct Opt {
    #[clap(subcommand)]
    command: Command,
}

fn main() -> anyhow::Result<()> {
    let Opt { command } = Opt::parse();
    match command {
        Command::New { path } => {
            cargo::new(&path).run()?;
            write_main(&path)?;
            update_cargo_manifest(&path)?;
        }
        Command::Init => {
            cargo::init().run()?;
            write_main(".".as_ref())?;
            update_cargo_manifest(".".as_ref())?;
        }
    }
    Ok(())
}
