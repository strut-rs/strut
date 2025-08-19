use sqlx_core::connection::Connection;
use sqlx_core::database::Database;
use sqlx_core::pool::PoolOptions;

#[cfg(feature = "mysql")]
pub mod mysql;

#[cfg(feature = "postgres")]
pub mod postgres;

#[cfg(feature = "sqlite")]
pub mod sqlite;

/// Defines a connection handle for a database supported by the `sqlx` crate.
/// The handle consists primarily of a database URL and credentials, along with
/// a bit of connection options.
///
/// This handle by itself does not implement any connection logic.
pub trait Handle {
    type Database: Database;

    /// Reports the handle name.
    fn name(&self) -> &str;

    /// Reports the handle identifier, which is a short version of the database
    /// URL, without the password or connection options. This identifier is
    /// generally safe for debug logging.
    fn identifier(&self) -> &str;

    /// Returns the [`ConnectOptions`](sqlx_core::connection::ConnectOptions) of
    /// this handle.
    ///
    /// This options object needs to be consumed when establishing an `sqlx`
    /// connection [`Pool`](sqlx_core::pool::Pool), thus a reference may not be
    /// always useful. It is possible to instead [destruct](Handle::destruct)
    /// this handle into the consumed components.
    fn connect_options(&self)
    -> &<<Self::Database as Database>::Connection as Connection>::Options;

    /// Returns the [`PoolOptions`] of this handle.
    ///
    /// This options object needs to be consumed when establishing an `sqlx`
    /// connection [`Pool`](sqlx_core::pool::Pool), thus a reference may not be
    /// always useful. It is possible to instead [destruct](Handle::destruct)
    /// this handle into the consumed components.
    fn pool_options(&self) -> &PoolOptions<Self::Database>;

    /// Destructs this handle into the two connection objects that are generally
    /// needed to establish an `sqlx` connection [`Pool`](sqlx_core::pool::Pool):
    /// [`ConnectOptions`](sqlx_core::connection::ConnectOptions) and
    /// [`PoolOptions`].
    fn destruct(
        self,
    ) -> (
        <<Self::Database as Database>::Connection as Connection>::Options,
        PoolOptions<Self::Database>,
    );
}
