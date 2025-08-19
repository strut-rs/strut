mod common;

#[cfg(test)]
mod tests {
    use crate::common::multiset::multiset;
    use crate::common::names::{random_token, ExQu};
    use crate::common::publisher::{prepare_publisher, TestPublisher};
    use crate::common::subscriber::{prepare_subscriber, PayloadSubscriber};
    use crate::common::util::non_zero;
    use pretty_assertions::assert_eq;
    use std::any::type_name_of_val;
    use strut_rabbitmq::{strut_shutdown, ConfirmationLevel, Exchange, ExchangeKind};

    #[tokio::test]
    #[ignore]
    async fn builtin_single() {
        // Given
        let payload_a = random_token();
        let payload_b = random_token();
        let payload_c = random_token();
        let names = ExQu::from(type_name_of_val(&builtin_single));
        let publisher_a = make_publisher("builtin_single.alpha.topic");
        let publisher_b = make_publisher("builtin_single.alpha.other.topic");
        let publisher_c = make_publisher("builtin_single.topic");
        let subscriber = make_subscriber(&names, "builtin_single.*.topic").await;

        // When
        let result_a = publisher_a.try_publish(&payload_a).await;
        let result_b = publisher_b.try_publish(&payload_b).await;
        let result_c = publisher_c.try_publish(&payload_c).await;
        let received = subscriber.receive_many().await;

        // Then
        assert!(result_a.is_ok());
        assert!(result_b.is_err());
        assert!(result_c.is_err());
        assert_eq!(received, multiset(&[payload_a]));

        // Finally
        strut_shutdown().await;
    }

    #[tokio::test]
    #[ignore]
    async fn builtin_multiple() {
        // Given
        let payload_a = random_token();
        let payload_b = random_token();
        let payload_c = random_token();
        let payload_d = random_token();
        let names = ExQu::from(type_name_of_val(&builtin_multiple));
        let publisher_a = make_publisher("builtin_multiple.alpha.topic");
        let publisher_b = make_publisher("builtin_multiple.alpha.other.topic");
        let publisher_c = make_publisher("builtin_multiple.topic");
        let publisher_d = make_publisher("builtin_multiple_topic");
        let subscriber = make_subscriber(&names, "builtin_multiple.#.topic").await;

        // When
        let result_a = publisher_a.try_publish(&payload_a).await;
        let result_b = publisher_b.try_publish(&payload_b).await;
        let result_c = publisher_c.try_publish(&payload_c).await;
        let result_d = publisher_d.try_publish(&payload_d).await;
        let received = subscriber.receive_many().await;

        // Then
        assert!(result_a.is_ok());
        assert!(result_b.is_ok());
        assert!(result_c.is_ok());
        assert!(result_d.is_err());
        assert_eq!(received, multiset(&[payload_a, payload_b, payload_c]));

        // Finally
        strut_shutdown().await;
    }

    #[tokio::test]
    #[ignore]
    async fn custom_single() {
        // Given
        let payload_a = random_token();
        let payload_b = random_token();
        let payload_c = random_token();
        let names = ExQu::from(type_name_of_val(&custom_single));
        let publisher_a = make_custom_publisher(&names, "custom_single.alpha.topic");
        let publisher_b = make_custom_publisher(&names, "custom_single.alpha.other.topic");
        let publisher_c = make_custom_publisher(&names, "custom_single.topic");
        let subscriber = make_custom_subscriber(&names, "custom_single.*.topic").await;

        // When
        let result_a = publisher_a.try_publish(&payload_a).await;
        let result_b = publisher_b.try_publish(&payload_b).await;
        let result_c = publisher_c.try_publish(&payload_c).await;
        let received = subscriber.receive_many().await;

        // Then
        assert!(result_a.is_ok());
        assert!(result_b.is_err());
        assert!(result_c.is_err());
        assert_eq!(received, multiset(&[payload_a]));

        // Finally
        strut_shutdown().await;
    }

    #[tokio::test]
    #[ignore]
    async fn custom_multiple() {
        // Given
        let payload_a = random_token();
        let payload_b = random_token();
        let payload_c = random_token();
        let payload_d = random_token();
        let names = ExQu::from(type_name_of_val(&custom_multiple));
        let publisher_a = make_custom_publisher(&names, "custom_multiple.alpha.topic");
        let publisher_b = make_custom_publisher(&names, "custom_multiple.alpha.other.topic");
        let publisher_c = make_custom_publisher(&names, "custom_multiple.topic");
        let publisher_d = make_custom_publisher(&names, "custom_multiple_topic");
        let subscriber = make_custom_subscriber(&names, "custom_multiple.#.topic").await;

        // When
        let result_a = publisher_a.try_publish(&payload_a).await;
        let result_b = publisher_b.try_publish(&payload_b).await;
        let result_c = publisher_c.try_publish(&payload_c).await;
        let result_d = publisher_d.try_publish(&payload_d).await;
        let received = subscriber.receive_many().await;

        // Then
        assert!(result_a.is_ok());
        assert!(result_b.is_ok());
        assert!(result_c.is_ok());
        assert!(result_d.is_err());
        assert_eq!(received, multiset(&[payload_a, payload_b, payload_c]));

        // Finally
        strut_shutdown().await;
    }

    fn make_publisher(routing_key: &str) -> TestPublisher {
        prepare_publisher(|egress| {
            egress
                .with_exchange(Exchange::AmqTopic.name())
                .with_routing_key(routing_key)
                .with_confirmation(ConfirmationLevel::Routed)
        })
    }

    async fn make_subscriber(names: &ExQu, binding_key: &str) -> PayloadSubscriber {
        prepare_subscriber(|ingress| {
            ingress
                .with_exchange(Exchange::AmqTopic)
                .with_queue_named(&names.queue)
                .with_binding_key(binding_key)
                .with_prefetch_count(Some(non_zero(10)))
                .with_batch_size(non_zero(10))
        })
        .await
    }

    fn make_custom_publisher(names: &ExQu, routing_key: &str) -> TestPublisher {
        prepare_publisher(|egress| {
            egress
                .with_exchange(&names.exchange)
                .with_routing_key(routing_key)
                .with_confirmation(ConfirmationLevel::Routed)
        })
    }

    async fn make_custom_subscriber(names: &ExQu, binding_key: &str) -> PayloadSubscriber {
        prepare_subscriber(|ingress| {
            ingress
                .with_exchange(
                    Exchange::builder()
                        .with_name(&names.exchange)
                        .with_kind(ExchangeKind::Topic)
                        .build()
                        .unwrap(),
                )
                .with_queue_named(&names.queue)
                .with_binding_key(binding_key)
                .with_prefetch_count(Some(non_zero(10)))
                .with_batch_size(non_zero(10))
        })
        .await
    }
}
