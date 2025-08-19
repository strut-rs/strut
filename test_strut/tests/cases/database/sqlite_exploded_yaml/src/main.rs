use strut::Database;

#[strut::main]
async fn main() {
    Database::default();
    Database::sqlite("named_sqlite");
}
