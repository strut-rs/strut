mod common;

#[cfg(test)]
mod tests {
    use crate::common::multiset::multiset;
    use crate::common::names::{random_token, ExQu, HEADER_KEY_A, HIT, MISS};
    use crate::common::publisher::{prepare_publisher, TestPublisher};
    use crate::common::subscriber::{prepare_subscriber, PayloadSubscriber};
    use crate::common::util::non_zero;
    use std::any::type_name_of_val;
    use strut_rabbitmq::{strut_shutdown, ConfirmationLevel, Exchange, ExchangeKind};

    #[tokio::test]
    #[ignore]
    async fn accepted() {
        // Given
        let payload_a1 = random_token();
        let payload_b1 = random_token();
        let payload_lost = random_token();

        // Given
        let names = Names::from(type_name_of_val(&accepted));

        // Given
        let sub = make_subscriber(&names.hit, HIT).await;

        // Given
        let pub_hit = make_publisher(&names.hit, ConfirmationLevel::Accepted);
        let pub_hit_lenient = make_publisher(&names.hit, ConfirmationLevel::Transmitted);
        let pub_miss = make_publisher(&names.miss, ConfirmationLevel::Accepted);
        let pub_miss_lenient = make_publisher(&names.miss, ConfirmationLevel::Transmitted);

        // When
        let result_a1 = pub_hit.try_publish_header(HIT, &payload_a1).await;
        let result_a2 = pub_hit.try_publish_header(MISS, &payload_lost).await;
        let result_b1 = pub_hit_lenient.try_publish_header(HIT, &payload_b1).await;
        let result_b2 = pub_hit_lenient
            .try_publish_header(MISS, &payload_lost)
            .await;
        let result_c1 = pub_miss.try_publish_header(HIT, &payload_lost).await;
        let result_c2 = pub_miss.try_publish_header(MISS, &payload_lost).await;
        let result_d1 = pub_miss_lenient
            .try_publish_header(HIT, &payload_lost)
            .await;
        let result_d2 = pub_miss_lenient
            .try_publish_header(MISS, &payload_lost)
            .await;
        let received = sub.receive_many().await;

        // Then
        assert!(result_a1.is_ok());
        assert!(result_a2.is_ok());
        assert!(result_b1.is_ok());
        assert!(result_b2.is_ok());
        assert!(result_c1.is_err());
        assert!(result_c2.is_err());
        assert!(result_d1.is_ok());
        assert!(result_d2.is_ok());
        assert_eq!(received, multiset(&[payload_a1, payload_b1]));

        // Finally
        strut_shutdown().await;
    }

    #[tokio::test]
    #[ignore]
    async fn routed() {
        // Given
        let payload_a1 = random_token();
        let payload_b1 = random_token();
        let payload_lost = random_token();

        // Given
        let names = Names::from(type_name_of_val(&routed));

        // Given
        let sub = make_subscriber(&names.hit, HIT).await;

        // Given
        let pub_hit = make_publisher(&names.hit, ConfirmationLevel::Routed);
        let pub_hit_lenient = make_publisher(&names.hit, ConfirmationLevel::Accepted);
        let pub_miss = make_publisher(&names.miss, ConfirmationLevel::Routed);
        let pub_miss_lenient = make_publisher(&names.miss, ConfirmationLevel::Accepted);

        // When
        let result_a1 = pub_hit.try_publish_header(HIT, &payload_a1).await;
        let result_a2 = pub_hit.try_publish_header(MISS, &payload_lost).await;
        let result_b1 = pub_hit_lenient.try_publish_header(HIT, &payload_b1).await;
        let result_b2 = pub_hit_lenient
            .try_publish_header(MISS, &payload_lost)
            .await;
        let result_c1 = pub_miss.try_publish_header(HIT, &payload_lost).await;
        let result_c2 = pub_miss.try_publish_header(MISS, &payload_lost).await;
        let result_d1 = pub_miss_lenient
            .try_publish_header(HIT, &payload_lost)
            .await;
        let result_d2 = pub_miss_lenient
            .try_publish_header(MISS, &payload_lost)
            .await;
        let received = sub.receive_many().await;

        // Then
        assert!(result_a1.is_ok());
        assert!(result_a2.is_err());
        assert!(result_b1.is_ok());
        assert!(result_b2.is_ok());
        assert!(result_c1.is_err());
        assert!(result_c2.is_err());
        assert!(result_d1.is_err());
        assert!(result_d2.is_err());
        assert_eq!(received, multiset(&[payload_a1, payload_b1]));

        // Finally
        strut_shutdown().await;
    }

    fn make_publisher(names: &ExQu, confirmation_level: ConfirmationLevel) -> TestPublisher {
        prepare_publisher(|egress| {
            egress
                .with_exchange(&names.exchange)
                .with_routing_key(&names.queue)
                .with_confirmation(confirmation_level)
        })
    }

    async fn make_subscriber(names: &ExQu, header: &str) -> PayloadSubscriber {
        prepare_subscriber(|ingress| {
            ingress
                .with_exchange(
                    Exchange::builder()
                        .with_name(&names.exchange)
                        .with_kind(ExchangeKind::Headers)
                        .build()
                        .unwrap(),
                )
                .with_queue_named(&names.queue)
                .with_binding_header(HEADER_KEY_A, header)
                .with_prefetch_count(Some(non_zero(100)))
                .with_batch_size(non_zero(100))
        })
        .await
    }

    pub struct Names {
        pub hit: ExQu,
        pub miss: ExQu,
    }

    impl Names {
        pub fn from(v: &str) -> Self {
            Self {
                hit: ExQu::hit(v),
                miss: ExQu::miss(v),
            }
        }
    }
}
