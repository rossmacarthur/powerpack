//! Construct a [`Cache`] in your workflow, providing any necessary
//! configuration.
//!
//! ```no_run
//! # use std::time::Duration;
//! # use powerpack_cache::Cache;
//! let cache = Cache::builder()
//!     .bundle_id("com.example.bundle")
//!     .ttl(Duration::from_secs(60))
//!     .build()
//!     .unwrap();
//! ```
//!
//! Then the only function to call is [`.load(..)`][Cache::load] which will
//! fetch the cached value and/or detach a process to update it.
//! ```no_run
//! # let mut cache = powerpack_cache::Cache::builder().build().unwrap();
//! let expensive_fn = || {
//!     // ...
//! #   Ok::<String, anyhow::Error>(String::from(""))
//! };
//!
//! let data = cache.load("key", "checksum", expensive_fn).unwrap();
//! ```

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json as json;
use thiserror::Error;

use powerpack_detach as detach;
use powerpack_env as env;

/// Raised when the cache is not populated within the poll duration.
#[derive(Debug, Clone, Error)]
#[non_exhaustive]
#[error("timeout waiting for cached data")]
pub struct TimeoutError {}

/// A builder for a cache.
///
/// Constructed using [`Cache::builder`].
#[derive(Debug, Clone)]
pub struct Builder {
    directory: Option<PathBuf>,
    bundle_id: Option<String>,
    ttl: Option<Duration>,
    poll_interval: Option<Duration>,
    poll_duration: Option<Duration>,
}

/// Manage a cache of data.
#[derive(Debug)]
pub struct Cache {
    directory: PathBuf,
    ttl: Duration,
    poll_interval: Duration,
    poll_duration: Duration,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Data<'a> {
    modified: SystemTime,
    checksum: &'a str,
    data: String,
}

impl Builder {
    /// Set the cache directory.
    pub fn directory(mut self, directory: impl Into<PathBuf>) -> Self {
        self.directory = Some(directory.into());
        self
    }

    /// Set the bundle id.
    pub fn bundle_id(mut self, bundle_id: impl Into<String>) -> Self {
        self.bundle_id = Some(bundle_id.into());
        self
    }

    /// Set the interval at which the cache will be updated.
    pub fn poll_interval(mut self, poll_interval: Duration) -> Self {
        self.poll_interval = Some(poll_interval);
        self
    }

    /// Set the duration to wait for the cache to be populated.
    pub fn poll_duration(mut self, poll_duration: Duration) -> Self {
        self.poll_duration = Some(poll_duration);
        self
    }

    /// Set the Time To Live (TTL) for the data in the cache.
    ///
    /// If the data in the cache is older than this then the cache will be
    /// automatically refreshed.
    pub fn ttl(mut self, tll: Duration) -> Self {
        self.ttl = Some(tll);
        self
    }

    /// Build the cache.
    pub fn build(self) -> Result<Cache> {
        let Self {
            directory,
            bundle_id,
            ttl,
            poll_interval,
            poll_duration,
        } = self;

        let directory = match directory {
            Some(d) => d,
            None => match env::workflow_cache() {
                Some(d) => d,
                None => {
                    let bundle_id = env::workflow_bundle_id()
                        .or(bundle_id)
                        .ok_or_else(|| anyhow!("no bundle id set"))?;
                    home::home_dir()
                        .ok_or_else(|| anyhow!("failed to find current user's home directory"))?
                        .join("Library/Caches/com.runningwithcrayons.Alfred/Workflow Data")
                        .join(bundle_id)
                }
            },
        };
        let ttl = ttl.unwrap_or_else(|| Duration::from_secs(30));
        let poll_interval = poll_interval.unwrap_or_else(|| Duration::from_millis(100));
        let poll_duration = poll_duration.unwrap_or_else(|| Duration::from_secs(1));

        Ok(Cache {
            directory,
            ttl,
            poll_interval,
            poll_duration,
        })
    }
}

impl Cache {
    /// Returns a new cache builder.
    pub fn builder() -> Builder {
        Builder {
            directory: None,
            bundle_id: None,
            ttl: None,
            poll_interval: None,
            poll_duration: None,
        }
    }

    /// Fetches the cache value and/or detaches a process to update it.
    pub fn load<F>(&mut self, key: &str, checksum: &str, f: F) -> Result<String>
    where
        F: FnOnce() -> Result<String>,
    {
        let directory = self.directory.join(key);
        let path = directory.join("data.json");

        let update_cache = || match update(&directory, &path, checksum, f) {
            Ok(true) => log::info!("fetched {} and updated cache", path.display()),
            Ok(false) => log::info!("another process updated cache for {}", path.display()),
            Err(err) => log::error!("{:#}", err),
        };

        match fs::read(&path) {
            Ok(data) => {
                let curr: Data = json::from_slice(&data)?;
                let needs_update = curr.checksum != checksum || {
                    let now = SystemTime::now();
                    now.duration_since(curr.modified)? > self.ttl
                };
                if needs_update {
                    detach::spawn(update_cache)?;
                }
                Ok(curr.data)
            }

            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                detach::spawn(update_cache)?;
                // wait for the cache to be populated
                let start = Instant::now();
                while Instant::now().duration_since(start) < self.poll_duration {
                    thread::sleep(self.poll_interval);
                    if let Ok(data) = fs::read(&path) {
                        let curr: Data = json::from_slice(&data)?;
                        return Ok(curr.data);
                    }
                }
                Err(TimeoutError {}.into())
            }

            Err(err) => Err(err.into()),
        }
    }
}

fn update<F>(directory: &Path, path: &Path, checksum: &str, f: F) -> Result<bool>
where
    F: FnOnce() -> Result<String>,
{
    let tmp = path.with_extension("tmp");
    if let Some(_guard) = fmutex::try_lock(directory)? {
        let data = f()?;
        fs::create_dir_all(path.parent().unwrap())?;
        let file = fs::File::create(&tmp)?;
        let modified = SystemTime::now();
        json::to_writer(
            &file,
            &Data {
                checksum,
                modified,
                data,
            },
        )?;
        fs::rename(tmp, path)?;
        Ok(true)
    } else {
        Ok(false)
    }
}
