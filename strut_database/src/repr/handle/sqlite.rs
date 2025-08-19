use crate::repr::handle::sqlite::optimize::ProxyOptimizeOnClose;
use crate::repr::handle::Handle;
use crate::repr::log::ProxyLogSettings;
use crate::repr::pool::ProxyPoolOptions;
use humantime::parse_duration;
use serde::de::{DeserializeSeed, Error, IgnoredAny, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::Sqlite;
use sqlx_core::connection::{ConnectOptions, Connection};
use sqlx_core::database::Database;
use sqlx_core::pool::PoolOptions;
use std::collections::{BTreeMap, HashMap};
use std::fmt::{Debug, Formatter};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use strut_deserialize::{Slug, SlugMap};
use strut_factory::impl_deserialize_field;

pub mod optimize;

/// Represents a collection of uniquely named [`SqliteHandle`]s.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SqliteHandleCollection {
    handles: SlugMap<SqliteHandle>,
}

/// Defines a connection handle for a SQLite database.
#[derive(Debug, Clone)]
pub struct SqliteHandle {
    name: Arc<str>,
    identifier: Arc<str>,
    connect_options: SqliteConnectOptions,
    pool_options: PoolOptions<Sqlite>,
}

impl SqliteHandleCollection {
    /// Reports whether this collection contains a [`SqliteHandle`] with the
    /// given unique name.
    pub fn contains(&self, name: &str) -> bool {
        self.handles.contains_key(name)
    }

    /// Retrieves `Some` reference to a [`SqliteHandle`] from this collection
    /// under the given name, or `None`, if the name is not present in the
    /// collection.
    pub fn get(&self, name: &str) -> Option<&SqliteHandle> {
        self.handles.get(name)
    }

    /// Retrieves a reference to a [`SqliteHandle`] from this collection under
    /// the given name. Panics if the name is not present in the collection.
    pub fn expect(&self, name: &str) -> &SqliteHandle {
        self.get(name)
            .unwrap_or_else(|| panic!("requested an undefined SQLite connection handle '{}'", name))
    }
}

impl SqliteHandle {
    /// Creates a new handle with the given name and the given
    /// [`SqliteConnectOptions`].
    pub fn new(
        name: impl AsRef<str>,
        connect_options: SqliteConnectOptions,
        pool_options: PoolOptions<Sqlite>,
    ) -> Self {
        let name = Arc::from(name.as_ref());
        let identifier = Arc::from(connect_options.get_filename().to_string_lossy().as_ref());

        Self {
            name,
            identifier,
            connect_options,
            pool_options,
        }
    }

    /// Consumes and re-creates this handle, applying the given `name`.
    ///
    /// This is intended mostly for testing convenience.
    pub fn recreate_with_name(self, name: impl AsRef<str>) -> Self {
        Self::new(name, self.connect_options, self.pool_options)
    }

    /// Consumes and re-creates this handle, applying the given `modifier`
    /// function to the internally held [`SqliteConnectOptions`].
    ///
    /// This is intended for cases where the connection options need to be
    /// modified with closures, which obviously cannot be done from a
    /// configuration file.
    pub fn recreate_with_connect_options(
        self,
        modifier: impl FnOnce(SqliteConnectOptions) -> SqliteConnectOptions,
    ) -> Self {
        let connect_options = modifier(self.connect_options);

        Self::new(self.name, connect_options, self.pool_options)
    }

    /// Consumes and re-creates this handle, applying the given `modifier`
    /// function to the internally held [`PoolOptions`].
    ///
    /// This is intended for cases where the pool options need to be modified
    /// with closures, which obviously cannot be done from a configuration file.
    pub fn recreate_with_pool_options(
        self,
        modifier: impl FnOnce(PoolOptions<Sqlite>) -> PoolOptions<Sqlite>,
    ) -> Self {
        let pool_options = modifier(self.pool_options);

        Self::new(self.name, self.connect_options, pool_options)
    }
}

impl Handle for SqliteHandle {
    type Database = Sqlite;

    fn name(&self) -> &str {
        &self.name
    }

    fn identifier(&self) -> &str {
        &self.identifier
    }

    fn connect_options(
        &self,
    ) -> &<<Self::Database as Database>::Connection as Connection>::Options {
        &self.connect_options
    }

    fn pool_options(&self) -> &PoolOptions<Self::Database> {
        &self.pool_options
    }

    fn destruct(
        self,
    ) -> (
        <<Self::Database as Database>::Connection as Connection>::Options,
        PoolOptions<Self::Database>,
    ) {
        (self.connect_options, self.pool_options)
    }
}

impl SqliteHandle {
    fn default_name() -> &'static str {
        "default"
    }
}

/// General trait implementations.
const _: () = {
    impl Default for SqliteHandle {
        fn default() -> Self {
            Self::new(
                Self::default_name(),
                SqliteConnectOptions::default(),
                PoolOptions::default(),
            )
        }
    }

    impl PartialEq for SqliteHandle {
        fn eq(&self, other: &Self) -> bool {
            self.name == other.name
                && self.identifier == other.identifier
                && format!("{:?}", self.connect_options) == format!("{:?}", other.connect_options)
                && format!("{:?}", self.pool_options) == format!("{:?}", other.pool_options)
        }
    }

    impl Eq for SqliteHandle {}

    impl AsRef<SqliteHandle> for SqliteHandle {
        fn as_ref(&self) -> &SqliteHandle {
            self
        }
    }

    impl AsRef<SqliteConnectOptions> for SqliteHandle {
        fn as_ref(&self) -> &SqliteConnectOptions {
            &self.connect_options
        }
    }

    impl AsRef<PoolOptions<Sqlite>> for SqliteHandle {
        fn as_ref(&self) -> &PoolOptions<Sqlite> {
            &self.pool_options
        }
    }
};

const _: () = {
    impl<S> FromIterator<(S, SqliteHandle)> for SqliteHandleCollection
    where
        S: Into<String>,
    {
        fn from_iter<T: IntoIterator<Item = (S, SqliteHandle)>>(iter: T) -> Self {
            let handles = iter.into_iter().map(|(k, v)| (k.into(), v)).collect();
            Self { handles }
        }
    }

    impl<const N: usize, S> From<[(S, SqliteHandle); N]> for SqliteHandleCollection
    where
        S: Into<String>,
    {
        fn from(value: [(S, SqliteHandle); N]) -> Self {
            value.into_iter().collect()
        }
    }
};

/// Deserialize implementation.
const _: () = {
    impl<'de> Deserialize<'de> for SqliteHandleCollection {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_map(SqliteHandleCollectionVisitor)
        }
    }

    struct SqliteHandleCollectionVisitor;

    impl<'de> Visitor<'de> for SqliteHandleCollectionVisitor {
        type Value = SqliteHandleCollection;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a map of SQLite connection handles")
        }

        fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let grouped = Slug::group_map(map)?;
            let mut handles = HashMap::with_capacity(grouped.len());

            for (key, value) in grouped {
                let seed = SqliteHandleSeed {
                    name: key.original(),
                };
                let handle = seed.deserialize(value).map_err(Error::custom)?;
                handles.insert(key, handle);
            }

            Ok(SqliteHandleCollection {
                handles: SlugMap::new(handles),
            })
        }
    }

    struct SqliteHandleSeed<'a> {
        name: &'a str,
    }

    impl<'de> DeserializeSeed<'de> for SqliteHandleSeed<'_> {
        type Value = SqliteHandle;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_any(SqliteHandleSeedVisitor { name: self.name })
        }
    }

    struct SqliteHandleSeedVisitor<'a> {
        name: &'a str,
    }

    impl<'de> Visitor<'de> for SqliteHandleSeedVisitor<'_> {
        type Value = SqliteHandle;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a map of SQLite connection handle or an SQLite URL")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            visit_url(value, Some(self.name))
        }

        fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            visit_handle(map, Some(self.name))
        }
    }

    impl<'de> Deserialize<'de> for SqliteHandle {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_any(SqliteHandleVisitor)
        }
    }

    struct SqliteHandleVisitor;

    impl<'de> Visitor<'de> for SqliteHandleVisitor {
        type Value = SqliteHandle;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a map of SQLite connection handle or an SQLite URL")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            visit_url(value, None)
        }

        fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            visit_handle(map, None)
        }
    }
};

pub(crate) fn visit_url<E>(value: &str, known_name: Option<&str>) -> Result<SqliteHandle, E>
where
    E: Error,
{
    let name = known_name.unwrap_or_else(|| SqliteHandle::default_name());
    let connect_options = SqliteConnectOptions::from_str(value).map_err(E::custom)?;

    Ok(SqliteHandle::new(
        name,
        connect_options,
        PoolOptions::default(),
    ))
}

pub(crate) fn visit_handle<'de, A>(
    mut map: A,
    known_name: Option<&str>,
) -> Result<SqliteHandle, A::Error>
where
    A: MapAccess<'de>,
{
    // Type hints are needed occasionally where the compiler is likely to
    // guess the type wrongly. This is especially true for `String` values
    // because the compiler would infer them to be `&str`, and some
    // compilers don’t support deserializing into a string reference.
    let mut name: Option<String> = None;
    let mut filename: Option<PathBuf> = None;
    let mut in_memory = None;
    let mut read_only = None;
    let mut create_if_missing = None;
    let mut shared_cache = None;
    let mut statement_cache_capacity = None;
    let mut busy_timeout = None;
    let mut log_settings = None;
    let mut immutable = None;
    let mut vfs: Option<Option<String>> = None;
    let mut pragmas: Option<BTreeMap<String, String>> = None;
    let mut extensions: Option<BTreeMap<String, Option<String>>> = None;
    let mut command_channel_size = None;
    let mut row_channel_size = None;
    let mut serialized = None;
    let mut thread_name_prefix: Option<String> = None;
    let mut optimize_on_close: Option<ProxyOptimizeOnClose> = None;
    let mut pool_options: Option<ProxyPoolOptions<Sqlite>> = None;

    while let Some(key) = map.next_key()? {
        match key {
            SqliteHandleField::name => key.poll(&mut map, &mut name)?,
            SqliteHandleField::url => {
                let url = map.next_value::<String>()?;
                return visit_url(&url, known_name);
            }
            SqliteHandleField::filename => key.poll(&mut map, &mut filename)?,
            SqliteHandleField::in_memory => key.poll(&mut map, &mut in_memory)?,
            SqliteHandleField::read_only => key.poll(&mut map, &mut read_only)?,
            SqliteHandleField::create_if_missing => key.poll(&mut map, &mut create_if_missing)?,
            SqliteHandleField::shared_cache => key.poll(&mut map, &mut shared_cache)?,
            SqliteHandleField::statement_cache_capacity => {
                key.poll(&mut map, &mut statement_cache_capacity)?
            }
            SqliteHandleField::busy_timeout => {
                let duration_string = map.next_value::<String>()?;
                let duration = parse_duration(&duration_string).map_err(Error::custom)?;
                busy_timeout = Some(duration);
                IgnoredAny
            }
            SqliteHandleField::log_settings => key.poll(&mut map, &mut log_settings)?,
            SqliteHandleField::immutable => key.poll(&mut map, &mut immutable)?,
            SqliteHandleField::vfs => key.poll(&mut map, &mut vfs)?,
            SqliteHandleField::pragmas => key.poll(&mut map, &mut pragmas)?,
            SqliteHandleField::extensions => key.poll(&mut map, &mut extensions)?,
            SqliteHandleField::command_channel_size => {
                key.poll(&mut map, &mut command_channel_size)?
            }
            SqliteHandleField::row_channel_size => key.poll(&mut map, &mut row_channel_size)?,
            SqliteHandleField::serialized => key.poll(&mut map, &mut serialized)?,
            SqliteHandleField::thread_name_prefix => key.poll(&mut map, &mut thread_name_prefix)?,
            SqliteHandleField::optimize_on_close => key.poll(&mut map, &mut optimize_on_close)?,
            SqliteHandleField::pool_options => key.poll(&mut map, &mut pool_options)?,
            SqliteHandleField::__ignore => map.next_value()?,
        };
    }

    let name = match known_name {
        Some(known_name) => known_name,
        None => name
            .as_deref()
            .unwrap_or_else(|| SqliteHandle::default_name()),
    };

    let mut connect_options = SqliteConnectOptions::default();

    if let Some(ref filename) = filename {
        connect_options = connect_options.filename(filename);
    }

    if let Some(in_memory) = in_memory {
        connect_options = connect_options.in_memory(in_memory);
    }

    if let Some(read_only) = read_only {
        connect_options = connect_options.read_only(read_only);
    }

    if let Some(create_if_missing) = create_if_missing {
        connect_options = connect_options.create_if_missing(create_if_missing);
    }

    if let Some(shared_cache) = shared_cache {
        connect_options = connect_options.shared_cache(shared_cache);
    }

    if let Some(statement_cache_capacity) = statement_cache_capacity {
        connect_options = connect_options.statement_cache_capacity(statement_cache_capacity);
    }

    if let Some(busy_timeout) = busy_timeout {
        connect_options = connect_options.busy_timeout(busy_timeout);
    }

    if let Some(ProxyLogSettings {
        statements_level,
        slow_statements_level,
        slow_statements_duration,
    }) = log_settings
    {
        connect_options = connect_options.log_statements(statements_level);
        connect_options =
            connect_options.log_slow_statements(slow_statements_level, slow_statements_duration);
    }

    if let Some(immutable) = immutable {
        connect_options = connect_options.immutable(immutable);
    }

    if let Some(Some(vfs)) = vfs {
        connect_options = connect_options.vfs(vfs);
    }

    if let Some(pragmas) = pragmas {
        for (key, value) in pragmas.into_iter() {
            connect_options = connect_options.pragma(key, value);
        }
    }

    if let Some(extensions) = extensions {
        for (key, value) in extensions.into_iter() {
            match value {
                None => connect_options = connect_options.extension(key),
                Some(value) => {
                    connect_options = connect_options.extension_with_entrypoint(key, value);
                }
            }
        }
    }

    if let Some(command_channel_size) = command_channel_size {
        connect_options = connect_options.command_buffer_size(command_channel_size);
    }

    if let Some(row_channel_size) = row_channel_size {
        connect_options = connect_options.row_buffer_size(row_channel_size);
    }

    if let Some(serialized) = serialized {
        connect_options = connect_options.serialized(serialized);
    }

    if let Some(thread_name_prefix) = thread_name_prefix {
        connect_options =
            connect_options.thread_name(move |num| format!("{} {}", thread_name_prefix, num));
    }

    if let Some(ProxyOptimizeOnClose {
        enabled,
        analysis_limit,
    }) = optimize_on_close
    {
        connect_options = connect_options.optimize_on_close(enabled, analysis_limit);
    }

    let pool_options = PoolOptions::from(pool_options.unwrap_or_default());

    Ok(SqliteHandle::new(name, connect_options, pool_options))
}

impl_deserialize_field!(
    SqliteHandleField,
    strut_deserialize::Slug::eq_as_slugs,
    url,
    name,
    filename | filepath | file,
    in_memory | memory,
    read_only | readonly | ro,
    create_if_missing,
    shared_cache,
    statement_cache_capacity,
    busy_timeout,
    log_settings,
    immutable,
    vfs,
    pragmas,
    extensions,
    command_channel_size | command_buffer_size,
    row_channel_size | row_buffer_size,
    // collations, // collations are functions; a function cannot be provided from a config file
    serialized,
    thread_name_prefix | thread_name,
    optimize_on_close | analysis_limit,
    pool_options | pool,
);

#[cfg(test)]
mod tests {
    use super::*;
    use log::LevelFilter;
    use pretty_assertions::assert_eq;
    use std::time::Duration;

    #[test]
    fn from_string() {
        // Given
        let input = r#"
sqlite://
"#;

        // When
        let actual_output = serde_yml::from_str::<SqliteHandle>(input).unwrap();
        let expected_output = SqliteHandle::default()
            .recreate_with_connect_options(|connect_options| connect_options.filename(""));

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn from_map_url() {
        // Given
        let input = r#"
url: sqlite://
"#;

        // When
        let actual_output = serde_yml::from_str::<SqliteHandle>(input).unwrap();
        let expected_output = SqliteHandle::default()
            .recreate_with_connect_options(|connect_options| connect_options.filename(""));

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn from_map_sparse() {
        // Given
        let input = r#"
filename: file.db
"#;

        // When
        let actual_output = serde_yml::from_str::<SqliteHandle>(input).unwrap();
        let expected_output = SqliteHandle::default()
            .recreate_with_connect_options(|connect_options| connect_options.filename("file.db"));

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn from_map_full() {
        // Given
        let input = r#"
filename: file.db
in_memory: true
read_only: true
create_if_missing: true
shared_cache: true
statement_cache_capacity: 777
busy_timeout: 2s 100ms
log_settings:
    statements_level: error
    slow_statements_level: error
    slow_statements_duration: 2s 500ms
immutable: true
vfs: some_vfs
pragmas:
    foreign_keys: OFF
extensions:
    extension_a: /path/to/entrypoint
command_channel_size: 17
row_channel_size: 18
serialized: true
thread_name_prefix: ~
optimize_on_close:
    enabled: true
    analysis_limit: 35
pool_options:
    min_connections: 3
    max_connections: 4
    test_before_acquire: false
    acquire_time_level: error
    acquire_slow_level: error
    acquire_slow_threshold: 300ms
    acquire_timeout: 17s
    max_lifetime: 18s
    idle_timeout: 19s
    "#;

        // When
        let actual_output = serde_yml::from_str::<SqliteHandle>(input).unwrap();
        let expected_output = SqliteHandle::default()
            .recreate_with_connect_options(|connect_options| {
                connect_options
                    .filename("file.db")
                    .in_memory(true)
                    .read_only(true)
                    .create_if_missing(true)
                    .shared_cache(true)
                    .statement_cache_capacity(777)
                    .busy_timeout(Duration::from_millis(2100))
                    .log_statements(LevelFilter::Error)
                    .log_slow_statements(LevelFilter::Error, Duration::from_millis(2500))
                    .immutable(true)
                    .vfs("some_vfs")
                    .pragma("foreign_keys", "OFF")
                    .extension_with_entrypoint("extension_a", "/path/to/entrypoint")
                    .command_buffer_size(17)
                    .row_buffer_size(18)
                    .serialized(true)
                    .optimize_on_close(true, 35)
            })
            .recreate_with_pool_options(|pool_options| {
                pool_options
                    .min_connections(3)
                    .max_connections(4)
                    .test_before_acquire(false)
                    .acquire_time_level(LevelFilter::Error)
                    .acquire_slow_level(LevelFilter::Error)
                    .acquire_slow_threshold(Duration::from_millis(300))
                    .acquire_timeout(Duration::from_secs(17))
                    .max_lifetime(Duration::from_secs(18))
                    .idle_timeout(Duration::from_secs(19))
            });

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn from_map_nulls() {
        // Given
        let input = r#"
filename: file.db
in_memory: true
read_only: true
create_if_missing: true
shared_cache: true
statement_cache_capacity: 777
busy_timeout: 2s 100ms
log_settings:
    statements_level: error
    slow_statements_level: error
    slow_statements_duration: 2s 500ms
immutable: true
vfs: ~
pragmas:
    foreign_keys: OFF
extensions:
    extension_a: /path/to/entrypoint
command_channel_size: 17
row_channel_size: 18
serialized: true
thread_name_prefix: ~
optimize_on_close: false
pool_options:
    min_connections: 3
    max_connections: 4
    test_before_acquire: false
    acquire_time_level: error
    acquire_slow_level: error
    acquire_slow_threshold: 300ms
    acquire_timeout: 17s
    max_lifetime: ~
    idle_timeout: ~
    "#;

        // When
        let actual_output = serde_yml::from_str::<SqliteHandle>(input).unwrap();
        let expected_output = SqliteHandle::default()
            .recreate_with_connect_options(|connect_options| {
                connect_options
                    .filename("file.db")
                    .in_memory(true)
                    .read_only(true)
                    .create_if_missing(true)
                    .shared_cache(true)
                    .statement_cache_capacity(777)
                    .busy_timeout(Duration::from_millis(2100))
                    .log_statements(LevelFilter::Error)
                    .log_slow_statements(LevelFilter::Error, Duration::from_millis(2500))
                    .immutable(true)
                    .pragma("foreign_keys", "OFF")
                    .extension_with_entrypoint("extension_a", "/path/to/entrypoint")
                    .command_buffer_size(17)
                    .row_buffer_size(18)
                    .serialized(true)
            })
            .recreate_with_pool_options(|pool_options| {
                pool_options
                    .min_connections(3)
                    .max_connections(4)
                    .test_before_acquire(false)
                    .acquire_time_level(LevelFilter::Error)
                    .acquire_slow_level(LevelFilter::Error)
                    .acquire_slow_threshold(Duration::from_millis(300))
                    .acquire_timeout(Duration::from_secs(17))
            });

        // Then
        assert_eq!(expected_output, actual_output);
    }
}
