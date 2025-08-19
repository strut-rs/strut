use strut::tracing::*;

#[strut::main]
async fn main() {
    error!(alert = true, "This will be also sent to Sentry");
}
