use config::{File, FileFormat, FileSourceFile};
use std::cmp::Ordering;
use std::path::PathBuf;
use strut_core::AppProfile;

/// Represents a single config file.
#[derive(Debug, Eq, PartialEq)]
pub enum ConfigFile {
    /// A file like `app.toml`.
    GenericToml(PathBuf),

    /// A file like `app.yaml` / `app.yml`.
    GenericYaml(PathBuf),

    /// A file like `app.prod.toml`.
    SpecificToml {
        /// The path to the file.
        path: PathBuf,

        /// The associated profile name.
        profile: String,
    },

    /// A file like `app.prod.yaml` / `app.prod.yml`.
    SpecificYaml {
        /// The path to the file.
        path: PathBuf,

        /// The associated profile name.
        profile: String,
    },
}

impl ConfigFile {
    /// Creates a [`ConfigFile`] from the given [`PathBuf`], if the path points
    /// to a workable config file.
    pub fn try_at(path: PathBuf) -> Option<Self> {
        Self::try_make_with_profile(path, None)
    }

    /// Creates a [`ConfigFile`] from the given [`PathBuf`], if the path points
    /// to a workable config file, optionally applying the given known profile
    /// name.
    pub fn try_make_with_profile(path: PathBuf, known_profile: Option<&str>) -> Option<Self> {
        // Read file name
        let name = match path.file_name().and_then(std::ffi::OsStr::to_str) {
            Some(name) => name,
            None => return None,
        };

        // Split file name on `.`
        let chunks = name.split('.').collect::<Vec<_>>();

        // Match chunk pattern
        match *chunks.as_slice() {
            [_name, extension] => {
                if is_toml_extension(extension) {
                    Self::toml_from(path, known_profile)
                } else if is_yaml_extension(extension) {
                    Self::yaml_from(path, known_profile)
                } else {
                    None
                }
            }
            [_name, profile, extension] => {
                // Do we know the profile already?
                if let Some(known_profile) = known_profile {
                    // If we know the profile already, only take specific file of that profile
                    if !known_profile.eq_ignore_ascii_case(profile) {
                        return None;
                    }
                }

                // Only take supported extensions
                if is_toml_extension(extension) {
                    let profile = profile.to_string();
                    Self::toml_from(path, Some(profile))
                } else if is_yaml_extension(extension) {
                    let profile = profile.to_string();
                    Self::yaml_from(path, Some(profile))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn toml_from(path: PathBuf, profile: Option<impl Into<String>>) -> Option<Self> {
        match profile {
            None => Some(ConfigFile::GenericToml(path)),
            Some(profile) => {
                let profile = profile.into();

                Some(ConfigFile::SpecificToml { path, profile })
            }
        }
    }

    fn yaml_from(path: PathBuf, profile: Option<impl Into<String>>) -> Option<Self> {
        match profile {
            None => Some(ConfigFile::GenericYaml(path)),
            Some(profile) => {
                let profile = profile.into();

                Some(ConfigFile::SpecificYaml { path, profile })
            }
        }
    }
}

impl ConfigFile {
    /// Reports whether this [`ConfigFile`] is applicable regardless of the
    /// [active](AppProfile::active) [`AppProfile`].
    pub fn is_generic(&self) -> bool {
        match *self {
            Self::GenericToml(_) | Self::GenericYaml(_) => true,
            Self::SpecificToml { .. } | Self::SpecificYaml { .. } => false,
        }
    }

    /// Reports whether this [`ConfigFile`] is applicable only to a particular
    /// [`AppProfile`].
    pub fn is_specific(&self) -> bool {
        !self.is_generic()
    }

    /// Returns a reference to the internally held [`PathBuf`].
    pub fn path(&self) -> &PathBuf {
        match *self {
            Self::GenericToml(ref path) => path,
            Self::GenericYaml(ref path) => path,
            Self::SpecificToml { ref path, .. } => path,
            Self::SpecificYaml { ref path, .. } => path,
        }
    }

    /// Returns a reference to the internally held profile name (if this
    /// variant is [specific](ConfigFile::is_specific)).
    pub fn profile(&self) -> Option<&str> {
        match *self {
            Self::GenericToml(_) => None,
            Self::GenericYaml(_) => None,
            Self::SpecificToml { ref profile, .. } => Some(profile),
            Self::SpecificYaml { ref profile, .. } => Some(profile),
        }
    }

    /// Reports whether this [`ConfigFile`] [applies](ConfigFile::applies_to) to
    /// the [active](AppProfile::active) [`AppProfile`].
    pub fn applies_to_active_profile(&self) -> bool {
        self.applies_to(AppProfile::active())
    }

    /// Reports whether this [`ConfigFile`] applies to the given [`AppProfile`].
    ///
    /// A generic config file (without a profile name in its file name) applies
    /// to any profile by default. A specific config file (with a profile name
    /// in its file name) applies to the given profile if the profile name
    /// matches.
    pub fn applies_to(&self, profile: impl AsRef<AppProfile>) -> bool {
        let given_profile = profile.as_ref();

        match *self {
            Self::GenericToml(_) | Self::GenericYaml(_) => true,
            Self::SpecificToml { ref profile, .. } | Self::SpecificYaml { ref profile, .. } => {
                given_profile.is(profile)
            }
        }
    }
}

impl PartialOrd for ConfigFile {
    /// Delegates to the [`Ord`] implementation.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ConfigFile {
    /// Implements the custom ordering rules for [`ConfigFile`].
    ///
    /// First rule is that generics always come before specifics (so that
    /// specifics would override generics). After that, both subgroups are
    /// ordered by their file path. For specific files, we order by the profile
    /// name before we order by the path.
    fn cmp(&self, other: &Self) -> Ordering {
        // Are the given two generic?
        let self_generic = self.is_generic();
        let other_generic = other.is_generic();

        // If we have a generic against specific, it’s an easy job
        match (self_generic, other_generic) {
            (true, false) => return Ordering::Less,
            (false, true) => return Ordering::Greater,
            _ => { /* either both generic or both specific */ }
        }

        // Extract path references
        let self_path = self.path();
        let other_path = other.path();

        // If both are generic, it’s also an easy job
        if self_generic {
            return self_path.cmp(other_path);
        }

        // Both are profile-specific: extract profile names
        let self_profile = self.profile();
        let other_profile = other.profile();

        // Unfortunately, profiles are optional, so check them first
        match (self_profile, other_profile) {
            // Both have profile name
            (Some(self_profile_name), Some(other_profile_name)) => {
                // Compare profile names first, then paths
                match self_profile_name.cmp(other_profile_name) {
                    Ordering::Equal => self_path.cmp(other_path),
                    non_eq => non_eq,
                }
            }
            // No profile names: just compare paths
            _ => self_path.cmp(other_path),
        }
    }
}

/// Reports whether the given string slice is a recognized YAML extension.
fn is_yaml_extension(ext: &str) -> bool {
    ext.eq_ignore_ascii_case("yml") || ext.eq_ignore_ascii_case("yaml")
}

/// Reports whether the given string slice is a recognized TOML extension.
fn is_toml_extension(ext: &str) -> bool {
    ext.eq_ignore_ascii_case("toml")
}

impl ConfigFile {
    /// Returns the corresponding [`FileFormat`].
    fn format(&self) -> FileFormat {
        match *self {
            Self::GenericToml(_) | Self::SpecificToml { .. } => FileFormat::Toml,
            Self::GenericYaml(_) | Self::SpecificYaml { .. } => FileFormat::Yaml,
        }
    }
}

impl From<ConfigFile> for PathBuf {
    fn from(file: ConfigFile) -> Self {
        match file {
            ConfigFile::GenericToml(path) => path,
            ConfigFile::GenericYaml(path) => path,
            ConfigFile::SpecificToml { path, .. } => path,
            ConfigFile::SpecificYaml { path, .. } => path,
        }
    }
}

impl From<ConfigFile> for File<FileSourceFile, FileFormat> {
    fn from(file: ConfigFile) -> Self {
        let format = file.format();
        let path = PathBuf::from(file);

        File::from(path).format(format)
    }
}
