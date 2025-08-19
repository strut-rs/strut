use strut::rabbitmq::*;
use strut::RabbitMq;

#[strut::main]
async fn main() {
    // Make publisher and subscriber
    let publisher: Publisher = RabbitMq::publisher("demo_egress");
    let subscriber: StringSubscriber = RabbitMq::string_subscriber("demo_ingress");

    // Ensure the queue exists before we start sending
    subscriber.declare().await;

    // Send message
    publisher.publish("Demo message").await;

    // Receive message
    let envelope: Envelope<String> = subscriber.receive().await;

    assert_eq!(envelope.payload(), "Demo message");

    // Ack message
    envelope.complete().await;
}
