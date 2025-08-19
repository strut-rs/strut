use strut::AppConfig;

#[strut::main]
async fn main() {
    let config = AppConfig::get();

    println!("Running {}...", config.name());
}
