use strut::database::sqlx::*;
use strut::Database;

#[strut::main]
async fn main() {
    let pool: Pool<Sqlite> = Database::default();

    // connection pool ready to use
}
