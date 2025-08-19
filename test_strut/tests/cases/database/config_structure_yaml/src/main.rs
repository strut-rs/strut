use strut::Database;

#[strut::main]
async fn main() {
    Database::default();
    Database::mysql("named_mysql_a");
    Database::mysql("named_mysql_b");
    Database::postgres("named_postgres_a");
    Database::postgres("named_postgres_b");
    Database::sqlite("named_sqlite_a");
    Database::sqlite("named_sqlite_b");
}
