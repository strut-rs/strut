use strut::{AppConfig, RabbitMq};

#[strut::main]
async fn main() {
    // Retrieve handles
    let _default_handle = AppConfig::get().rabbitmq().default_handle();
    let _named_handle = AppConfig::get()
        .rabbitmq()
        .extra_handles()
        .expect("named_handle");

    // Make publisher and subscriber
    let _publisher = RabbitMq::publisher("named_publisher");
    let _subscriber = RabbitMq::undecoded_subscriber("named_subscriber");
}
