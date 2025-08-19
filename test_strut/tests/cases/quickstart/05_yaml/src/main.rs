use strut::tracing::info;
use strut::AppConfig;

#[strut::main]
async fn main() {
    let config = AppConfig::get();

    info!("Running {}...", config.name());
}
