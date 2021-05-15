//! Environment variables in workflows.
//!
//! See <https://www.alfredapp.com/help/workflows/script-environment-variables/>

use std::env;
use std::ffi::OsStr;
use std::path::PathBuf;

/// Fetches the environment variable `key` from the current process.
///
/// This function is similar to [`env::var(key).ok()`][env::var] but it also
/// maps an empty string to `None`.
///
/// # None
///
/// Returns `None` in the following cases:
/// - if the environment variable is not present.
/// - if the environment variable is not valid Unicode.
/// - if the environment variable is set to an empty string.
///
/// # Panics
///
/// This function may panic if key is empty, contains an ASCII equals sign `'='`
/// or the NUL character `'\0'`, or when the value contains the NUL character.
pub fn var<K: AsRef<OsStr>>(key: K) -> Option<String> {
    env::var(key).ok().filter(|s| !s.is_empty())
}

/// Whether or not the user currently has the debug panel open.
pub fn is_debug() -> bool {
    var("alfred_debug").as_deref() == Some("1")
}

/// The location of the `Alfred.alfredpreferences` directory.
///
/// If a user has synced their settings, this will allow you to find out where
/// their settings are.
pub fn preferences() -> Option<PathBuf> {
    var("alfred_preferences").map(PathBuf::from)
}

/// The Alfred version that is currently running.
///
/// This may be useful if your workflow depends on particular Alfred features.
pub fn version() -> Option<String> {
    var("alfred_version")
}

/// The Alfred build version that is currently running.
///
/// This may be useful if your workflow depends on particular Alfred features.
pub fn version_build() -> Option<u32> {
    var("alfred_version_build").and_then(|s| s.parse().ok())
}

/// The bundle ID of the currently running workflow.
pub fn workflow_bundle_id() -> Option<String> {
    var("alfred_workflow_bundleid")
}

/// The name of the currently running workflow.
pub fn workflow_name() -> Option<String> {
    var("alfred_workflow_name")
}

/// The unique ID of the currently running workflow.
pub fn workflow_uid() -> Option<String> {
    var("alfred_workflow_uid")
}

/// The version of the currently running workflow.
pub fn workflow_version() -> Option<String> {
    var("alfred_workflow_version")
}

/// The recommended directory for volatile workflow data.
///
/// This will only be populated if your workflow has a bundle id set.
pub fn workflow_cache() -> Option<PathBuf> {
    var("alfred_workflow_cache").map(PathBuf::from)
}

/// The recommended directory for non-volatile workflow data.
///
/// This will only be populated if your workflow has a bundle id set.
pub fn workflow_data() -> Option<PathBuf> {
    var("alfred_workflow_data").map(PathBuf::from)
}
