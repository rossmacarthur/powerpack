//! ⚡ Cache management for your Alfred workflow
//!
//! This crate provides a simple cache management system for your Alfred
//! workflow. Data is cached in the workflow's cache directory and is updated
//! asynchronously.
//!
//! The cache supports arbitrary data types for each key as long as they can be
//! serialized and deserialized from JSON.
//!
//! # Concepts
//!
//! - `key`: a unique identifier for a piece of data stored in the cache.
//!
//! - `ttl`: the Time To Live (TTL) for the data in the cache. If the data in
//!   the cache is older than this then it is considered "expired".
//!
//! - `checksum`: an optional checksum for a particular cache `key`. You can use
//!   this to bust the cache for some other reason than the data being expired.
//!
//! - `update_fn`: a function that is called to update the cache for a `key`.
//!   This is typically some operation that is expensive and/or slow and you do
//!   not want to block the Alfred workflow. This function is called
//!   asynchronously to update the cache. If the cache is already being updated
//!   by another process, then the function is not called.
//!
//! The following behaviour is determined by the [policy](QueryPolicy) of the
//! query:
//! - When to call a provided `update_fn`.
//! - When to return bad, expired, or checksum mismatched data.
//!
//! # Usage
//!
//! Use a [`Builder`] to construct a new [`Cache`]`.
//!
//! ```no_run
//! # mod powerpack { pub extern crate powerpack_cache as cache; } // mock re-export
//! use std::time::Duration;
//! use powerpack::cache;
//!
//! let cache = cache::Builder::new().ttl(Duration::from_secs(60 * 60)).build();
//! ```
//!
//! Then the only function to call is [`.query(..)`][Cache::query] which will
//! fetch the cached value and/or detach a process to update it.
//! ```no_run
//! # use powerpack_cache as cache;
//! # let mut cache = cache::Builder::new().build();
//! #
//! let expensive_fn = || {
//!     // perform some expensive operation, like fetching
//!     // something over the internet
//! #   Ok::<String, std::convert::Infallible>(String::from(""))
//! };
//!
//! let q = cache::Query::new("unique_key").update_fn(expensive_fn);
//! let data = cache.query(q)?;
//! # Ok::<(), cache::QueryError>(())
//! ```
//!

mod query;

use std::error::Error as StdError;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use flagset::FlagSet;
use serde::{Deserialize, Serialize};
use serde_json as json;
use thiserror::Error;

use powerpack_detach as detach;
use powerpack_env as env;

pub use crate::query::{Query, QueryError, QueryPolicy};

/// The cache file name, the version indicates the format of the data
const DATA: &str = "v1.json";

/// Raised when constructing a new cache.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum BuildError {
    /// Raised when the home directory cannot be determined
    #[error("home directory not found")]
    NoHomeDir,
}

/// Raised when updating data in the cache
#[derive(Debug, Error)]
#[non_exhaustive]
enum UpdateError {
    /// Raised when an I/O error occurs
    #[error("io error")]
    Io(#[from] io::Error),

    /// Raised when a JSON serialization error occurs
    #[error("serialization error")]
    Serialize(#[from] json::Error),

    /// Raised when an error
    #[error("update fn failed: {0}")]
    UpdateFn(#[from] Box<dyn StdError + Send + Sync + 'static>),
}

/// A builder for a cache.
#[derive(Debug, Clone)]
pub struct Builder {
    directory: Option<PathBuf>,
    query_policy: FlagSet<QueryPolicy>,
    ttl: Duration,
    initial_poll: Option<Duration>,
}

/// Manage a cache of data on disk.
#[derive(Debug)]
pub struct Cache {
    directory: PathBuf,
    query_policy: FlagSet<QueryPolicy>,
    ttl: Duration,
    initial_poll: Option<Duration>,
}

/// The data stored in the cache.
///
/// Breaking changes need to bump the version in the cache file name.
#[derive(Debug, Clone, Deserialize, Serialize)]
struct CacheData<'a, T> {
    modified: SystemTime,
    checksum: Option<&'a str>,
    data: T,
}

impl Default for Builder {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    /// Returns a new cache builder.
    #[inline]
    pub fn new() -> Self {
        Builder {
            directory: None,
            query_policy: QueryPolicy::default_set(),
            ttl: Duration::from_secs(60),
            initial_poll: None,
        }
    }

    /// Set the cache directory.
    ///
    /// Defaults to `{alfred_workflow_cache}/cache`
    ///
    /// These should be set by Alfred, but if not:
    /// - `{alfred_workflow_cache}` defaults to `~/Library/Caches/com.runningwithcrayons.Alfred/Workflow Data/{alfred_workflow_bundleid}`
    /// - `{alfred_workflow_bundleid}` defaults to `powerpack`
    ///
    /// See [`powerpack_env::workflow_cache_or_default`] for more information.
    #[inline]
    pub fn directory(mut self, directory: impl Into<PathBuf>) -> Self {
        self.directory = Some(directory.into());
        self
    }

    /// Set the query policy for the cache.
    ///
    /// This is used to determine things like when updates should occur and
    /// stale data is allowed to be returned.
    pub fn policy(mut self, query_policy: impl Into<FlagSet<QueryPolicy>>) -> Self {
        self.query_policy = query_policy.into();
        self
    }

    /// Set the default Time To Live (TTL) for the data in the cache.
    ///
    /// This is used if the query does not specify a TTL.
    ///
    /// If the data in the cache is older than this then the cache will be
    /// automatically refreshed. Stale data will be returned in the meantime.
    ///
    /// Defaults to 60 seconds.
    #[inline]
    pub fn ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }

    /// Set the initial poll duration.
    ///
    /// This is used if the query does not specify an initial poll duration.
    ///
    /// This is the duration to wait for the cache to be populated on the first
    /// call. If the cache is not populated within this duration, a miss error
    /// will be raised.
    ///
    /// Defaults to not polling at all. This means the initial call to
    /// [`.query()`](Cache::query) will return immediately with
    /// [`Err(QueryError::Miss)`][QueryError::Miss].
    #[inline]
    pub fn initial_poll(mut self, initial_poll: Duration) -> Self {
        self.initial_poll = Some(initial_poll);
        self
    }

    /// Try build the cache.
    ///
    /// This can fail if the user's home directory cannot be determined.
    pub fn try_build(self) -> Result<Cache, BuildError> {
        let Self {
            directory,
            query_policy,
            ttl,
            initial_poll,
        } = self;

        let directory = match directory {
            Some(directory) => directory,
            None => env::try_workflow_cache_or_default()
                .ok_or(BuildError::NoHomeDir)?
                .join("cache"),
        };

        Ok(Cache {
            directory,
            query_policy,
            ttl,
            initial_poll,
        })
    }

    /// Build the cache.
    ///
    /// # Panics
    ///
    /// If the user's home directory cannot be determined.
    #[track_caller]
    #[inline]
    pub fn build(self) -> Cache {
        self.try_build().expect("failed to build cache")
    }
}

struct CacheDataHolder<'a, T> {
    result: Result<CacheData<'a, T>, json::Error>,
    is_bad_data: bool,
    is_checksum_mismatch: bool,
    is_expired: bool,
}

impl<'a, T> CacheDataHolder<'a, T> {
    fn build(data: &'a [u8], checksum: Option<&str>, ttl: Duration) -> Self
    where
        T: for<'de> Deserialize<'de>,
    {
        let result: Result<CacheData<T>, _> = json::from_slice(data);
        match &result {
            Ok(d) => {
                let is_checksum_mismatch = checksum.is_some() && d.checksum != checksum;
                let is_expired = d.modified.elapsed().map_or(true, |d| d > ttl);
                Self {
                    result,
                    is_bad_data: false,
                    is_checksum_mismatch,
                    is_expired,
                }
            }
            Err(_) => Self {
                result,
                is_bad_data: true,
                is_checksum_mismatch: false,
                is_expired: false,
            },
        }
    }

    fn should_update(&self, policy: FlagSet<QueryPolicy>) -> bool {
        policy.contains(QueryPolicy::UpdateAlways)
            || self.is_bad_data && policy.contains(QueryPolicy::UpdateBadData)
            || self.is_checksum_mismatch && policy.contains(QueryPolicy::UpdateChecksumMismatch)
            || self.is_expired && policy.contains(QueryPolicy::UpdateExpired)
    }

    #[rustfmt::skip]
    fn should_return(&self, policy: FlagSet<QueryPolicy>) -> bool {
        policy.contains(QueryPolicy::ReturnAlways) || {
            (!self.is_bad_data || policy.contains(QueryPolicy::ReturnBadDataErr))
            && (!self.is_checksum_mismatch || policy.contains(QueryPolicy::ReturnChecksumMismatch))
            && (!self.is_expired || policy.contains(QueryPolicy::ReturnExpired))
        }
    }

    fn into_result(self, policy: FlagSet<QueryPolicy>) -> Result<T, QueryError> {
        if self.should_return(policy) {
            Ok(self.result.map(|c| c.data)?)
        } else {
            Err(QueryError::Miss)
        }
    }
}

impl Cache {
    /// Fetches the cache value according to the [`Query`].
    pub fn query<'a, T, E>(&self, query: Query<'a, T, E>) -> Result<T, QueryError>
    where
        T: Serialize + for<'de> Deserialize<'de>,
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        let Query {
            key,
            checksum,
            policy,
            ttl,
            initial_poll,
            update_fn,
            ..
        } = query;

        let directory = self.directory.join(key);
        let path = directory.join(DATA);

        let checksum = checksum.as_deref();
        let policy = policy.unwrap_or(self.query_policy);
        let ttl = ttl.unwrap_or(self.ttl);
        let initial_poll = initial_poll.or(self.initial_poll).map(|d| {
            let sleep = (d / 5).min(Duration::from_millis(100)).min(d);
            (d, sleep)
        });

        let update_cache = update_fn.map(|f| {
            || match update(&directory, &path, checksum, f) {
                Ok(true) => log::info!("cache: updated {key}"),
                Ok(false) => log::debug!("cache: another process updated {key}"),
                Err(err) => log::error!(
                    "cache: failed to update {key}: {}",
                    detach::format_err(&err)
                ),
            }
        });

        match fs::read(&path) {
            Ok(data) => {
                let data = CacheDataHolder::build(&data, checksum, ttl);
                if let Some(update_cache) = update_cache
                    && data.should_update(policy)
                {
                    detach::spawn(update_cache)?;
                }
                data.into_result(policy)
            }

            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                if let Some(update_cache) = update_cache {
                    detach::spawn(update_cache)?;
                }

                // wait for the cache to be populated
                if let Some((poll_duration, poll_sleep)) = initial_poll {
                    let start = Instant::now();
                    while Instant::now().duration_since(start) < poll_duration {
                        thread::sleep(poll_sleep);
                        match fs::read(&path) {
                            Ok(data) => {
                                let data = CacheDataHolder::build(&data, checksum, ttl);
                                return data.into_result(policy);
                            }
                            Err(err) if err.kind() == io::ErrorKind::NotFound => continue,
                            Err(err) => return Err(err.into()),
                        }
                    }
                }

                Err(QueryError::Miss)
            }

            Err(err) => Err(err.into()),
        }
    }
}

fn update<'a, T, E>(
    directory: &Path,
    path: &Path,
    checksum: Option<&str>,
    f: Box<dyn FnOnce() -> Result<T, E> + 'a>,
) -> Result<bool, UpdateError>
where
    T: Serialize + for<'de> Deserialize<'de>,
    E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    fs::create_dir_all(directory)?;
    let tmp = path.with_extension("tmp");
    match fmutex::try_lock(directory)? {
        Some(_guard) => {
            let data = f().map_err(Into::into)?;
            let file = fs::File::create(&tmp)?;
            let modified = SystemTime::now();
            json::to_writer(
                &file,
                &CacheData {
                    checksum,
                    modified,
                    data,
                },
            )?;
            fs::rename(tmp, path)?;
            Ok(true)
        }
        None => Ok(false),
    }
}
