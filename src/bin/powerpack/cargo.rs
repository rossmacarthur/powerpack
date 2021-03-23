use std::ffi::OsStr;
use std::process;

use anyhow::{bail, Result};

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
    pub fn run(&mut self) -> Result<()> {
        if !self.cmd.status()?.success() {
            bail!("`cargo` did not exit successfully");
        }
        Ok(())
    }
}

/// Run a `cargo new` command.
#[must_use]
pub fn new<P: AsRef<OsStr>>(path: P) -> Cargo {
    Cargo::new("new").arg("--bin").arg(path)
}

/// Run a `cargo init` command.
#[must_use]
pub fn init() -> Cargo {
    Cargo::new("init").arg("--bin")
}
