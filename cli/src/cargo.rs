use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::str::FromStr;

use anyhow::{bail, Context, Result};
pub use cargo_metadata as metadata;
use toml_edit as toml;

#[derive(Debug)]
pub struct Cargo {
    cmd: process::Command,
}

impl Cargo {
    fn new<S: AsRef<OsStr>>(cmd: S) -> Self {
        Self {
            cmd: process::Command::new("cargo"),
        }
        .arg(cmd)
    }

    fn arg<S: AsRef<OsStr>>(mut self, arg: S) -> Self {
        self.cmd.arg(arg);
        self
    }

    /// Run the `cargo` process.
    fn run(&mut self) -> Result<()> {
        if !self.cmd.status()?.success() {
            bail!("`cargo` did not exit successfully");
        }
        Ok(())
    }
}

/// Run a `cargo init` command.
pub fn init<P: AsRef<OsStr>>(path: P) -> Result<()> {
    Cargo::new("init").arg("--bin").arg(path).run()
}

/// Run a `cargo build` command.
pub fn build() -> Result<()> {
    Cargo::new("build").arg("--release").run()
}

/// Get the release binary path.
pub fn binary_name() -> Result<String> {
    let metadata = metadata::MetadataCommand::new().exec()?;
    let package = metadata.root_package().context("no root package")?;
    let binary = package
        .targets
        .iter()
        .find(|target| target.kind.iter().any(|kind| kind == "bin"))
        .context("no binary")?
        .name
        .clone();
    Ok(binary)
}

/// Get the workspace directory.
pub fn workspace_directory() -> Result<PathBuf> {
    let metadata = metadata::MetadataCommand::new().exec()?;
    Ok(metadata.workspace_root.into())
}

/// Get the target directory.
pub fn target_directory() -> Result<PathBuf> {
    let metadata = metadata::MetadataCommand::new().exec()?;
    Ok(metadata.target_directory.into())
}

/// Read the Cargo manifest.
pub fn read_manifest(dir: &Path) -> Result<toml::Document> {
    let manifest_path = dir.join("Cargo.toml");
    let contents = fs::read_to_string(&manifest_path)?;
    let doc = toml::Document::from_str(&contents)?;
    Ok(doc)
}

/// Write a Cargo manifest.
pub fn write_manifest(dir: &Path, doc: &toml::Document) -> Result<()> {
    let path = dir.join("Cargo.toml");
    fs::write(&path, &doc.to_string_in_original_order())?;
    Ok(())
}
