use strut::database::sqlx::*;
use strut::Database;

#[strut::main]
async fn main() {
    let default_mysql: Pool<MySql> = Database::default(); // assuming MySQL/MariaDB is the default type

    let named_mysql: Pool<MySql> = Database::mysql("named_mysql");
    let named_postgres: Pool<Postgres> = Database::postgres("named_postgres");
    let named_sqlite: Pool<Sqlite> = Database::sqlite("named_sqlite");
}
