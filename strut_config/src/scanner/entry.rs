use crate::scanner::dir::ConfigDir;
use crate::ConfigFile;
use std::path::PathBuf;
use strut_core::AppProfile;

/// Represents a filesystem entry that is relevant for config files.
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum ConfigEntry {
    /// A directory.
    Directory(ConfigDir),

    /// A config file.
    File(ConfigFile),
}

impl ConfigEntry {
    /// Creates a [directory](ConfigEntry::Directory)-type [`ConfigEntry`] from
    /// the given path with no questions asked (no validation on the path).
    ///
    /// This is intended for creating root entries.
    pub fn dir(path: PathBuf) -> Self {
        Self::Directory(ConfigDir::at(path))
    }

    /// Creates a [`ConfigEntry`] from the given [`PathBuf`], if the path points
    /// to a workable config-related filesystem entry.
    pub fn try_from(path: PathBuf) -> Option<Self> {
        Self::try_from_with_profile(path, None)
    }

    /// Creates a [`ConfigEntry`] from the given [`PathBuf`], if the path points
    /// to a workable config-related filesystem entry, optionally applying the
    /// given known profile name to the config files.
    pub fn try_from_with_profile(path: PathBuf, known_profile: Option<&str>) -> Option<Self> {
        // A directory is easy
        if path.is_dir() {
            let config_dir = ConfigDir::make_with_profile(path, known_profile);
            return Some(ConfigEntry::Directory(config_dir));
        }

        // A file is easy too
        if let Some(config_file) = ConfigFile::try_make_with_profile(path, known_profile) {
            return Some(ConfigEntry::File(config_file));
        }

        None
    }
}

impl ConfigEntry {
    /// Reports whether this [`ConfigEntry`] is a
    /// [directory](ConfigEntry::Directory).
    pub fn is_directory(&self) -> bool {
        matches!(self, ConfigEntry::Directory(_))
    }

    /// Reports whether this [`ConfigEntry`] is a config
    /// [file](ConfigEntry::File).
    pub fn is_file(&self) -> bool {
        matches!(self, ConfigEntry::File(_))
    }

    /// Returns a reference to the internally held [`PathBuf`].
    pub fn path(&self) -> &PathBuf {
        match *self {
            ConfigEntry::Directory(ref config_dir) => config_dir.path(),
            ConfigEntry::File(ref config_file) => config_file.path(),
        }
    }

    /// Returns a reference to the internally held file/directory name.
    pub fn name(&self) -> Option<&str> {
        self.path().file_name().and_then(std::ffi::OsStr::to_str)
    }

    /// Reports whether this [`ConfigEntry`] [applies](ConfigEntry::applies_to)
    /// to the [active](AppProfile::active) [`AppProfile`].
    pub fn applies_to_active_profile(&self) -> bool {
        self.applies_to(AppProfile::active())
    }

    /// Reports whether this [`ConfigEntry`] applies to the given [`AppProfile`].
    ///
    /// Delegates to the underlying logic for both the
    /// [directory](ConfigDir::applies_to) and the
    /// [file](ConfigFile::applies_to) variants.
    pub fn applies_to(&self, profile: impl AsRef<AppProfile>) -> bool {
        match *self {
            ConfigEntry::Directory(ref config_dir) => config_dir.applies_to(profile),
            ConfigEntry::File(ref config_file) => config_file.applies_to(profile),
        }
    }

    /// Consumes this [`ConfigEntry`] and yields only [`ConfigFile`]s.
    pub fn to_config_file(self) -> Option<ConfigFile> {
        match self {
            ConfigEntry::Directory(_) => None,
            ConfigEntry::File(config_file) => Some(config_file),
        }
    }
}

impl ConfigEntry {
    /// Iterates over [`ConfigEntry`]s that are the immediate children of this
    /// [`ConfigEntry`], if this entry is a [directory](ConfigEntry::Directory).
    /// If this entry is a file, yields a [`Once`](std::iter::Once) iterator
    /// over that file.
    ///
    /// If this entry is a directory that is [associated](ConfigDir::Specific)
    /// with a profile — the profile is carried over into all nested entries.
    ///
    /// All failure conditions (e.g., non-existing paths, un-readable files) are
    /// silently ignored.
    ///
    /// This is intended for convenient flat-mapping to move deeper into nested
    /// directories, if any.
    pub fn cd(self) -> ConfigEntryIter {
        match self {
            ConfigEntry::Directory(ref config_dir) => {
                let config_entries = config_dir.expand(config_dir.profile());
                ConfigEntryIter::Directory(config_entries.into_iter())
            }
            ConfigEntry::File(_) => ConfigEntryIter::File(std::iter::once(self)),
        }
    }

    /// Same as [`cd`](ConfigEntry::cd), but if this entry is a directory, then
    /// instead of carrying over the profile that this directory may be
    /// associated with, captures the profile from this directory’s name.
    pub fn cd_capturing_profile(self) -> ConfigEntryIter {
        match self {
            ConfigEntry::Directory(ref config_dir) => {
                let config_entries = config_dir.expand(config_dir.name());
                ConfigEntryIter::Directory(config_entries.into_iter())
            }
            ConfigEntry::File(_) => ConfigEntryIter::File(std::iter::once(self)),
        }
    }

    /// Same as [`cd`](ConfigEntry::cd), but if this entry is a directory, then
    /// instead of carrying over the profile that this directory may be
    /// associated with, explicitly “forgets” any associated profile: the
    /// children entries will not be associated with any profile.
    pub fn cd_forgetting_profile(self) -> ConfigEntryIter {
        match self {
            ConfigEntry::Directory(ref config_dir) => {
                let config_entries = config_dir.expand(None);
                ConfigEntryIter::Directory(config_entries.into_iter())
            }
            ConfigEntry::File(_) => ConfigEntryIter::File(std::iter::once(self)),
        }
    }
}

impl From<ConfigEntry> for PathBuf {
    fn from(file: ConfigEntry) -> Self {
        match file {
            ConfigEntry::Directory(config_dir) => PathBuf::from(config_dir),
            ConfigEntry::File(config_file) => PathBuf::from(config_file),
        }
    }
}

/// Represents an iterator on the expanded [`ConfigEntry`]:
pub enum ConfigEntryIter {
    /// For a [directory](ConfigEntry::Directory), iterates over nested
    /// [`ConfigEntry`]s.
    Directory(std::vec::IntoIter<ConfigEntry>),

    /// For a [file](ConfigEntry::File), iterates over that single file.
    File(std::iter::Once<ConfigEntry>),
}

impl Iterator for ConfigEntryIter {
    type Item = ConfigEntry;

    fn next(&mut self) -> Option<Self::Item> {
        match *self {
            Self::Directory(ref mut iter) => iter.next(),
            Self::File(ref mut iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match *self {
            Self::Directory(ref iter) => iter.size_hint(),
            Self::File(ref iter) => iter.size_hint(),
        }
    }
}
