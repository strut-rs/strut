mod common;

#[cfg(test)]
mod tests {
    use crate::common::multiset::multiset;
    use crate::common::names::{mangle, random_token};
    use crate::common::publisher::{prepare_publisher, TestPublisher};
    use crate::common::subscriber::{prepare_subscriber, PayloadSubscriber};
    use crate::common::util::non_zero;
    use pretty_assertions::assert_eq;
    use std::any::type_name_of_val;
    use strut_rabbitmq::{strut_shutdown, Exchange};

    #[tokio::test]
    #[ignore]
    async fn publish_receive_many() {
        // Given
        let payload_a = random_token();
        let payload_b = random_token();
        let queue = mangle(type_name_of_val(&publish_receive_many));
        let publisher = make_publisher(&queue);
        let subscriber = make_subscriber(&queue).await;

        // When
        publisher.publish(&payload_a).await;
        publisher.publish(&payload_b).await;
        let received = subscriber.receive_many().await;

        // Then
        assert_eq!(received, multiset(&[payload_a, payload_b]));

        // Finally
        strut_shutdown().await;
    }

    #[tokio::test]
    #[ignore]
    async fn publish_many_receive_many() {
        // Given
        let payload_a = random_token();
        let payload_b = random_token();
        let queue = mangle(type_name_of_val(&publish_many_receive_many));
        let publisher = make_publisher(&queue);
        let subscriber = make_subscriber(&queue).await;

        // When
        publisher.publish_many(vec![&payload_a, &payload_b]).await;
        let received = subscriber.receive_many().await;

        // Then
        assert_eq!(received, multiset(&[payload_a, payload_b]));

        // Finally
        strut_shutdown().await;
    }

    fn make_publisher(routing_key: &str) -> TestPublisher {
        prepare_publisher(|egress| {
            egress
                .with_exchange(Exchange::Default.name())
                .with_routing_key(routing_key)
        })
    }

    async fn make_subscriber(queue: &str) -> PayloadSubscriber {
        prepare_subscriber(|ingress| {
            ingress
                .with_exchange(Exchange::Default)
                .with_queue_named(queue)
                .with_prefetch_count(Some(non_zero(2)))
                .with_batch_size(non_zero(2))
        })
        .await
    }
}
