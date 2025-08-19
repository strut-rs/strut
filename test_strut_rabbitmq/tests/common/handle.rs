use strut_rabbitmq::{DsnChunks, Handle};

pub fn make_rabbitmq_handle() -> Handle {
    let port = std::env::var("RABBITMQ_PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(5672);

    Handle::new(
        "test_rabbitmq",
        DsnChunks {
            host: "localhost",
            port,
            user: "admin",
            password: "admin",
            vhost: "/",
        },
    )
}
