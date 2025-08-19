mod common;

#[cfg(test)]
mod tests {
    use crate::common::multiset::multiset;
    use crate::common::names::{random_token, ExQu, HIT, MISS};
    use crate::common::publisher::{prepare_publisher, TestPublisher};
    use crate::common::subscriber::{prepare_subscriber, PayloadSubscriber};
    use crate::common::util::non_zero;
    use pretty_assertions::assert_eq;
    use std::any::type_name_of_val;
    use strut_rabbitmq::{strut_shutdown, ConfirmationLevel, Exchange, ExchangeKind};

    #[tokio::test]
    #[ignore]
    async fn builtin() {
        // Given
        let payload_a = random_token();
        let payload_b = random_token();
        let names = ExQu::from(type_name_of_val(&builtin));
        let publisher_a = make_publisher(HIT);
        let publisher_b = make_publisher(MISS);
        let subscriber = make_subscriber(&names).await;

        // When
        let result_a = publisher_a.try_publish(&payload_a).await;
        let result_b = publisher_b.try_publish(&payload_b).await;
        let received = subscriber.receive_many().await;

        // Then
        assert!(result_a.is_ok());
        assert!(result_b.is_err());
        assert_eq!(received, multiset(&[payload_a]));

        // Finally
        strut_shutdown().await;
    }

    #[tokio::test]
    #[ignore]
    async fn custom() {
        // Given
        let payload_a = random_token();
        let payload_b = random_token();
        let names = ExQu::from(type_name_of_val(&custom));
        let publisher_a = make_custom_publisher(&names, HIT);
        let publisher_b = make_custom_publisher(&names, MISS);
        let subscriber = make_custom_subscriber(&names).await;

        // When
        let result_a = publisher_a.try_publish(&payload_a).await;
        let result_b = publisher_b.try_publish(&payload_b).await;
        let received = subscriber.receive_many().await;

        // Then
        assert!(result_a.is_ok());
        assert!(result_b.is_err());
        assert_eq!(received, multiset(&[payload_a]));

        // Finally
        strut_shutdown().await;
    }

    fn make_publisher(routing_key: &str) -> TestPublisher {
        prepare_publisher(|egress| {
            egress
                .with_exchange(Exchange::AmqDirect.name())
                .with_routing_key(routing_key)
                .with_confirmation(ConfirmationLevel::Routed)
        })
    }

    async fn make_subscriber(names: &ExQu) -> PayloadSubscriber {
        prepare_subscriber(|ingress| {
            ingress
                .with_exchange(Exchange::AmqDirect)
                .with_queue_named(&names.queue)
                .with_binding_key(HIT)
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

    async fn make_custom_subscriber(names: &ExQu) -> PayloadSubscriber {
        prepare_subscriber(|ingress| {
            ingress
                .with_exchange(
                    Exchange::builder()
                        .with_name(&names.exchange)
                        .with_kind(ExchangeKind::Direct)
                        .build()
                        .unwrap(),
                )
                .with_queue_named(&names.queue)
                .with_binding_key(HIT)
                .with_prefetch_count(Some(non_zero(10)))
                .with_batch_size(non_zero(10))
        })
        .await
    }
}
