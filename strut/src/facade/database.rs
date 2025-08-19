#[cfg(feature = "database-mysql")]
use strut_database::sqlx::MySql;
#[cfg(feature = "database-postgres")]
use strut_database::sqlx::Postgres;
#[cfg(feature = "database-sqlite")]
use strut_database::sqlx::Sqlite;
#[cfg(any(
    feature = "database-mysql",
    feature = "database-postgres",
    feature = "database-sqlite",
))]
use {
    crate::AppConfig,
    parking_lot::Mutex as SyncMutex,
    std::collections::HashMap,
    std::sync::OnceLock,
    strut_database::{sqlx::Pool, Connector},
};

/// Provides access to globally shared database connection pools.
///
/// This facade allows you to retrieve connection pools that are configured in the
/// [`DatabaseConfig`] section of your application configuration.
///
/// ## Usage
///
/// - To get the default connection pool, use [`Database::default`]. The default
///   database type is determined by Strut's feature flags.
/// - To get a specific named connection pool, use type-specific methods like
///   [`Database::mysql`], [`Database::postgres`], or [`Database::sqlite`].
///
/// Connection pools are initialized lazily on their first access and are then
/// cached for the application's lifetime. Subsequent requests for the same pool
/// return a clone of the cached one, making access efficient.
///
/// [`DatabaseConfig`]: strut_database::DatabaseConfig
/// [`Database::default`]: Database::default
/// [`Database::mysql`]: Database::mysql
/// [`Database::postgres`]: Database::postgres
/// [`Database::sqlite`]: Database::sqlite
#[cfg(any(
    feature = "database-mysql",
    feature = "database-postgres",
    feature = "database-sqlite",
))]
pub struct Database;

/// Implements retrieval of the default database connection [`Pool`] for cases
/// where [MySQL](strut_database::sqlx::MySql) is the default.
#[cfg(any(
    feature = "database-default-mysql",
    all(
        feature = "database-mysql",
        not(feature = "database-postgres"),
        not(feature = "database-sqlite"),
    ),
))]
impl Database {
    /// Returns the default database connection pool.
    ///
    /// The pool is initialized on the first call using the default connection
    /// details from the application configuration. It is then cached globally for
    /// the application's lifetime.
    ///
    /// Subsequent calls efficiently return a clone of the cached pool. The
    /// function signature indicates the database driver (`MySql`, `Postgres`, etc.)
    /// determined by Strut's feature flags.
    pub fn default() -> Pool<MySql> {
        static POOL: OnceLock<Pool<MySql>> = OnceLock::new();

        POOL.get_or_init(Self::start_default).clone()
    }

    /// Starts up a fresh [`Connector`] for the default
    /// [MySQL](MySql) database in the background.
    fn start_default() -> Pool<MySql> {
        let handle = AppConfig::get().database().default_handle().clone();

        Connector::start(handle)
    }
}

/// Implements retrieval of the default database connection [`Pool`] for cases
/// where [PostgreSQL](Postgres) is the default.
#[cfg(any(
    feature = "database-default-postgres",
    all(
        feature = "database-postgres",
        not(feature = "database-mysql"),
        not(feature = "database-sqlite"),
    ),
))]
impl Database {
    /// Returns the default database connection pool.
    ///
    /// The pool is initialized on the first call using the default connection
    /// details from the application configuration. It is then cached globally for
    /// the application's lifetime.
    ///
    /// Subsequent calls efficiently return a clone of the cached pool. The
    /// function signature indicates the database driver (`MySql`, `Postgres`, etc.)
    /// determined by Strut's feature flags.
    pub fn default() -> Pool<Postgres> {
        static POOL: OnceLock<Pool<Postgres>> = OnceLock::new();

        POOL.get_or_init(Self::start_default).clone()
    }

    /// Starts up a fresh [`Connector`] for the default [PostgreSQL](Postgres)
    /// database in the background.
    fn start_default() -> Pool<Postgres> {
        let handle = AppConfig::get().database().default_handle().clone();

        Connector::start(handle)
    }
}

/// Implements retrieval of the default database connection [`Pool`] for cases
/// where [SQLite](Sqlite) is the default.
#[cfg(any(
    feature = "database-default-sqlite",
    all(
        feature = "database-sqlite",
        not(feature = "database-mysql"),
        not(feature = "database-postgres"),
    ),
))]
impl Database {
    /// Returns the default database connection pool.
    ///
    /// The pool is initialized on the first call using the default connection
    /// details from the application configuration. It is then cached globally for
    /// the application's lifetime.
    ///
    /// Subsequent calls efficiently return a clone of the cached pool. The
    /// function signature indicates the database driver (`MySql`, `Postgres`, etc.)
    /// determined by Strut's feature flags.
    pub fn default() -> Pool<Sqlite> {
        static POOL: OnceLock<Pool<Sqlite>> = OnceLock::new();

        POOL.get_or_init(Self::start_default).clone()
    }

    /// Starts up a fresh [`Connector`] for the default [SQLite](Sqlite)
    /// database in the background.
    fn start_default() -> Pool<Sqlite> {
        let handle = AppConfig::get().database().default_handle().clone();

        Connector::start(handle)
    }
}

/// Implements retrieval of [MySQL](MySql) connection [`Pool`]s by name.
#[cfg(feature = "database-mysql")]
impl Database {
    /// Returns the connection pool for a named MySQL database.
    ///
    /// Retrieves a pool by its name as defined in the application's database
    /// configuration. Like the default pool, named pools are initialized lazily
    /// on first access for a given name and then cached for reuse.
    ///
    /// # Panics
    ///
    /// Panics if no MySQL database with the specified `name` is configured.
    pub fn mysql(name: impl AsRef<str>) -> Pool<MySql> {
        static MAP: OnceLock<SyncMutex<HashMap<String, Pool<MySql>>>> = OnceLock::new();

        let name = name.as_ref();

        MAP.get_or_init(Self::init_mysql_map)
            .lock()
            .entry(name.to_string())
            .or_insert_with(|| Self::make_mysql_pool(name))
            .clone()
    }

    /// Creates a [sync mutex](SyncMutex) containing an empty [`HashMap`].
    fn init_mysql_map() -> SyncMutex<HashMap<String, Pool<MySql>>> {
        SyncMutex::new(HashMap::new())
    }

    /// Starts up a fresh [`Connector`] for the [MySQL](MySql) database of the
    /// given `name` in the background.
    fn make_mysql_pool(name: &str) -> Pool<MySql> {
        let handle = AppConfig::get()
            .database()
            .mysql_handles()
            .expect(name)
            .clone();

        Connector::start(handle)
    }
}

/// Implements retrieval of [PostgreSQL](Postgres) connection [`Pool`]s by name.
#[cfg(feature = "database-postgres")]
impl Database {
    /// Returns the connection pool for a named PostgreSQL database.
    ///
    /// Retrieves a pool by its name as defined in the application's database
    /// configuration. Like the default pool, named pools are initialized lazily
    /// on first access for a given name and then cached for reuse.
    ///
    /// # Panics
    ///
    /// Panics if no PostgreSQL database with the specified `name` is configured.
    pub fn postgres(name: impl AsRef<str>) -> Pool<Postgres> {
        static MAP: OnceLock<SyncMutex<HashMap<String, Pool<Postgres>>>> = OnceLock::new();

        let name = name.as_ref();

        MAP.get_or_init(Self::init_postgres_map)
            .lock()
            .entry(name.to_string())
            .or_insert_with(|| Self::make_postgres_pool(name))
            .clone()
    }

    /// Creates a [sync mutex](SyncMutex) containing an empty [`HashMap`].
    fn init_postgres_map() -> SyncMutex<HashMap<String, Pool<Postgres>>> {
        SyncMutex::new(HashMap::new())
    }

    /// Starts up a fresh [`Connector`] for the [PostgreSQL](Postgres) database of the
    /// given `name` in the background.
    fn make_postgres_pool(name: &str) -> Pool<Postgres> {
        let handle = AppConfig::get()
            .database()
            .postgres_handles()
            .expect(name)
            .clone();

        Connector::start(handle)
    }
}

/// Implements retrieval of [SQLite](Sqlite) connection [`Pool`]s by name.
#[cfg(feature = "database-sqlite")]
impl Database {
    /// Returns the connection pool for a named SQLite database.
    ///
    /// Retrieves a pool by its name as defined in the application's database
    /// configuration. Like the default pool, named pools are initialized lazily
    /// on first access for a given name and then cached for reuse.
    ///
    /// # Panics
    ///
    /// Panics if no SQLite database with the specified `name` is configured.
    pub fn sqlite(name: impl AsRef<str>) -> Pool<Sqlite> {
        static MAP: OnceLock<SyncMutex<HashMap<String, Pool<Sqlite>>>> = OnceLock::new();

        let name = name.as_ref();

        MAP.get_or_init(Self::init_sqlite_map)
            .lock()
            .entry(name.to_string())
            .or_insert_with(|| Self::make_sqlite_pool(name))
            .clone()
    }

    /// Creates a [sync mutex](SyncMutex) containing an empty [`HashMap`].
    fn init_sqlite_map() -> SyncMutex<HashMap<String, Pool<Sqlite>>> {
        SyncMutex::new(HashMap::new())
    }

    /// Starts up a fresh [`Connector`] for the [SQLite](Sqlite) database of the
    /// given `name` in the background.
    fn make_sqlite_pool(name: &str) -> Pool<Sqlite> {
        let handle = AppConfig::get()
            .database()
            .sqlite_handles()
            .expect(name)
            .clone();

        Connector::start(handle)
    }
}
