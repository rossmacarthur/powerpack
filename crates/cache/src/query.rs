use std::convert::Infallible;
use std::fmt::Write as _;
use std::io;
use std::time::Duration;

use flagset::{FlagSet, flags};
use serde_json as json;
use thiserror::Error;

/// Raised when accessing data in the cache.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum QueryError {
    /// Raised when there is a cache miss.
    #[error("cache miss")]
    Miss,

    /// Raised when an I/O error occurs.
    ///
    /// This can occur when reading the cache file.
    #[error("io error")]
    Io(#[from] io::Error),

    /// Raised when JSON deserialization occurs.
    ///
    /// Data is stored in the cache as JSON, so this error is raised when
    /// deserializing the data fails.
    ///
    /// Since the caller provides the type that is stored in the cache, this
    /// will typically occur if the type changes.
    #[error("deserialization error")]
    BadData(#[from] json::Error),
}

flags! {
    /// The policy for querying the cache.
    ///
    /// Holds various toggles for when to update the cache and when to return
    /// stale data. This type follows a bitflag pattern, so multiple flags can
    /// be combined using the `|` operator.
    ///
    /// The default policy is [`QueryPolicy::default_set()`], which is
    /// equivalent to the following example.
    ///
    /// # Examples
    ///
    /// ```
    /// # use powerpack_cache::{Query, QueryPolicy};
    /// let q = Query::new("unique_key").policy(
    ///     QueryPolicy::UpdateBadData
    ///     | QueryPolicy::UpdateChecksumMismatch
    ///     | QueryPolicy::UpdateExpired
    ///     | QueryPolicy::ReturnExpired
    /// );
    /// # let q: Query<'_, ()> = q;
    /// ```
    pub enum QueryPolicy: u16 {
        /// Always update the cache when a [`Query::update_fn`] is provided.
        ///
        /// This option overrides the other `Update...`flags and will always
        /// update the cache. If not set then the cache will only be updated if
        /// the data is bad or stale. Outlined in the below flags.
        ///
        /// Generally this should not be set because:
        /// - It defeats the purpose of a TTL
        /// - Alfred can spawn many instances of a process in a short period of
        ///   time, e.g. one for each character typed, this could result in many
        ///   unnecessary updates (depending on what type of data you're
        ///   storing).
        UpdateAlways,

        /// Update the cache if the data is bad (fails to deserialize).
        ///
        /// Generally this should be set because if the data is bad then you
        /// want to correct it in the cache.
        UpdateBadData,

        /// Update the cache if the checksum is different.
        ///
        /// Generally this should be set because the checksum is used to
        /// determine if the data is still applicable.
        UpdateChecksumMismatch,

        /// Update the cache if the data is expired.
        UpdateExpired,

        /// Always return data if it is available.
        ///
        /// This option is forward compatible with any other flags that may be
        /// added in the future for returning data. Right now it is equivalent
        /// to `ReturnBadDataErr | ReturnChecksumMismatch | ReturnExpired`.
        ///
        /// Generally this should not be set.
        ReturnAlways,

        /// Return the error if the data is bad, [`QueryError::BadData`] which
        /// contains the deserialization error in the source.
        ///
        /// If not set then [`QueryError::Miss`] will be returned.
        ///
        /// Whether this should be set depends on whether your code is planning
        /// on handling the error.
        ReturnBadDataErr,

        /// Return data even if the checksum is different.
        ///
        /// If not set then [`QueryError::Miss`] will be returned.
        ///
        /// Generally this should not be set because the checksum is used to
        /// determine if the data is still applicable.
        ReturnChecksumMismatch,

        /// Return data if it is expired.
        ///
        /// If not set then [`QueryError::Miss`] will be returned.
        ///
        /// Generally this should be set because for Alfred workflows it is
        /// desirable to return *something* even if the data is expired.
        ReturnExpired,
    }
}

impl QueryPolicy {
    /// Returns the default policy.
    ///
    /// The default enables the following flags only.
    /// - [`QueryPolicy::UpdateBadData`]
    /// - [`QueryPolicy::UpdateChecksumMismatch`]
    /// - [`QueryPolicy::UpdateExpired`]
    /// - [`QueryPolicy::ReturnExpired`]
    pub fn default_set() -> FlagSet<Self> {
        QueryPolicy::UpdateBadData
            | QueryPolicy::UpdateChecksumMismatch
            | QueryPolicy::UpdateExpired
            | QueryPolicy::ReturnExpired
    }
}

/// Query the cache for data.
///
/// A query must be constructed, using the builder pattern and then passed to
/// [`Cache::query`](crate::Cache::query) to retrieve the data.
///
/// The following fields are required when constructing a query:
///
/// - `key`: passed to [`Query::new`], this is a unique identifier for the
///   data in the cache, and is used to determine the name of the cache file.
///
/// - type `T`: the type of the data stored in the cache, it must implement
///   [`serde::Serialize`] and [`serde::Deserialize`].
///
/// The following fields are optional:
///
/// - `update_fn`: used to update the cache, see [`Query::update_fn`].
/// - `checksum`: used to determine staleness of the cache, see
///   [`Query::checksum`].
/// - `policy`: used to determine when to update the cache and when to return
///   stale data, see [`Query::policy`].
/// - `ttl`: the Time To Live (TTL) for the data in the cache, see
///   [`Query::ttl`].
/// - `initial_poll`: the duration to wait for the cache to be populated on the
///   first call, see [`Query::initial_poll`].
///
pub struct Query<'a, T, E = Infallible> {
    pub(crate) key: &'a str,
    pub(crate) update_fn: Option<Box<dyn FnOnce() -> Result<T, E> + 'a>>,
    pub(crate) policy: Option<FlagSet<QueryPolicy>>,
    pub(crate) checksum: Option<String>,
    pub(crate) ttl: Option<Duration>,
    pub(crate) initial_poll: Option<Duration>,
}

impl<'a> Query<'a, (), Infallible> {
    /// Returns a new cache query.
    ///
    /// The key is used to determine the name of the cache file.
    #[inline]
    pub fn new(key: &'a str) -> Self {
        Query {
            key,
            update_fn: None,
            policy: None,
            checksum: None,
            ttl: None,
            initial_poll: None,
        }
    }

    /// Set the function to update the cache.
    ///
    /// This function is called if the cache needs to be updated.
    ///
    /// # 💡 Note
    ///
    /// The cache is updated in a separate process to avoid blocking the main
    /// thread, this means that any errors from the update function will not be
    /// propagated. Stale data will be returned in the meantime.
    #[inline]
    pub fn update_fn<F, T, E>(self, update_fn: F) -> Query<'a, T, E>
    where
        F: FnOnce() -> Result<T, E> + 'a,
    {
        Query {
            key: self.key,
            checksum: self.checksum,
            policy: self.policy,
            ttl: self.ttl,
            initial_poll: self.initial_poll,
            update_fn: Some(Box::new(update_fn)),
        }
    }
}

impl<T, E> Query<'_, T, E> {
    /// Set the checksum for the cache.
    ///
    /// This is used to determine staleness and is used in two places:
    ///
    /// - Whether to the cache needs to be updated (in addition to the TTL).
    ///   If the checksum is different to the one stored in the cache then the
    ///   cache might be updated, depending on the [`QueryPolicy`].
    ///
    /// - Whether to return stale data. If the checksum is different to the one
    ///   stored in the cache then depending on the [`QueryPolicy`] stale data
    ///   may be returned.
    ///
    #[inline]
    pub fn checksum<C>(mut self, checksum: C) -> Self
    where
        C: AsRef<[u8]>,
    {
        self.checksum = Some(to_hex(checksum.as_ref()));
        self
    }

    /// Set the policy for the cache query.
    ///
    /// This is used to determine when updates should occur and stale data is
    /// allowed to be returned.
    ///
    /// Defaults to the cache's policy.
    #[inline]
    pub fn policy(mut self, policy: impl Into<FlagSet<QueryPolicy>>) -> Self {
        self.policy = Some(policy.into());
        self
    }

    /// Set the Time To Live (TTL) for the data in the cache.
    ///
    /// If the data in the cache is older than this then the cache will be
    /// automatically refreshed. Stale data will be returned in the meantime.
    ///
    /// Defaults to the cache's TTL.
    #[inline]
    pub fn ttl(mut self, ttl: Duration) -> Self {
        self.ttl = Some(ttl);
        self
    }

    /// Set the initial poll duration.
    ///
    /// This is the duration to wait for the cache to be populated on the first
    /// call. If the cache is not populated within this duration, a miss error
    /// will be raised.
    ///
    /// Defaults to the cache's initial poll duration.
    #[inline]
    pub fn initial_poll(mut self, initial_poll: Duration) -> Self {
        self.initial_poll = Some(initial_poll);
        self
    }
}

fn to_hex(b: &[u8]) -> String {
    let mut s = String::with_capacity(b.len() * 2);
    for byte in b {
        write!(&mut s, "{:02x}", byte).unwrap();
    }
    s
}
