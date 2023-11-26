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

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Debug,
    Release,
}

#[derive(Debug)]
pub struct Metadata {
    pub workspace_dir: PathBuf,
    pub manifest_dir: PathBuf,
    pub target_dir: PathBuf,
    pub package_name: String,
    pub binary_names: Vec<String>,
}

impl Cargo {
    fn new<S: AsRef<OsStr>>(subcmd: S) -> Self {
        let mut cmd = process::Command::new("cargo");
        cmd.arg(subcmd);
        Self { cmd }
    }

    fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
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

impl Mode {
    pub fn dir(&self) -> &Path {
        Path::new(match self {
            Self::Debug => "debug",
            Self::Release => "release",
        })
    }
}

/// Run a `cargo init` command.
pub fn init<P, N>(path: P, name: Option<N>) -> Result<()>
where
    P: AsRef<OsStr>,
    N: AsRef<OsStr>,
{
    let mut cmd = Cargo::new("init");
    if let Some(name) = name {
        cmd.arg("--name").arg(name);
    }
    cmd.arg("--bin").arg(path);
    cmd.run()
}

/// Run a `cargo build` command.
pub fn build(
    mode: Mode,
    package: Option<&str>,
    bins: &[String],
    target: Option<&str>,
) -> Result<()> {
    let mut cmd = Cargo::new("build");
    if let Some(package) = package {
        cmd.arg("--package").arg(package);
    }
    if let Mode::Release = mode {
        cmd.arg("--release");
    }
    for bin in bins {
        cmd.arg("--bin");
        cmd.arg(bin);
    }
    if let Some(target) = target {
        cmd.arg("--target");
        cmd.arg(target);
    }
    cmd.run()
}

/// Run a `cargo metadata` command.
pub fn metadata(package: Option<&str>) -> Result<Metadata> {
    let metadata::Metadata {
        packages,
        workspace_root,
        target_directory,
        resolve,
        ..
    } = metadata::MetadataCommand::new().exec()?;

    let pkg = match package {
        Some(n) => packages
            .into_iter()
            .find(|pkg| pkg.name == n)
            .with_context(|| format!("package not found: `{}`", n))?,
        None => (move || {
            let root = resolve.as_ref()?.root.as_ref()?;
            packages.into_iter().find(|pkg| &pkg.id == root)
        })()
        .context("no root package")?,
    };

    let binary_names = pkg
        .targets
        .into_iter()
        .filter(|target| target.kind.iter().any(|kind| kind == "bin"))
        .map(|target| target.name)
        .collect();

    Ok(Metadata {
        workspace_dir: workspace_root.into(),
        manifest_dir: pkg.manifest_path.parent().unwrap().into(),
        target_dir: target_directory.into(),
        package_name: pkg.name,
        binary_names,
    })
}

/// Read the Cargo manifest.
pub fn read_manifest(dir: &Path) -> Result<toml::Document> {
    let manifest_path = dir.join("Cargo.toml");
    let contents = fs::read_to_string(manifest_path)?;
    let doc = toml::Document::from_str(&contents)?;
    Ok(doc)
}

/// Write a Cargo manifest.
pub fn write_manifest(dir: &Path, doc: &toml::Document) -> Result<()> {
    let path = dir.join("Cargo.toml");
    fs::write(path, doc.to_string())?;
    Ok(())
}
