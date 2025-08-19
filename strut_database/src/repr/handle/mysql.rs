use crate::repr::cert::ProxyCertificateInput;
use crate::repr::handle::mysql::ssl::ProxyMySqlSslMode;
use crate::repr::handle::Handle;
use crate::repr::log::ProxyLogSettings;
use crate::repr::pool::ProxyPoolOptions;
use serde::de::{DeserializeSeed, Error, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use sqlx::mysql::{MySqlConnectOptions, MySqlSslMode};
use sqlx::MySql;
use sqlx_core::connection::{ConnectOptions, Connection};
use sqlx_core::database::Database;
use sqlx_core::pool::PoolOptions;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use strut_deserialize::{Slug, SlugMap};
use strut_factory::impl_deserialize_field;

pub mod ssl;

/// Represents a collection of uniquely named [`MySqlHandle`]s.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MySqlHandleCollection {
    handles: SlugMap<MySqlHandle>,
}

/// Defines a connection handle for a MySQL database.
#[derive(Debug, Clone)]
pub struct MySqlHandle {
    name: Arc<str>,
    identifier: Arc<str>,
    connect_options: MySqlConnectOptions,
    pool_options: PoolOptions<MySql>,
}

impl MySqlHandle {
    /// Creates a new handle with the given name and the given
    /// [`MySqlConnectOptions`].
    pub fn new(
        name: impl AsRef<str>,
        connect_options: MySqlConnectOptions,
        pool_options: PoolOptions<MySql>,
    ) -> Self {
        let name = Arc::from(name.as_ref());
        let identifier = Self::compose_identifier(
            connect_options.get_host(),
            connect_options.get_port(),
            connect_options.get_username(),
            connect_options.get_database(),
        );

        Self {
            name,
            identifier,
            connect_options,
            pool_options,
        }
    }

    /// Composes a non-sensitive identifier useful for debug-printing a handle.
    fn compose_identifier(host: &str, port: u16, user: &str, database: Option<&str>) -> Arc<str> {
        Arc::from(format!(
            "mysql://{}@{}:{}/{}",
            user,
            host,
            port,
            database.unwrap_or(""),
        ))
    }

    /// Consumes and re-creates this handle, applying the given `name`.
    ///
    /// This is intended mostly for testing convenience.
    pub fn recreate_with_name(self, name: impl AsRef<str>) -> Self {
        Self::new(name, self.connect_options, self.pool_options)
    }

    /// Consumes and re-creates this handle, applying the given `modifier`
    /// function to the internally held [`MySqlConnectOptions`].
    ///
    /// This is intended for cases where the connection options need to be
    /// modified with closures, which obviously cannot be done from a
    /// configuration file.
    pub fn recreate_with_connect_options(
        self,
        modifier: impl FnOnce(MySqlConnectOptions) -> MySqlConnectOptions,
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
        modifier: impl FnOnce(PoolOptions<MySql>) -> PoolOptions<MySql>,
    ) -> Self {
        let pool_options = modifier(self.pool_options);

        Self::new(self.name, self.connect_options, pool_options)
    }
}

impl MySqlHandleCollection {
    /// Reports whether this collection contains a [`MySqlHandle`] with the
    /// given unique name.
    pub fn contains(&self, name: &str) -> bool {
        self.handles.contains_key(name)
    }

    /// Retrieves `Some` reference to a [`MySqlHandle`] from this collection
    /// under the given name, or `None`, if the name is not present in the
    /// collection.
    pub fn get(&self, name: &str) -> Option<&MySqlHandle> {
        self.handles.get(name)
    }

    /// Retrieves a reference to a [`MySqlHandle`] from this collection under
    /// the given name. Panics if the name is not present in the collection.
    pub fn expect(&self, name: &str) -> &MySqlHandle {
        self.get(name)
            .unwrap_or_else(|| panic!("requested an undefined MySQL connection handle '{}'", name))
    }
}

impl Handle for MySqlHandle {
    type Database = MySql;

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

impl MySqlHandle {
    fn default_name() -> &'static str {
        "default"
    }
}

/// General trait implementations.
const _: () = {
    impl Default for MySqlHandle {
        fn default() -> Self {
            Self::new(
                Self::default_name(),
                MySqlConnectOptions::default(),
                PoolOptions::default(),
            )
        }
    }

    impl PartialEq for MySqlHandle {
        fn eq(&self, other: &Self) -> bool {
            self.name == other.name
                && self.identifier == other.identifier
                && format!("{:?}", self.connect_options) == format!("{:?}", other.connect_options)
                && format!("{:?}", self.pool_options) == format!("{:?}", other.pool_options)
        }
    }

    impl Eq for MySqlHandle {}

    impl AsRef<MySqlHandle> for MySqlHandle {
        fn as_ref(&self) -> &MySqlHandle {
            self
        }
    }

    impl AsRef<MySqlConnectOptions> for MySqlHandle {
        fn as_ref(&self) -> &MySqlConnectOptions {
            &self.connect_options
        }
    }

    impl AsRef<PoolOptions<MySql>> for MySqlHandle {
        fn as_ref(&self) -> &PoolOptions<MySql> {
            &self.pool_options
        }
    }
};

const _: () = {
    impl<S> FromIterator<(S, MySqlHandle)> for MySqlHandleCollection
    where
        S: Into<String>,
    {
        fn from_iter<T: IntoIterator<Item = (S, MySqlHandle)>>(iter: T) -> Self {
            let handles = iter.into_iter().map(|(k, v)| (k.into(), v)).collect();
            Self { handles }
        }
    }

    impl<const N: usize, S> From<[(S, MySqlHandle); N]> for MySqlHandleCollection
    where
        S: Into<String>,
    {
        fn from(value: [(S, MySqlHandle); N]) -> Self {
            value.into_iter().collect()
        }
    }
};

/// Deserialize implementation.
const _: () = {
    impl<'de> Deserialize<'de> for MySqlHandleCollection {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_map(MySqlHandleCollectionVisitor)
        }
    }

    struct MySqlHandleCollectionVisitor;

    impl<'de> Visitor<'de> for MySqlHandleCollectionVisitor {
        type Value = MySqlHandleCollection;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a map of MySQL connection handles")
        }

        fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let grouped = Slug::group_map(map)?;
            let mut handles = HashMap::with_capacity(grouped.len());

            for (key, value) in grouped {
                let seed = MySqlHandleSeed {
                    name: key.original(),
                };
                let handle = seed.deserialize(value).map_err(Error::custom)?;
                handles.insert(key, handle);
            }

            Ok(MySqlHandleCollection {
                handles: SlugMap::new(handles),
            })
        }
    }

    struct MySqlHandleSeed<'a> {
        name: &'a str,
    }

    impl<'de> DeserializeSeed<'de> for MySqlHandleSeed<'_> {
        type Value = MySqlHandle;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_any(MySqlHandleSeedVisitor { name: self.name })
        }
    }

    struct MySqlHandleSeedVisitor<'a> {
        name: &'a str,
    }

    impl<'de> Visitor<'de> for MySqlHandleSeedVisitor<'_> {
        type Value = MySqlHandle;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a map of MySQL connection handle or a MySQL URL")
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

    impl<'de> Deserialize<'de> for MySqlHandle {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_any(MySqlHandleVisitor)
        }
    }

    struct MySqlHandleVisitor;

    impl<'de> Visitor<'de> for MySqlHandleVisitor {
        type Value = MySqlHandle;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a map of MySQL connection handle or a MySQL URL")
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

pub(crate) fn visit_url<E>(value: &str, known_name: Option<&str>) -> Result<MySqlHandle, E>
where
    E: Error,
{
    let name = known_name.unwrap_or_else(|| MySqlHandle::default_name());
    let connect_options = MySqlConnectOptions::from_str(value).map_err(E::custom)?;

    Ok(MySqlHandle::new(
        name,
        connect_options,
        PoolOptions::default(),
    ))
}

pub(crate) fn visit_handle<'de, A>(
    mut map: A,
    known_name: Option<&str>,
) -> Result<MySqlHandle, A::Error>
where
    A: MapAccess<'de>,
{
    // Type hints are needed occasionally where the compiler is likely to
    // guess the type wrongly. This is especially true for `String` values
    // because the compiler would infer them to be `&str`, and some
    // compilers don’t support deserializing into a string reference.
    let mut name: Option<String> = None;
    let mut host: Option<String> = None;
    let mut port = None;
    let mut socket: Option<Option<PathBuf>> = None;
    let mut username: Option<String> = None;
    // The underlying MySqlConnectOptions expose the password as a string,
    // so no point obfuscating here.
    let mut password: Option<Option<String>> = None;
    let mut database: Option<Option<String>> = None;
    let mut ssl_mode: Option<ProxyMySqlSslMode> = None;
    let mut ssl_ca: Option<Option<ProxyCertificateInput>> = None;
    let mut ssl_client_cert: Option<Option<ProxyCertificateInput>> = None;
    let mut ssl_client_key: Option<Option<ProxyCertificateInput>> = None;
    let mut statement_cache_capacity = None;
    let mut charset: Option<String> = None;
    let mut collation: Option<Option<String>> = None;
    let mut log_settings: Option<ProxyLogSettings> = None;
    let mut pipes_as_concat = None;
    let mut enable_cleartext_plugin = None;
    let mut no_engine_substitution = None;
    let mut timezone: Option<Option<String>> = None;
    let mut set_names = None;
    let mut pool_options: Option<ProxyPoolOptions<MySql>> = None;

    while let Some(key) = map.next_key()? {
        match key {
            MySqlHandleField::name => key.poll(&mut map, &mut name)?,
            MySqlHandleField::url => {
                let url = map.next_value::<String>()?;
                return visit_url(&url, known_name);
            }
            MySqlHandleField::host => key.poll(&mut map, &mut host)?,
            MySqlHandleField::port => key.poll(&mut map, &mut port)?,
            MySqlHandleField::socket => key.poll(&mut map, &mut socket)?,
            MySqlHandleField::username => key.poll(&mut map, &mut username)?,
            MySqlHandleField::password => key.poll(&mut map, &mut password)?,
            MySqlHandleField::database => key.poll(&mut map, &mut database)?,
            MySqlHandleField::ssl_mode => key.poll(&mut map, &mut ssl_mode)?,
            MySqlHandleField::ssl_ca => key.poll(&mut map, &mut ssl_ca)?,
            MySqlHandleField::ssl_client_cert => key.poll(&mut map, &mut ssl_client_cert)?,
            MySqlHandleField::ssl_client_key => key.poll(&mut map, &mut ssl_client_key)?,
            MySqlHandleField::statement_cache_capacity => {
                key.poll(&mut map, &mut statement_cache_capacity)?
            }
            MySqlHandleField::charset => key.poll(&mut map, &mut charset)?,
            MySqlHandleField::collation => key.poll(&mut map, &mut collation)?,
            MySqlHandleField::log_settings => key.poll(&mut map, &mut log_settings)?,
            MySqlHandleField::pipes_as_concat => key.poll(&mut map, &mut pipes_as_concat)?,
            MySqlHandleField::enable_cleartext_plugin => {
                key.poll(&mut map, &mut enable_cleartext_plugin)?
            }
            MySqlHandleField::no_engine_substitution => {
                key.poll(&mut map, &mut no_engine_substitution)?
            }
            MySqlHandleField::timezone => key.poll(&mut map, &mut timezone)?,
            MySqlHandleField::set_names => key.poll(&mut map, &mut set_names)?,
            MySqlHandleField::pool_options => key.poll(&mut map, &mut pool_options)?,
            MySqlHandleField::__ignore => map.next_value()?,
        };
    }

    let name = match known_name {
        Some(known_name) => known_name,
        None => name
            .as_deref()
            .unwrap_or_else(|| MySqlHandle::default_name()),
    };

    let mut connect_options = MySqlConnectOptions::default();

    if let Some(ref host) = host {
        connect_options = connect_options.host(host);
    }

    if let Some(port) = port {
        connect_options = connect_options.port(port);
    }

    if let Some(Some(ref socket)) = socket {
        connect_options = connect_options.socket(socket);
    }

    if let Some(ref username) = username {
        connect_options = connect_options.username(username);
    }

    if let Some(Some(ref password)) = password {
        connect_options = connect_options.password(password);
    }

    if let Some(Some(ref database)) = database {
        connect_options = connect_options.database(database);
    }

    if let Some(ssl_mode) = ssl_mode {
        connect_options = connect_options.ssl_mode(MySqlSslMode::from(ssl_mode));
    }

    if let Some(Some(ssl_ca)) = ssl_ca {
        match ssl_ca {
            ProxyCertificateInput::Inline(bytes) => {
                connect_options = connect_options.ssl_ca_from_pem(bytes);
            }
            ProxyCertificateInput::File(ref path) => {
                connect_options = connect_options.ssl_ca(path);
            }
        }
    }

    if let Some(Some(ssl_client_cert)) = ssl_client_cert {
        match ssl_client_cert {
            ProxyCertificateInput::Inline(ref bytes) => {
                connect_options = connect_options.ssl_client_cert_from_pem(bytes);
            }
            ProxyCertificateInput::File(ref path) => {
                connect_options = connect_options.ssl_client_cert(path);
            }
        }
    }

    if let Some(Some(ssl_client_key)) = ssl_client_key {
        match ssl_client_key {
            ProxyCertificateInput::Inline(ref bytes) => {
                connect_options = connect_options.ssl_client_key_from_pem(bytes);
            }
            ProxyCertificateInput::File(ref path) => {
                connect_options = connect_options.ssl_client_key(path);
            }
        }
    }

    if let Some(statement_cache_capacity) = statement_cache_capacity {
        connect_options = connect_options.statement_cache_capacity(statement_cache_capacity);
    }

    if let Some(ref charset) = charset {
        connect_options = connect_options.charset(charset);
    }

    if let Some(Some(ref collation)) = collation {
        connect_options = connect_options.collation(collation);
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

    if let Some(pipes_as_concat) = pipes_as_concat {
        connect_options = connect_options.pipes_as_concat(pipes_as_concat);
    }

    if let Some(enable_cleartext_plugin) = enable_cleartext_plugin {
        connect_options = connect_options.enable_cleartext_plugin(enable_cleartext_plugin);
    }

    if let Some(no_engine_substitution) = no_engine_substitution {
        connect_options = connect_options.no_engine_substitution(no_engine_substitution);
    }

    if let Some(timezone) = timezone {
        connect_options = connect_options.timezone(timezone);
    }

    if let Some(set_names) = set_names {
        connect_options = connect_options.set_names(set_names);
    }

    let pool_options = PoolOptions::from(pool_options.unwrap_or_default());

    Ok(MySqlHandle::new(name, connect_options, pool_options))
}

impl_deserialize_field!(
    MySqlHandleField,
    strut_deserialize::Slug::eq_as_slugs,
    name,
    url,
    host | hostname,
    port,
    socket,
    username | user,
    password | pass,
    database | db,
    ssl_mode | ssl,
    ssl_ca,
    ssl_client_cert,
    ssl_client_key,
    statement_cache_capacity,
    charset,
    collation,
    log_settings | log,
    pipes_as_concat,
    enable_cleartext_plugin,
    no_engine_substitution,
    timezone | tz,
    set_names,
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
mysql://
"#;

        // When
        let actual_output = serde_yml::from_str::<MySqlHandle>(input).unwrap();
        let expected_output = MySqlHandle::default();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn from_map_url() {
        // Given
        let input = r#"
url: mysql://
"#;

        // When
        let actual_output = serde_yml::from_str::<MySqlHandle>(input).unwrap();
        let expected_output = MySqlHandle::default();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn from_map_sparse() {
        // Given
        let input = r#"
host: example.com
port: 9999
username: alice
password: secret
database: candy_shop
"#;

        // When
        let actual_output = serde_yml::from_str::<MySqlHandle>(input).unwrap();
        let expected_output =
            MySqlHandle::default().recreate_with_connect_options(|connect_options| {
                connect_options
                    .host("example.com")
                    .port(9999)
                    .username("alice")
                    .password("secret")
                    .database("candy_shop")
            });

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn from_map_full() {
        // Given
        let input = r#"
host: example.com
port: 9999
username: alice
password: secret
database: candy_shop
socket: /var/run/mysqld/mysqld.sock
ssl_mode: verify_ca
ssl_ca: /etc/ssl/certs/ca-certificates.crt
ssl_client_cert: /etc/ssl/certs/client-cert.pem
ssl_client_key: /etc/ssl/private/client-key.pem
statement_cache_capacity: 777
charset: latin1
collation: latin1_bin
log_settings:
    statements_level: error
    slow_statements_level: error
    slow_statements_duration: 2s 500ms
pipes_as_concat: false
enable_cleartext_plugin: true
no_engine_substitution: false
timezone: +07:30
set_names: false
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
        let actual_output = serde_yml::from_str::<MySqlHandle>(input).unwrap();
        let expected_output = MySqlHandle::default()
            .recreate_with_connect_options(|connect_options| {
                connect_options
                    .host("example.com")
                    .port(9999)
                    .username("alice")
                    .password("secret")
                    .database("candy_shop")
                    .socket("/var/run/mysqld/mysqld.sock")
                    .ssl_mode(MySqlSslMode::VerifyCa)
                    .ssl_ca("/etc/ssl/certs/ca-certificates.crt")
                    .ssl_client_cert("/etc/ssl/certs/client-cert.pem")
                    .ssl_client_key("/etc/ssl/private/client-key.pem")
                    .statement_cache_capacity(777)
                    .charset("latin1")
                    .collation("latin1_bin")
                    .log_statements(LevelFilter::Error)
                    .log_slow_statements(LevelFilter::Error, Duration::from_millis(2500))
                    .pipes_as_concat(false)
                    .enable_cleartext_plugin(true)
                    .no_engine_substitution(false)
                    .timezone(Some("+07:30".to_string()))
                    .set_names(false)
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
host: example.com
port: 9999
username: alice
password: ~
database: ~
socket: ~
ssl_mode: verify_ca
ssl_ca: ~
ssl_client_cert: ~
ssl_client_key: ~
statement_cache_capacity: 777
charset: latin1
collation: ~
log_settings:
    statements_level: error
    slow_statements_level: error
    slow_statements_duration: 2s 500ms
pipes_as_concat: false
enable_cleartext_plugin: true
no_engine_substitution: false
timezone: ~
set_names: false
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
        let actual_output = serde_yml::from_str::<MySqlHandle>(input).unwrap();
        let expected_output = MySqlHandle::default()
            .recreate_with_connect_options(|connect_options| {
                connect_options
                    .host("example.com")
                    .port(9999)
                    .username("alice")
                    .ssl_mode(MySqlSslMode::VerifyCa)
                    .statement_cache_capacity(777)
                    .charset("latin1")
                    .log_statements(LevelFilter::Error)
                    .log_slow_statements(LevelFilter::Error, Duration::from_millis(2500))
                    .pipes_as_concat(false)
                    .enable_cleartext_plugin(true)
                    .no_engine_substitution(false)
                    .timezone(None)
                    .set_names(false)
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
