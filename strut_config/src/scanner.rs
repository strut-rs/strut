use crate::{ConfigEntry, ConfigFile};
use std::env;
use std::path::PathBuf;
use strut_core::Pivot;

pub mod dir;
pub mod entry;
pub mod file;

/// A small facade for finding the [`ConfigFile`]s relevant for the current
/// binary crate.
pub struct Scanner;

impl Scanner {
    /// Discovers and returns all [`ConfigFile`]s relevant to the current binary
    /// crate and the [active](AppProfile::active) [`AppProfile`].
    ///
    /// The returned vector is **ordered for precedence**: later files in the
    /// list should override earlier files when keys overlap.
    ///
    /// ## Search Location
    ///
    /// Scans for configuration files within the resolved
    /// [configuration directory](Self::resolve_config_dir).
    ///
    /// ## Supported Formats
    ///
    /// The following file formats are recognized:
    /// - TOML (`.toml`)
    /// - YAML (`.yml`, `.yaml`)
    ///
    /// ## File Types
    ///
    /// - **Generic config files**: Apply to all profiles.
    ///   - Pattern: `config/{any_name}.{ext}`
    /// - **Profile-specific config files**: Apply only if the file’s profile
    ///   matches the [active](AppProfile::active) [`AppProfile`].
    ///   - Patterns:
    ///     - `config/{any_name}.{profile}.{ext}`
    ///     - `config/{profile}/{any_name}.{ext}`
    ///
    /// ## Ordering
    ///
    /// - Generic files always precede profile-specific files.
    /// - Within each group (generic and profile-specific), files are ordered
    ///   lexicographically by full path.
    ///
    /// ## Notes
    ///
    /// - File and directory names are matched case-insensitively.
    ///
    /// ## Returns
    ///
    /// An ordered `Vec<ConfigFile>` containing all discovered configuration
    /// files.
    pub fn find_config_files(dir_name: Option<&str>) -> Vec<ConfigFile> {
        // Resolve the config directory
        let config_dir = Self::resolve_config_dir(dir_name);

        // Resolve the config files
        let mut config_files = ConfigEntry::dir(config_dir) // start with config dir
            .cd() // dive one level in
            .flat_map(ConfigEntry::cd_capturing_profile) // dive another level in, capturing profile name from directory name
            .filter(ConfigEntry::applies_to_active_profile) // keep everything associated with active profile
            .filter_map(ConfigEntry::to_config_file) // keep only config files (discard any further nested directories)
            .collect::<Vec<_>>(); // collect into a vector

        // Sort logically in place
        config_files.sort();

        config_files
    }

    /// Resolves the application’s **configuration directory**: where the
    /// framework looks for configuration files.
    ///
    /// Dynamically determines the path at runtime.
    ///
    /// Resolution order:
    /// 1. If the `APP_CONFIG_DIR` environment variable is set, its value is
    ///    used.
    /// 2. Otherwise, if a non-empty `path` argument is provided, it is used.
    /// 3. Otherwise, defaults to a directory named `"config"`.
    ///
    /// If the resolved path is relative, it is interpreted relative to the
    /// [pivot directory](Self::resolve_pivot_dir). Returns an absolute
    /// [`PathBuf`] of the configuration directory.
    fn resolve_config_dir(path: Option<&str>) -> PathBuf {
        let input_path = env::var("APP_CONFIG_DIR") // environment takes highest priority
            .map(PathBuf::from)
            .ok()
            .or_else(|| {
                path.map(str::trim) // if no environment, then argument
                    .filter(|s| !s.is_empty())
                    .map(PathBuf::from)
            })
            .unwrap_or(PathBuf::from("config")); // if no argument, then global default

        if input_path.is_absolute() {
            input_path
        } else {
            Pivot::resolve().join(input_path)
        }
    }
}
