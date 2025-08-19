use strut::Database;

#[strut::main]
async fn main() {
    Database::default();
    Database::postgres("named_postgres_a");
    Database::postgres("named_postgres_b");
}
