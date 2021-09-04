mod alfred;
mod cargo;

use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::prelude::*;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{AppSettings, Clap};
use peter::Stylize;
use toml_edit as toml;

fn print(header: &str, message: impl AsRef<str>) {
    if atty::is(atty::Stream::Stdout) {
        println!("{:>12} {}", header.bold().green(), message.as_ref());
    } else {
        println!("{:>12} {}", header, message.as_ref());
    }
}

fn prompt_for_workflow_info(doc: &toml::Document) -> Result<alfred::WorkflowInfo> {
    let package_name = doc["package"]["name"].as_str().context("expected string")?;
    println!("Please enter the workflow details:");
    Ok(alfred::WorkflowInfo {
        name: package_name.to_owned(),
        bin_name: package_name.to_owned(),
        bundle_id: casual::prompt("Bundle ID: ").get(),
        author: casual::prompt("Author: ").get(),
        description: casual::prompt("Description: ").get(),
        keyword: casual::prompt("Keyword: ").get(),
    })
}

/// Create a new Alfred workflow in the given directory.
fn init(manifest_dir: &Path, name: Option<OsString>) -> Result<()> {
    cargo::init(manifest_dir, name)?;
    let doc = cargo::read_manifest(manifest_dir).context("failed to read Cargo manifest")?;
    let package_name = doc["package"]["name"].as_str().context("expected string")?;

    // Write the info.plist file
    let info = prompt_for_workflow_info(&doc)?;
    let info = alfred::build_info_plist(&info);
    let workflow_dir = manifest_dir.join("workflow");
    fs::create_dir_all(&workflow_dir)?;
    info.to_file_xml(workflow_dir.join("info.plist"))?;

    // Add workflow/<binary> to the gitignore file (if it exists)
    if let Ok(mut file) = fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open(manifest_dir.join(".gitignore"))
    {
        writeln!(file, "/workflow/{}", package_name)?;
    }

    // Add dependencies to Cargo manifest.
    {
        let mut doc = doc;
        let table = &mut doc["dependencies"];
        table["powerpack"] = toml::value(env!("CARGO_PKG_VERSION"));
        cargo::write_manifest(manifest_dir, &doc)?;
    }

    // Write our custom `main.rs`
    let main = manifest_dir.join("src").join("main.rs");
    fs::write(&main, include_str!("main.template.rs"))?;
    print("Finished", "created example script filter workflow");

    Ok(())
}

/// Build the workflow.
fn build(release: bool) -> Result<()> {
    let mode = if release {
        cargo::Mode::Release
    } else {
        cargo::Mode::Debug
    };
    cargo::build(mode)?;

    let workspace_dir = cargo::workspace_directory()?;
    let target_dir = cargo::target_directory()?;
    let binary_name = cargo::binary_name()?;
    fs::create_dir_all(workspace_dir.join("workflow"))?;

    let src = target_dir.join(mode.dir()).join(&binary_name);
    let dst = workspace_dir.join("workflow").join(&binary_name);
    fs::copy(&src, &dst)?;

    print(
        "Copied",
        format!(
            "binary to `{}`",
            dst.strip_prefix(env::current_dir()?)?.display()
        ),
    );

    Ok(())
}

fn find_link(workflow_dir: &Path, workflows_dir: &Path) -> Result<Option<PathBuf>> {
    for entry in fs::read_dir(&workflows_dir)?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|entry| entry.file_type().unwrap().is_symlink())
    {
        let path = entry.path();
        if path.read_link()? == workflow_dir {
            return Ok(Some(path));
        }
    }
    Ok(None)
}

/// Link the workflow.
fn link() -> Result<()> {
    let workflow_dir = cargo::workspace_directory()?.join("workflow");
    let workflows_dir = alfred::workflows_directory()?;

    match find_link(&workflow_dir, &workflows_dir)? {
        Some(exists) => {
            print(
                "Symlinked",
                format!("workflow directory to `{}`", exists.display()),
            );
        }
        None => {
            let uid = uuid::Uuid::new_v4().to_string().to_uppercase();
            let dst = workflows_dir.join(&format!("user.workflow.{}", uid));
            symlink(&workflow_dir, &dst)?;
            print(
                "Symlinked",
                format!("workflow directory to `{}`", dst.display()),
            );
        }
    }
    Ok(())
}

/// Package the workflow into a `.alfredworkflow` file.
fn package() -> Result<()> {
    let workflow_dir = cargo::workspace_directory()?.join("workflow");
    let dist_dir = cargo::target_directory()?.join("workflow");
    let mut binary_name = cargo::binary_name()?;

    // Just a hack because I tend to suffix my workflows with this.
    if let Some(new) = binary_name.strip_suffix("-alfred-workflow") {
        binary_name = new.to_owned();
    }

    let dst = &dist_dir.join(binary_name).with_extension("alfredworkflow");

    fs::create_dir_all(&dist_dir)?;
    alfred::package(&workflow_dir, dst)?;
    print(
        "Packaged",
        format!(
            "workflow at `{}`",
            dst.strip_prefix(env::current_dir()?)?.display()
        ),
    );

    Ok(())
}

#[derive(Debug, Clap)]
enum Command {
    /// Create a new Rust alfred workflow.
    New {
        path: PathBuf,
        #[clap(long)]
        name: Option<OsString>,
    },
    /// Create a new Rust alfred workflow in an existing directory [default: .]
    Init {
        path: Option<PathBuf>,
        #[clap(long)]
        name: Option<OsString>,
    },
    /// Build the workflow.
    Build {
        #[clap(long)]
        release: bool,
    },
    /// Symlink the workflow directory to the Alfred workflow directory.
    Link,
    /// Package the workflow as an `.alfredworkflow` file.
    Package,
}

#[derive(Debug, Clap)]
#[clap(
    about,
    author,
    version,
    setting = AppSettings::DeriveDisplayOrder,
    setting = AppSettings::DisableHelpSubcommand,
    setting = AppSettings::DisableVersionForSubcommands,
    setting = AppSettings::PropagateVersion,
    setting = AppSettings::SubcommandRequiredElseHelp,
)]
struct Opt {
    #[clap(subcommand)]
    command: Command,
}

fn main() -> anyhow::Result<()> {
    let Opt { command } = Opt::parse();
    match command {
        Command::New { path, name } => {
            fs::create_dir_all(&path)?;
            init(&path, name)?;
        }
        Command::Init { path, name } => {
            let path = path.as_deref().unwrap_or_else(|| Path::new("."));
            init(path, name)?;
        }
        Command::Build { release } => {
            build(release)?;
        }
        Command::Link => {
            link()?;
        }
        Command::Package => {
            build(true)?;
            package()?;
        }
    }
    Ok(())
}
