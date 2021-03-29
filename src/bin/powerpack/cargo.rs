use std::ffi::OsStr;
use std::path::PathBuf;
use std::process;

use anyhow::{bail, Context, Result};
pub use cargo_metadata as metadata;

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

/// Run a `cargo new` command.
pub fn new<P: AsRef<OsStr>>(path: P) -> Result<()> {
    Cargo::new("new").arg("--bin").arg(path).run()
}

/// Run a `cargo init` command.
pub fn init() -> Result<()> {
    Cargo::new("init").arg("--bin").run()
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
    Ok(metadata.workspace_directory.into())
}

/// Get the target directory.
pub fn target_directory() -> Result<PathBuf> {
    let metadata = metadata::MetadataCommand::new().exec()?;
    Ok(metadata.target_directory.into())
}
