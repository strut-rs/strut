use strut::AppConfig;

#[strut::main]
async fn main() {
    AppConfig::get().sentry();
}
