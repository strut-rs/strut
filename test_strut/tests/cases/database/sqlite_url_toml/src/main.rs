use strut::Database;

#[strut::main]
async fn main() {
    Database::default();
    Database::sqlite("named_sqlite_a");
    Database::sqlite("named_sqlite_b");
}
