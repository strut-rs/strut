use std::env;
use std::path::PathBuf;

/// Owns the logic for [resolving](Pivot::resolve) the runtime pivot directory.
///
/// ## Pivot directory
///
/// Pivot directory is the current working directory of the runtime process,
/// **unless** running using Cargo (e.g.: `cargo run`, `cargo test`, via IDE,
/// etc.): then it’s the directory containing `Cargo.toml`.
pub struct Pivot;

impl Pivot {
    /// Resolves the **pivot directory** at runtime, which is the directory
    /// relative to which the file queries are normally performed.
    ///
    /// Example: the config directory, when given as a relative path, is
    /// resolved relative to the pivot directory.
    ///
    /// When running a binary in a development environment (e.g.: using
    /// `cargo run`, `cargo test`, or from an IDE), the pivot directory is the
    /// directory containing the crate’s `Cargo.toml` file. This is detected by
    /// reading the `CARGO_MANIFEST_DIR` environment variable at runtime.
    ///
    /// When running a binary without using `cargo` (e.g., by executing the
    /// binary in production), the `CARGO_MANIFEST_DIR` environment variable is
    /// normally not set, and the pivot directory is the process’s
    /// [current](env::current_dir) working directory.
    pub fn resolve() -> PathBuf {
        env::var("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| Self::require_current_dir())
    }

    /// Returns the current working directory at runtime, or panics if it is not
    /// accessible (e.g., it doesn’t exist or the user has insufficient
    /// permissions).
    fn require_current_dir() -> PathBuf {
        env::current_dir().expect(concat!(
            "it should be possible to access the current working directory",
            " at runtime"
        ))
    }
}
