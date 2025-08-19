use crate::repr::cert::ProxyCertificateInput;
use crate::repr::handle::postgres::ssl::ProxyPgSslMode;
use crate::repr::handle::Handle;
use crate::repr::log::ProxyLogSettings;
use crate::repr::pool::ProxyPoolOptions;
use serde::de::{DeserializeSeed, Error, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use sqlx::Postgres;
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

pub mod ssl;

/// Represents a collection of uniquely named [`PostgresHandle`]s.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PostgresHandleCollection {
    handles: SlugMap<PostgresHandle>,
}

/// Defines a connection handle for a PostgreSQL database.
#[derive(Debug, Clone)]
pub struct PostgresHandle {
    name: Arc<str>,
    identifier: Arc<str>,
    connect_options: PgConnectOptions,
    pool_options: PoolOptions<Postgres>,
}

impl PostgresHandleCollection {
    /// Reports whether this collection contains a [`PostgresHandle`] with the
    /// given unique name.
    pub fn contains(&self, name: &str) -> bool {
        self.handles.contains_key(name)
    }

    /// Retrieves `Some` reference to a [`PostgresHandle`] from this collection
    /// under the given name, or `None`, if the name is not present in the
    /// collection.
    pub fn get(&self, name: &str) -> Option<&PostgresHandle> {
        self.handles.get(name)
    }

    /// Retrieves a reference to a [`PostgresHandle`] from this collection under
    /// the given name. Panics if the name is not present in the collection.
    pub fn expect(&self, name: &str) -> &PostgresHandle {
        self.get(name).unwrap_or_else(|| {
            panic!(
                "requested an undefined PostgreSQL connection handle '{}'",
                name
            )
        })
    }
}

impl PostgresHandle {
    /// Creates a new handle with the given name and the given
    /// [`PgConnectOptions`].
    pub fn new(
        name: impl AsRef<str>,
        connect_options: PgConnectOptions,
        pool_options: PoolOptions<Postgres>,
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
            "postgres://{}@{}:{}/{}",
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
    /// function to the internally held [`PgConnectOptions`].
    ///
    /// This is intended for cases where the connection options need to be
    /// modified with closures, which obviously cannot be done from a
    /// configuration file.
    pub fn recreate_with_connect_options(
        self,
        modifier: impl FnOnce(PgConnectOptions) -> PgConnectOptions,
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
        modifier: impl FnOnce(PoolOptions<Postgres>) -> PoolOptions<Postgres>,
    ) -> Self {
        let pool_options = modifier(self.pool_options);

        Self::new(self.name, self.connect_options, pool_options)
    }
}

impl Handle for PostgresHandle {
    type Database = Postgres;

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

impl PostgresHandle {
    fn default_name() -> &'static str {
        "default"
    }
}

/// General trait implementations.
const _: () = {
    impl Default for PostgresHandle {
        fn default() -> Self {
            Self::new(
                Self::default_name(),
                PgConnectOptions::default(),
                PoolOptions::default(),
            )
        }
    }

    impl PartialEq for PostgresHandle {
        fn eq(&self, other: &Self) -> bool {
            self.name == other.name
                && self.identifier == other.identifier
                && format!("{:?}", self.connect_options) == format!("{:?}", other.connect_options)
                && format!("{:?}", self.pool_options) == format!("{:?}", other.pool_options)
        }
    }

    impl Eq for PostgresHandle {}

    impl AsRef<PostgresHandle> for PostgresHandle {
        fn as_ref(&self) -> &PostgresHandle {
            self
        }
    }

    impl AsRef<PgConnectOptions> for PostgresHandle {
        fn as_ref(&self) -> &PgConnectOptions {
            &self.connect_options
        }
    }

    impl AsRef<PoolOptions<Postgres>> for PostgresHandle {
        fn as_ref(&self) -> &PoolOptions<Postgres> {
            &self.pool_options
        }
    }
};

const _: () = {
    impl<S> FromIterator<(S, PostgresHandle)> for PostgresHandleCollection
    where
        S: Into<String>,
    {
        fn from_iter<T: IntoIterator<Item = (S, PostgresHandle)>>(iter: T) -> Self {
            let handles = iter.into_iter().map(|(k, v)| (k.into(), v)).collect();
            Self { handles }
        }
    }

    impl<const N: usize, S> From<[(S, PostgresHandle); N]> for PostgresHandleCollection
    where
        S: Into<String>,
    {
        fn from(value: [(S, PostgresHandle); N]) -> Self {
            value.into_iter().collect()
        }
    }
};

/// Deserialize implementation.
const _: () = {
    impl<'de> Deserialize<'de> for PostgresHandleCollection {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_map(PostgresHandleCollectionVisitor)
        }
    }

    struct PostgresHandleCollectionVisitor;

    impl<'de> Visitor<'de> for PostgresHandleCollectionVisitor {
        type Value = PostgresHandleCollection;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a map of PostgreSQL connection handles")
        }

        fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let grouped = Slug::group_map(map)?;
            let mut handles = HashMap::with_capacity(grouped.len());

            for (key, value) in grouped {
                let seed = PostgresHandleSeed {
                    name: key.original(),
                };
                let handle = seed.deserialize(value).map_err(Error::custom)?;
                handles.insert(key, handle);
            }

            Ok(PostgresHandleCollection {
                handles: SlugMap::new(handles),
            })
        }
    }

    struct PostgresHandleSeed<'a> {
        name: &'a str,
    }

    impl<'de> DeserializeSeed<'de> for PostgresHandleSeed<'_> {
        type Value = PostgresHandle;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_any(PostgresHandleSeedVisitor { name: self.name })
        }
    }

    struct PostgresHandleSeedVisitor<'a> {
        name: &'a str,
    }

    impl<'de> Visitor<'de> for PostgresHandleSeedVisitor<'_> {
        type Value = PostgresHandle;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a map of PostgreSQL connection handle or a PostgreSQL URL")
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

    impl<'de> Deserialize<'de> for PostgresHandle {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_any(PostgresHandleVisitor)
        }
    }

    struct PostgresHandleVisitor;

    impl<'de> Visitor<'de> for PostgresHandleVisitor {
        type Value = PostgresHandle;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a map of PostgreSQL connection handle or a PostgreSQL URL")
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

pub(crate) fn visit_url<E>(value: &str, known_name: Option<&str>) -> Result<PostgresHandle, E>
where
    E: Error,
{
    let name = known_name.unwrap_or_else(|| PostgresHandle::default_name());
    let connect_options = PgConnectOptions::from_str(value).map_err(E::custom)?;

    Ok(PostgresHandle::new(
        name,
        connect_options,
        PoolOptions::default(),
    ))
}

pub(crate) fn visit_handle<'de, A>(
    mut map: A,
    known_name: Option<&str>,
) -> Result<PostgresHandle, A::Error>
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
    // The underlying PgConnectOptions expose the password as a string,
    // so no point obfuscating here.
    let mut password: Option<Option<String>> = None;
    let mut database: Option<Option<String>> = None;
    let mut ssl_mode: Option<ProxyPgSslMode> = None;
    let mut ssl_root_cert: Option<Option<ProxyCertificateInput>> = None;
    let mut ssl_client_cert: Option<Option<ProxyCertificateInput>> = None;
    let mut ssl_client_key: Option<Option<ProxyCertificateInput>> = None;
    let mut statement_cache_capacity = None;
    let mut application_name: Option<Option<String>> = None;
    let mut log_settings: Option<ProxyLogSettings> = None;
    let mut extra_float_digits: Option<Option<i8>> = None;
    let mut options: Option<Option<BTreeMap<String, String>>> = None;
    let mut pool_options: Option<ProxyPoolOptions<Postgres>> = None;

    while let Some(key) = map.next_key()? {
        match key {
            PostgresHandleField::name => key.poll(&mut map, &mut name)?,
            PostgresHandleField::url => {
                let url = map.next_value::<String>()?;
                return visit_url(&url, known_name);
            }
            PostgresHandleField::host => key.poll(&mut map, &mut host)?,
            PostgresHandleField::port => key.poll(&mut map, &mut port)?,
            PostgresHandleField::socket => key.poll(&mut map, &mut socket)?,
            PostgresHandleField::username => key.poll(&mut map, &mut username)?,
            PostgresHandleField::password => key.poll(&mut map, &mut password)?,
            PostgresHandleField::database => key.poll(&mut map, &mut database)?,
            PostgresHandleField::ssl_mode => key.poll(&mut map, &mut ssl_mode)?,
            PostgresHandleField::ssl_root_cert => key.poll(&mut map, &mut ssl_root_cert)?,
            PostgresHandleField::ssl_client_cert => key.poll(&mut map, &mut ssl_client_cert)?,
            PostgresHandleField::ssl_client_key => key.poll(&mut map, &mut ssl_client_key)?,
            PostgresHandleField::statement_cache_capacity => {
                key.poll(&mut map, &mut statement_cache_capacity)?
            }
            PostgresHandleField::application_name => key.poll(&mut map, &mut application_name)?,
            PostgresHandleField::log_settings => key.poll(&mut map, &mut log_settings)?,
            PostgresHandleField::extra_float_digits => {
                key.poll(&mut map, &mut extra_float_digits)?
            }
            PostgresHandleField::options => key.poll(&mut map, &mut options)?,
            PostgresHandleField::pool_options => key.poll(&mut map, &mut pool_options)?,
            PostgresHandleField::__ignore => map.next_value()?,
        };
    }

    let name = match known_name {
        Some(known_name) => known_name,
        None => name
            .as_deref()
            .unwrap_or_else(|| PostgresHandle::default_name()),
    };

    let mut connect_options = PgConnectOptions::default();

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
        connect_options = connect_options.ssl_mode(PgSslMode::from(ssl_mode));
    }

    if let Some(Some(ssl_root_cert)) = ssl_root_cert {
        match ssl_root_cert {
            ProxyCertificateInput::Inline(bytes) => {
                connect_options = connect_options.ssl_root_cert_from_pem(bytes);
            }
            ProxyCertificateInput::File(ref path) => {
                connect_options = connect_options.ssl_root_cert(path);
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

    if let Some(Some(ref application_name)) = application_name {
        connect_options = connect_options.application_name(application_name);
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

    if let Some(extra_float_digits) = extra_float_digits {
        connect_options = connect_options.extra_float_digits(extra_float_digits);
    }

    if let Some(Some(options)) = options {
        connect_options = connect_options.options(options);
    }

    let pool_options = PoolOptions::from(pool_options.unwrap_or_default());

    Ok(PostgresHandle::new(name, connect_options, pool_options))
}

impl_deserialize_field!(
    PostgresHandleField,
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
    ssl_root_cert,
    ssl_client_cert,
    ssl_client_key,
    statement_cache_capacity,
    application_name,
    log_settings | log,
    extra_float_digits | float_digits,
    options,
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
postgres://
"#;

        // When
        let actual_output = serde_yml::from_str::<PostgresHandle>(input).unwrap();
        let expected_output = PostgresHandle::default();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn from_map_url() {
        // Given
        let input = r#"
url: postgres://
"#;

        // When
        let actual_output = serde_yml::from_str::<PostgresHandle>(input).unwrap();
        let expected_output = PostgresHandle::default();

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
        let actual_output = serde_yml::from_str::<PostgresHandle>(input).unwrap();
        let expected_output =
            PostgresHandle::default().recreate_with_connect_options(|connect_options| {
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
ssl_mode: verify_full
ssl_root_cert: /etc/ssl/certs/ca-certificates.crt
ssl_client_cert: /etc/ssl/certs/client-cert.pem
ssl_client_key: /etc/ssl/private/client-key.pem
statement_cache_capacity: 777
application_name: custom
log_settings:
    statements_level: error
    slow_statements_level: error
    slow_statements_duration: 2s 500ms
extra_float_digits: 3
options:
    search_path: myschema,public
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
        let actual_output = serde_yml::from_str::<PostgresHandle>(input).unwrap();
        let expected_output = PostgresHandle::default()
            .recreate_with_connect_options(|connect_options| {
                connect_options
                    .host("example.com")
                    .port(9999)
                    .username("alice")
                    .password("secret")
                    .database("candy_shop")
                    .socket("/var/run/mysqld/mysqld.sock")
                    .ssl_mode(PgSslMode::VerifyFull)
                    .ssl_root_cert("/etc/ssl/certs/ca-certificates.crt")
                    .ssl_client_cert("/etc/ssl/certs/client-cert.pem")
                    .ssl_client_key("/etc/ssl/private/client-key.pem")
                    .statement_cache_capacity(777)
                    .application_name("custom")
                    .log_statements(LevelFilter::Error)
                    .log_slow_statements(LevelFilter::Error, Duration::from_millis(2500))
                    .extra_float_digits(3)
                    .options(HashMap::from([("search_path", "myschema,public")]))
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
ssl_mode: verify_full
ssl_root_cert: ~
ssl_client_cert: ~
ssl_client_key: ~
statement_cache_capacity: 777
application_name: ~
log_settings:
    statements_level: error
    slow_statements_level: error
    slow_statements_duration: 2s 500ms
extra_float_digits: ~
options: ~
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
        let actual_output = serde_yml::from_str::<PostgresHandle>(input).unwrap();
        let expected_output = PostgresHandle::default()
            .recreate_with_connect_options(|connect_options| {
                connect_options
                    .host("example.com")
                    .port(9999)
                    .username("alice")
                    .ssl_mode(PgSslMode::VerifyFull)
                    .statement_cache_capacity(777)
                    .log_statements(LevelFilter::Error)
                    .log_slow_statements(LevelFilter::Error, Duration::from_millis(2500))
                    .extra_float_digits(None)
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
