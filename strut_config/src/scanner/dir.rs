use crate::ConfigEntry;
use std::path::PathBuf;
use strut_core::AppProfile;

/// Represents a single config-related directory.
///
/// Unlike with [`ConfigFile`]s, the path validity is not checked on
/// instantiation. But given that the filesystem is completely external to the
/// application, the existence or validity of any given path cannot be assumed
/// anyway.
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum ConfigDir {
    /// A directory that is not associated with an [`AppProfile`].
    Generic(PathBuf),

    /// A directory that is associated with an [`AppProfile`] of the given name.
    Specific {
        /// The path to the directory.
        path: PathBuf,

        /// The associated profile name.
        profile: String,
    },
}

impl ConfigDir {
    /// Creates a [`ConfigDir`] from the given [`PathBuf`], if the path points
    /// to a workable directory.
    pub fn at(path: PathBuf) -> Self {
        Self::make_with_profile(path, None)
    }

    /// Creates a [`ConfigDir`] from the given [`PathBuf`], if the path points
    /// to a workable directory, optionally applying the given known profile
    /// name.
    pub fn make_with_profile(path: PathBuf, known_profile: Option<&str>) -> Self {
        // If a profile is known, assign it
        if let Some(known_profile) = known_profile {
            return Self::Specific {
                path,
                profile: known_profile.to_string(),
            };
        }

        // Otherwise, just make a generic directory
        Self::Generic(path)
    }

    /// Creates a [`ConfigDir`] from the given [`PathBuf`], attempting to
    /// capture its name as a profile name. If the name cannot be captured
    /// (e.g., if the path doesn’t exist), the [generic](ConfigDir::Generic)
    /// variant is returned.
    pub fn make_capturing_profile(path: PathBuf) -> Self {
        // Read file name
        match path.file_name().and_then(std::ffi::OsStr::to_str) {
            Some(name) => {
                let profile = name.to_string();
                Self::Specific { path, profile }
            }
            None => Self::Generic(path),
        }
    }
}

impl ConfigDir {
    /// Reports the name of this directory, if it is readable.
    pub fn name(&self) -> Option<&str> {
        self.path().file_name().and_then(std::ffi::OsStr::to_str)
    }

    /// Reports whether this [`ConfigDir`] is applicable regardless of the
    /// [active](AppProfile::active) [`AppProfile`].
    pub fn is_generic(&self) -> bool {
        match *self {
            Self::Generic(_) => true,
            Self::Specific { .. } => false,
        }
    }

    /// Reports whether this [`ConfigDir`] is applicable only to a particular
    /// [`AppProfile`].
    pub fn is_specific(&self) -> bool {
        !self.is_generic()
    }

    /// Returns a reference to the internally held [`PathBuf`].
    pub fn path(&self) -> &PathBuf {
        match *self {
            Self::Generic(ref path) => path,
            Self::Specific { ref path, .. } => path,
        }
    }

    /// Returns a reference to the internally held profile name (if this
    /// variant is [specific](ConfigDir::is_specific)).
    pub fn profile(&self) -> Option<&str> {
        match *self {
            Self::Generic(_) => None,
            Self::Specific { ref profile, .. } => Some(profile),
        }
    }

    /// Reports whether this [`ConfigDir`] [applies](ConfigDir::applies_to) to
    /// the [active](AppProfile::active) [`AppProfile`].
    pub fn applies_to_active_profile(&self) -> bool {
        self.applies_to(AppProfile::active())
    }

    /// Reports whether this [`ConfigDir`] applies to the given [`AppProfile`].
    ///
    /// A generic config file (without a profile name in its file name) applies
    /// to any profile by default. A specific config file (with a profile name
    /// in its file name) applies to the given profile if the profile name
    /// matches.
    pub fn applies_to(&self, profile: impl AsRef<AppProfile>) -> bool {
        let given_profile = profile.as_ref();

        match *self {
            Self::Generic(_) => true,
            Self::Specific { ref profile, .. } => given_profile.is(profile),
        }
    }

    /// Expands this directory into a vector of nested [`ConfigEntry`]s.
    pub fn expand(&self, profile: Option<&str>) -> Vec<ConfigEntry> {
        std::fs::read_dir(self.path())
            .into_iter()
            .flat_map(|read_dir| {
                read_dir
                    .filter_map(Result::ok)
                    .map(|entry| entry.path())
                    .filter_map(|path| ConfigEntry::try_from_with_profile(path, profile))
            })
            .collect::<Vec<_>>()
    }
}

impl From<ConfigDir> for PathBuf {
    fn from(file: ConfigDir) -> Self {
        match file {
            ConfigDir::Generic(path) => path,
            ConfigDir::Specific { path, .. } => path,
        }
    }
}
