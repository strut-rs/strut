use serde::de::{Error, IgnoredAny, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_value::Value;
use std::collections::BTreeMap;
use std::fmt::Formatter;
use strut_factory::impl_deserialize_field;

/// Represents the application-level configuration section that covers everything
/// related to database connectivity, primarily the instance URL and credentials
/// for all database servers that this application works with.
///
/// This config comes with a custom [`Deserialize`] implementation, to support more
/// human-oriented textual configuration.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DatabaseConfig {
    #[cfg(any(
        feature = "default-mysql",
        all(feature = "mysql", not(feature = "postgres"), not(feature = "sqlite")),
    ))]
    default_handle: crate::MySqlHandle,

    #[cfg(any(
        feature = "default-postgres",
        all(feature = "postgres", not(feature = "mysql"), not(feature = "sqlite")),
    ))]
    default_handle: crate::PostgresHandle,

    #[cfg(any(
        feature = "default-sqlite",
        all(feature = "sqlite", not(feature = "mysql"), not(feature = "postgres")),
    ))]
    default_handle: crate::SqliteHandle,

    #[cfg(feature = "mysql")]
    mysql_handles: crate::MySqlHandleCollection,

    #[cfg(feature = "postgres")]
    postgres_handles: crate::PostgresHandleCollection,

    #[cfg(feature = "sqlite")]
    sqlite_handles: crate::SqliteHandleCollection,
}

impl DatabaseConfig {
    /// Returns the default [`MySqlHandle`](crate::MySqlHandle) for this
    /// configuration.
    #[cfg(any(
        feature = "default-mysql",
        all(feature = "mysql", not(feature = "postgres"), not(feature = "sqlite")),
    ))]
    pub fn default_handle(&self) -> &crate::MySqlHandle {
        &self.default_handle
    }

    /// Returns the default [`PostgresHandle`](crate::PostgresHandle) for this
    /// configuration.
    #[cfg(any(
        feature = "default-postgres",
        all(feature = "postgres", not(feature = "mysql"), not(feature = "sqlite")),
    ))]
    pub fn default_handle(&self) -> &crate::PostgresHandle {
        &self.default_handle
    }

    /// Returns the default [`SqliteHandle`](crate::SqliteHandle) for this
    /// configuration.
    #[cfg(any(
        feature = "default-sqlite",
        all(feature = "sqlite", not(feature = "mysql"), not(feature = "postgres")),
    ))]
    pub fn default_handle(&self) -> &crate::SqliteHandle {
        &self.default_handle
    }

    /// Returns the named [`MySqlHandle`](crate::MySqlHandle)s for this
    /// configuration.
    #[cfg(feature = "mysql")]
    pub fn mysql_handles(&self) -> &crate::MySqlHandleCollection {
        &self.mysql_handles
    }

    /// Returns the named [`PostgresHandle`](crate::PostgresHandle)s for this
    /// configuration.
    #[cfg(feature = "postgres")]
    pub fn postgres_handles(&self) -> &crate::PostgresHandleCollection {
        &self.postgres_handles
    }

    /// Returns the named [`SqliteHandle`](crate::SqliteHandle)s for this
    /// configuration.
    #[cfg(feature = "sqlite")]
    pub fn sqlite_handles(&self) -> &crate::SqliteHandleCollection {
        &self.sqlite_handles
    }
}

const _: () = {
    impl<'de> Deserialize<'de> for DatabaseConfig {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_map(DatabaseConfigVisitor)
        }
    }

    struct DatabaseConfigVisitor;

    impl<'de> Visitor<'de> for DatabaseConfigVisitor {
        type Value = DatabaseConfig;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str(
                "a map of application database configuration or a string URL for default database",
            )
        }

        fn visit_str<E>(self, _value: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            #[cfg(any(
                feature = "default-mysql",
                all(feature = "mysql", not(feature = "postgres"), not(feature = "sqlite")),
            ))]
            let default_handle = crate::repr::handle::mysql::visit_url::<E>(_value, None)?;

            #[cfg(any(
                feature = "default-postgres",
                all(feature = "postgres", not(feature = "mysql"), not(feature = "sqlite")),
            ))]
            let default_handle = crate::repr::handle::postgres::visit_url::<E>(_value, None)?;

            #[cfg(any(
                feature = "default-sqlite",
                all(feature = "sqlite", not(feature = "mysql"), not(feature = "postgres")),
            ))]
            let default_handle = crate::repr::handle::sqlite::visit_url::<E>(_value, None)?;

            Ok(DatabaseConfig {
                #[cfg(any(
                    feature = "default-mysql",
                    feature = "default-postgres",
                    feature = "default-sqlite",
                    all(feature = "mysql", not(feature = "postgres"), not(feature = "sqlite")),
                    all(feature = "postgres", not(feature = "mysql"), not(feature = "sqlite")),
                    all(feature = "sqlite", not(feature = "mysql"), not(feature = "postgres")),
                ))]
                default_handle,
                ..DatabaseConfig::default()
            })
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            #[cfg(any(
                feature = "default-mysql",
                feature = "default-postgres",
                feature = "default-sqlite",
                all(feature = "mysql", not(feature = "postgres"), not(feature = "sqlite")),
                all(feature = "postgres", not(feature = "mysql"), not(feature = "sqlite")),
                all(feature = "sqlite", not(feature = "mysql"), not(feature = "postgres")),
            ))]
            let mut default_handle = None;

            #[cfg(feature = "mysql")]
            let mut mysql_handles = None;

            #[cfg(feature = "postgres")]
            let mut postgres_handles = None;

            #[cfg(feature = "sqlite")]
            let mut sqlite_handles = None;

            let mut discarded = BTreeMap::<Value, Value>::new();

            while let Some(key) = map.next_key::<Value>()? {
                let field = DatabaseConfigField::deserialize(key.clone()).map_err(Error::custom)?;

                match field {
                    #[cfg(any(
                        feature = "default-mysql",
                        feature = "default-postgres",
                        feature = "default-sqlite",
                        all(feature = "mysql", not(feature = "postgres"), not(feature = "sqlite")),
                        all(feature = "postgres", not(feature = "mysql"), not(feature = "sqlite")),
                        all(feature = "sqlite", not(feature = "mysql"), not(feature = "postgres")),
                    ))]
                    DatabaseConfigField::default_handle => {
                        field.poll(&mut map, &mut default_handle)?
                    }

                    #[cfg(not(any(
                        feature = "default-mysql",
                        feature = "default-postgres",
                        feature = "default-sqlite",
                        all(feature = "mysql", not(feature = "postgres"), not(feature = "sqlite")),
                        all(feature = "postgres", not(feature = "mysql"), not(feature = "sqlite")),
                        all(feature = "sqlite", not(feature = "mysql"), not(feature = "postgres")),
                    )))]
                    DatabaseConfigField::default_handle => map.next_value::<IgnoredAny>()?,

                    #[cfg(feature = "mysql")]
                    DatabaseConfigField::mysql_handles => {
                        field.poll(&mut map, &mut mysql_handles)?
                    }
                    #[cfg(not(feature = "mysql"))]
                    DatabaseConfigField::mysql_handles => map.next_value()?,

                    #[cfg(feature = "postgres")]
                    DatabaseConfigField::postgres_handles => {
                        field.poll(&mut map, &mut postgres_handles)?
                    }
                    #[cfg(not(feature = "postgres"))]
                    DatabaseConfigField::postgres_handles => map.next_value()?,

                    #[cfg(feature = "sqlite")]
                    DatabaseConfigField::sqlite_handles => {
                        field.poll(&mut map, &mut sqlite_handles)?
                    }
                    #[cfg(not(feature = "sqlite"))]
                    DatabaseConfigField::sqlite_handles => map.next_value()?,

                    DatabaseConfigField::__ignore => {
                        discarded.insert(key, map.next_value()?);
                        IgnoredAny
                    }
                };
            }

            #[cfg(any(
                feature = "default-mysql",
                all(feature = "mysql", not(feature = "postgres"), not(feature = "sqlite")),
            ))]
            if default_handle.is_none() {
                default_handle = Some(
                    crate::MySqlHandle::deserialize(Value::Map(discarded))
                        .map_err(Error::custom)?,
                );
            }

            #[cfg(any(
                feature = "default-postgres",
                all(feature = "postgres", not(feature = "mysql"), not(feature = "sqlite")),
            ))]
            if default_handle.is_none() {
                default_handle = Some(
                    crate::PostgresHandle::deserialize(Value::Map(discarded))
                        .map_err(Error::custom)?,
                );
            }

            #[cfg(any(
                feature = "default-sqlite",
                all(feature = "sqlite", not(feature = "mysql"), not(feature = "postgres")),
            ))]
            if default_handle.is_none() {
                default_handle = Some(
                    crate::SqliteHandle::deserialize(Value::Map(discarded))
                        .map_err(Error::custom)?,
                );
            }

            Ok(DatabaseConfig {
                #[cfg(any(
                    feature = "default-mysql",
                    feature = "default-postgres",
                    feature = "default-sqlite",
                    all(feature = "mysql", not(feature = "postgres"), not(feature = "sqlite")),
                    all(feature = "postgres", not(feature = "mysql"), not(feature = "sqlite")),
                    all(feature = "sqlite", not(feature = "mysql"), not(feature = "postgres")),
                ))]
                default_handle: default_handle.unwrap_or_default(),

                #[cfg(feature = "mysql")]
                mysql_handles: mysql_handles.unwrap_or_default(),

                #[cfg(feature = "postgres")]
                postgres_handles: postgres_handles.unwrap_or_default(),

                #[cfg(feature = "sqlite")]
                sqlite_handles: sqlite_handles.unwrap_or_default(),
            })
        }
    }

    impl_deserialize_field!(
        DatabaseConfigField,
        strut_deserialize::Slug::eq_as_slugs,
        default_handle | default,
        mysql_handles | mysql,
        postgres_handles | postgres | pg | postgre_sql | postgresql,
        sqlite_handles | sqlite,
    );
};

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn empty_input() {
        // Given
        let input = r#"
"#;
        let expected_output = DatabaseConfig::default();

        // When
        let actual_output = serde_yml::from_str::<DatabaseConfig>(input).unwrap();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    #[cfg(any(
        feature = "default-mysql",
        all(feature = "mysql", not(feature = "postgres"), not(feature = "sqlite")),
    ))]
    fn default_mysql_url() {
        // Given
        let input = r#"
url: mysql://alice:secret@example.com:9999/candy_shop
"#;
        let expected_output = DatabaseConfig {
            default_handle: make_test_mysql(None),
            ..DatabaseConfig::default()
        };

        // When
        let actual_output = serde_yml::from_str::<DatabaseConfig>(input).unwrap();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    #[cfg(any(
        feature = "default-mysql",
        all(feature = "mysql", not(feature = "postgres"), not(feature = "sqlite")),
    ))]
    fn default_mysql_exploded() {
        // Given
        let input = r#"
host: example.com
port: 9999
username: alice
password: secret
database: candy_shop
"#;
        let expected_output = DatabaseConfig {
            default_handle: make_test_mysql(None),
            ..DatabaseConfig::default()
        };

        // When
        let actual_output = serde_yml::from_str::<DatabaseConfig>(input).unwrap();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    #[cfg(any(
        feature = "default-postgres",
        all(feature = "postgres", not(feature = "mysql"), not(feature = "sqlite")),
    ))]
    fn default_postgres_url() {
        // Given
        let input = r#"
url: postgres://alice:secret@example.com:9999/candy_shop
"#;
        let expected_output = DatabaseConfig {
            default_handle: make_test_postgres(None),
            ..DatabaseConfig::default()
        };

        // When
        let actual_output = serde_yml::from_str::<DatabaseConfig>(input).unwrap();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    #[cfg(any(
        feature = "default-postgres",
        all(feature = "postgres", not(feature = "mysql"), not(feature = "sqlite")),
    ))]
    fn default_postgres_exploded() {
        // Given
        let input = r#"
host: example.com
port: 9999
username: alice
password: secret
database: candy_shop
"#;
        let expected_output = DatabaseConfig {
            default_handle: make_test_postgres(None),
            ..DatabaseConfig::default()
        };

        // When
        let actual_output = serde_yml::from_str::<DatabaseConfig>(input).unwrap();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    #[cfg(any(
        feature = "default-sqlite",
        all(feature = "sqlite", not(feature = "mysql"), not(feature = "postgres")),
    ))]
    fn default_sqlite_url() {
        // Given
        let input = r#"
url: sqlite://file.db
"#;
        let expected_output = DatabaseConfig {
            default_handle: make_test_sqlite(None),
            ..DatabaseConfig::default()
        };

        // When
        let actual_output = serde_yml::from_str::<DatabaseConfig>(input).unwrap();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    #[cfg(any(
        feature = "default-sqlite",
        all(feature = "sqlite", not(feature = "mysql"), not(feature = "postgres")),
    ))]
    fn default_sqlite_exploded() {
        // Given
        let input = r#"
filename: file.db
"#;
        let expected_output = DatabaseConfig {
            default_handle: make_test_sqlite(None),
            ..DatabaseConfig::default()
        };

        // When
        let actual_output = serde_yml::from_str::<DatabaseConfig>(input).unwrap();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    #[cfg(feature = "mysql")]
    fn mysql() {
        // Given
        let input = r#"
mysql:
    mysql_a: mysql://alice:secret@example.com:9999/candy_shop
    mysql_b:
        host: example.com
        port: 9999
        username: alice
        password: secret
        database: candy_shop
"#;
        let expected_output = DatabaseConfig {
            mysql_handles: crate::MySqlHandleCollection::from([
                ("mysql_a", make_test_mysql(Some("mysql_a"))),
                ("mysql_b", make_test_mysql(Some("mysql_b"))),
            ]),
            ..DatabaseConfig::default()
        };

        // When
        let actual_output = serde_yml::from_str::<DatabaseConfig>(input).unwrap();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    #[cfg(feature = "postgres")]
    fn postgres() {
        // Given
        let input = r#"
postgres:
    postgres_a: postgres://alice:secret@example.com:9999/candy_shop
    postgres_b:
        host: example.com
        port: 9999
        username: alice
        password: secret
        database: candy_shop
"#;
        let expected_output = DatabaseConfig {
            postgres_handles: crate::PostgresHandleCollection::from([
                ("postgres_a", make_test_postgres(Some("postgres_a"))),
                ("postgres_b", make_test_postgres(Some("postgres_b"))),
            ]),
            ..DatabaseConfig::default()
        };

        // When
        let actual_output = serde_yml::from_str::<DatabaseConfig>(input).unwrap();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    #[cfg(feature = "sqlite")]
    fn sqlite() {
        // Given
        let input = r#"
sqlite:
    sqlite_a: sqlite://file.db
    sqlite_b:
        filename: file.db
"#;
        let expected_output = DatabaseConfig {
            sqlite_handles: crate::SqliteHandleCollection::from([
                ("sqlite_a", make_test_sqlite(Some("sqlite_a"))),
                ("sqlite_b", make_test_sqlite(Some("sqlite_b"))),
            ]),
            ..DatabaseConfig::default()
        };

        // When
        let actual_output = serde_yml::from_str::<DatabaseConfig>(input).unwrap();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    #[cfg(all(feature = "mysql", feature = "postgres", feature = "sqlite"))]
    fn all_types() {
        // Given
        let input = r#"
mysql:
    mysql_a: mysql://alice:secret@example.com:9999/candy_shop
    mysql_b:
        host: example.com
        port: 9999
        username: alice
        password: secret
        database: candy_shop

postgres:
    postgres_a: postgres://alice:secret@example.com:9999/candy_shop
    postgres_b:
        host: example.com
        port: 9999
        username: alice
        password: secret
        database: candy_shop

sqlite:
    sqlite_a: sqlite://file.db
    sqlite_b:
        filename: file.db
"#;
        let expected_output = DatabaseConfig {
            mysql_handles: crate::MySqlHandleCollection::from([
                ("mysql_a", make_test_mysql(Some("mysql_a"))),
                ("mysql_b", make_test_mysql(Some("mysql_b"))),
            ]),
            postgres_handles: crate::PostgresHandleCollection::from([
                ("postgres_a", make_test_postgres(Some("postgres_a"))),
                ("postgres_b", make_test_postgres(Some("postgres_b"))),
            ]),
            sqlite_handles: crate::SqliteHandleCollection::from([
                ("sqlite_a", make_test_sqlite(Some("sqlite_a"))),
                ("sqlite_b", make_test_sqlite(Some("sqlite_b"))),
            ]),
            ..DatabaseConfig::default()
        };

        // When
        let actual_output = serde_yml::from_str::<DatabaseConfig>(input).unwrap();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[cfg(feature = "mysql")]
    fn make_test_mysql(name: Option<&str>) -> crate::MySqlHandle {
        let mut handle =
            crate::MySqlHandle::default().recreate_with_connect_options(|connect_options| {
                connect_options
                    .username("alice")
                    .password("secret")
                    .host("example.com")
                    .port(9999)
                    .database("candy_shop")
            });

        if let Some(name) = name {
            handle = handle.recreate_with_name(name);
        }

        handle
    }

    #[cfg(feature = "postgres")]
    fn make_test_postgres(name: Option<&str>) -> crate::PostgresHandle {
        let mut handle =
            crate::PostgresHandle::default().recreate_with_connect_options(|connect_options| {
                connect_options
                    .username("alice")
                    .password("secret")
                    .host("example.com")
                    .port(9999)
                    .database("candy_shop")
            });

        if let Some(name) = name {
            handle = handle.recreate_with_name(name);
        }

        handle
    }

    #[cfg(feature = "sqlite")]
    fn make_test_sqlite(name: Option<&str>) -> crate::SqliteHandle {
        let mut handle = crate::SqliteHandle::default()
            .recreate_with_connect_options(|connect_options| connect_options.filename("file.db"));

        if let Some(name) = name {
            handle = handle.recreate_with_name(name);
        }

        handle
    }
}
