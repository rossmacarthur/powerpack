mod alfred;
mod cargo;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{AppSettings, Clap};
use toml_edit as toml;

fn prompt_for_workflow_info(doc: &toml::Document) -> Result<alfred::WorkflowInfo> {
    let package_name = doc["package"]["name"].as_str().context("expected string")?;

    let author = doc["package"]["authors"][0]
        .as_str()
        .context("expected string")?;
    let author = match author.find(" <") {
        Some(i) => &author[i..],
        None => author,
    };

    Ok(alfred::WorkflowInfo {
        name: package_name.to_owned(),
        bin_name: package_name.to_owned(),
        bundle_id: casual::prompt("Enter the workflow bundle ID: ").get(),
        author: author.to_owned(),
        description: casual::prompt("Enter the workflow description: ").get(),
        keyword: casual::prompt("Enter the workflow keyword: ").get(),
    })
}

/// Create a new Alfred workflow in the given directory.
fn init(manifest_dir: &Path) -> Result<()> {
    cargo::init(manifest_dir)?;
    let doc = cargo::read_manifest(manifest_dir)?;

    // Write the info.plist file
    let info = prompt_for_workflow_info(&doc)?;
    let info = alfred::build_info_plist(&info);
    let workflow_dir = manifest_dir.join("workflow");
    fs::create_dir_all(&workflow_dir)?;
    info.to_file_xml(workflow_dir.join("info.plist"))?;

    // Add dependencies to Cargo manifest.
    {
        let mut doc = doc;
        let table = &mut doc["dependencies"];
        table["anyhow"] = toml::value("1.0");
        table["powerpack"] = toml::value(env!("CARGO_PKG_VERSION"));
        cargo::write_manifest(manifest_dir, &doc)?;
    }

    // Write our custom `main.rs`
    let main = manifest_dir.join("src").join("main.rs");
    fs::write(main, include_str!("main.rs.template"))?;

    Ok(())
}

/// Build the workflow.
fn build() -> Result<()> {
    cargo::build()?;
    let workspace_dir = cargo::workspace_directory()?;
    let target_dir = cargo::target_directory()?;
    let binary_name = cargo::binary_name()?;
    fs::create_dir_all(workspace_dir.join("workflow"))?;
    fs::copy(
        target_dir.join("release").join(&binary_name),
        workspace_dir.join("workflow").join(&binary_name),
    )?;
    Ok(())
}

#[derive(Debug, Clap)]
enum Command {
    /// Create a new Rust alfred workflow.
    New { path: PathBuf },
    /// Create a new Rust alfred workflow in an existing directory [default: .]
    Init { path: Option<PathBuf> },
    /// Build the workflow.
    Build,
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
            fs::create_dir_all(&path)?;
            init(&path)?;
        }
        Command::Init { path } => {
            let path = path
                .as_ref()
                .map(PathBuf::as_path)
                .unwrap_or(Path::new("."));
            init(path)?;
        }
        Command::Build => {
            build()?;
        }
    }
    Ok(())
}
