mod common;

#[cfg(test)]
mod tests {
    use crate::common::multiset::multiset;
    use crate::common::names::{mangle, random_token};
    use crate::common::publisher::{prepare_publisher, TestPublisher};
    use crate::common::subscriber::{prepare_subscriber, PayloadSubscriber};
    use pretty_assertions::assert_eq;
    use std::any::type_name_of_val;
    use strut_rabbitmq::{strut_shutdown, Exchange};

    #[tokio::test]
    #[ignore]
    async fn try_publish() {
        // Given
        let payload = random_token();
        let queue = mangle(type_name_of_val(&try_publish));
        let publisher = make_publisher(&queue);
        let subscriber = make_subscriber(&queue).await;

        // When
        publisher.try_publish(&payload).await.unwrap();
        let received = subscriber.receive().await;

        // Then
        assert_eq!(received, payload);

        // Finally
        strut_shutdown().await;
    }

    #[tokio::test]
    #[ignore]
    async fn publish() {
        // Given
        let payload = random_token();
        let queue = mangle(type_name_of_val(&publish));
        let publisher = make_publisher(&queue);
        let subscriber = make_subscriber(&queue).await;

        // When
        publisher.publish(&payload).await;
        let received = subscriber.receive().await;

        // Then
        assert_eq!(received, payload);

        // Finally
        strut_shutdown().await;
    }

    #[tokio::test]
    #[ignore]
    async fn try_publish_many() {
        // Given
        let payload_a = random_token();
        let payload_b = random_token();
        let queue = mangle(type_name_of_val(&try_publish_many));
        let publisher = make_publisher(&queue);
        let subscriber = make_subscriber(&queue).await;

        // When
        publisher
            .try_publish_many(vec![&payload_a, &payload_b])
            .await
            .unwrap();
        let received_a = subscriber.receive().await;
        let received_b = subscriber.receive().await;

        // Then
        assert_eq!(received_a, payload_a);
        assert_eq!(received_b, payload_b);

        // Finally
        strut_shutdown().await;
    }

    #[tokio::test]
    #[ignore]
    async fn publish_many() {
        // Given
        let payload_a = random_token();
        let payload_b = random_token();
        let queue = mangle(type_name_of_val(&publish_many));
        let publisher = make_publisher(&queue);
        let subscriber = make_subscriber(&queue).await;

        // When
        publisher.publish_many(vec![&payload_a, &payload_b]).await;
        let received_a = subscriber.receive().await;
        let received_b = subscriber.receive().await;

        // Then
        assert_eq!(received_a, payload_a);
        assert_eq!(received_b, payload_b);

        // Finally
        strut_shutdown().await;
    }

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
        let received_a = subscriber.receive_many().await;
        let received_b = subscriber.receive_many().await;

        // Then
        assert_eq!(received_a, multiset(&[payload_a]));
        assert_eq!(received_b, multiset(&[payload_b]));

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
        let received_a = subscriber.receive_many().await;
        let received_b = subscriber.receive_many().await;

        // Then
        assert_eq!(received_a, multiset(&[payload_a]));
        assert_eq!(received_b, multiset(&[payload_b]));

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
        })
        .await
    }
}
