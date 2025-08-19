use strut::Database;

#[strut::main]
async fn main() {
    Database::default();
    Database::mysql("named_mysql_a");
    Database::mysql("named_mysql_b");
}
