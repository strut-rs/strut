use strut::AppConfig;

#[strut::main]
async fn main() {
    let config = AppConfig::get();

    println!(
        "{:?}",
        config.database().mysql_handles().expect("MyDataBase"),
    );
    println!(
        "{:?}",
        config.database().mysql_handles().expect("OTHER_DATABASE"),
    );
}
