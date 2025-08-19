mod common;

#[cfg(test)]
mod tests {
    use crate::common::multiset::{multiset, Dropbox, Multiset};
    use crate::common::names::{random_token, random_u32, ExQu, HEADER_KEY_A, HEADER_KEY_B};
    use crate::common::publisher::{prepare_publisher, TestPublisher};
    use crate::common::subscriber::{
        prepare_subscriber, prepare_subscriber_with, PayloadSubscriber,
    };
    use crate::common::util::non_zero;
    use pretty_assertions::assert_eq;
    use std::any::type_name_of_val;
    use strut_rabbitmq::{
        strut_shutdown, ConfirmationLevel, Dispatch, Exchange, ExchangeKind,
        HeadersMatchingBehavior,
    };

    #[tokio::test]
    #[ignore]
    async fn builtin_all() {
        // Given
        let payload_a = random_token();
        let payload_b = random_token();
        let payload_c = random_token();
        let payload_d = random_token();
        let payload_e = random_token();
        let payload_f = random_token();

        // Given
        let header_a = random_token();
        let header_b = random_u32();
        let names = ExQu::from(type_name_of_val(&builtin_all));

        // Given
        let publisher = make_publisher();
        let subscriber = make_subscriber(&names, &header_a, header_b, true).await;

        // When
        let result_a = publisher
            .try_publish_headers(&header_a, header_b, &payload_a)
            .await;
        let result_b = publisher
            .try_publish_headers(&"", header_b, &payload_b)
            .await;
        let result_c = publisher
            .try_publish_headers(&header_a, 0, &payload_c)
            .await;
        let result_d = publisher.try_publish_header(&header_a, &payload_d).await;
        let result_e = publisher.try_publish_headers(&"", 0, &payload_e).await;
        let result_f = publisher.try_publish(&payload_f).await;
        let received = subscriber.receive_many().await;

        // Then
        assert!(result_a.is_ok());
        assert!(result_b.is_err());
        assert!(result_c.is_err());
        assert!(result_d.is_err());
        assert!(result_e.is_err());
        assert!(result_f.is_err());
        assert_eq!(received, multiset(&[payload_a]));

        // Finally
        strut_shutdown().await;
    }

    #[tokio::test]
    #[ignore]
    async fn builtin_any() {
        // Given
        let payload_a = random_token();
        let payload_b = random_token();
        let payload_c = random_token();
        let payload_d = random_token();
        let payload_e = random_token();
        let payload_f = random_token();

        // Given
        let header_a = random_token();
        let header_b = random_u32();
        let names = ExQu::from(type_name_of_val(&builtin_any));

        // Given
        let publisher = make_publisher();
        let subscriber = make_subscriber(&names, &header_a, header_b, false).await;

        // When
        let result_a = publisher
            .try_publish_headers(&header_a, header_b, &payload_a)
            .await;
        let result_b = publisher
            .try_publish_headers(&"", header_b, &payload_b)
            .await;
        let result_c = publisher
            .try_publish_headers(&header_a, 0, &payload_c)
            .await;
        let result_d = publisher.try_publish_header(&header_a, &payload_d).await;
        let result_e = publisher.try_publish_headers(&"", 0, &payload_e).await;
        let result_f = publisher.try_publish(&payload_f).await;
        let received = subscriber.receive_many().await;

        // Then
        assert!(result_a.is_ok());
        assert!(result_b.is_ok());
        assert!(result_c.is_ok());
        assert!(result_d.is_ok());
        assert!(result_e.is_err());
        assert!(result_f.is_err());
        assert_eq!(
            received,
            multiset(&[payload_a, payload_b, payload_c, payload_d]),
        );

        // Finally
        strut_shutdown().await;
    }

    #[tokio::test]
    #[ignore]
    async fn custom_all() {
        // Given
        let payload_a = random_token();
        let payload_b = random_token();
        let payload_c = random_token();
        let payload_d = random_token();
        let payload_e = random_token();
        let payload_f = random_token();

        // Given
        let header_a = random_token();
        let header_b = random_u32();
        let names = ExQu::from(type_name_of_val(&custom_all));

        // Given
        let publisher = make_custom_publisher(&names);
        let subscriber = make_custom_subscriber(&names, &header_a, header_b, true).await;

        // When
        let result_a = publisher
            .try_publish_headers(&header_a, header_b, &payload_a)
            .await;
        let result_b = publisher
            .try_publish_headers(&"", header_b, &payload_b)
            .await;
        let result_c = publisher
            .try_publish_headers(&header_a, 0, &payload_c)
            .await;
        let result_d = publisher.try_publish_header(&header_a, &payload_d).await;
        let result_e = publisher.try_publish_headers(&"", 0, &payload_e).await;
        let result_f = publisher.try_publish(&payload_f).await;
        let received = subscriber.receive_many().await;

        // Then
        assert!(result_a.is_ok());
        assert!(result_b.is_err());
        assert!(result_c.is_err());
        assert!(result_d.is_err());
        assert!(result_e.is_err());
        assert!(result_f.is_err());
        assert_eq!(received, multiset(&[payload_a]));

        // Finally
        strut_shutdown().await;
    }

    #[tokio::test]
    #[ignore]
    async fn custom_any() {
        // Given
        let payload_a = random_token();
        let payload_b = random_token();
        let payload_c = random_token();
        let payload_d = random_token();
        let payload_e = random_token();
        let payload_f = random_token();

        // Given
        let header_a = random_token();
        let header_b = random_u32();
        let names = ExQu::from(type_name_of_val(&custom_any));

        // Given
        let publisher = make_custom_publisher(&names);
        let subscriber = make_custom_subscriber(&names, &header_a, header_b, false).await;

        // When
        let result_a = publisher
            .try_publish_headers(&header_a, header_b, &payload_a)
            .await;
        let result_b = publisher
            .try_publish_headers(&"", header_b, &payload_b)
            .await;
        let result_c = publisher
            .try_publish_headers(&header_a, 0, &payload_c)
            .await;
        let result_d = publisher.try_publish_header(&header_a, &payload_d).await;
        let result_e = publisher.try_publish_headers(&"", 0, &payload_e).await;
        let result_f = publisher.try_publish(&payload_f).await;
        let received = subscriber.receive_many().await;

        // Then
        assert!(result_a.is_ok());
        assert!(result_b.is_ok());
        assert!(result_c.is_ok());
        assert!(result_d.is_ok());
        assert!(result_e.is_err());
        assert!(result_f.is_err());
        assert_eq!(
            received,
            multiset(&[payload_a, payload_b, payload_c, payload_d]),
        );

        // Finally
        strut_shutdown().await;
    }

    #[tokio::test]
    #[ignore]
    async fn header_types() {
        // Given
        let names = ExQu::from(type_name_of_val(&header_types));

        // Given
        let publisher = prepare_publisher(|egress| {
            egress
                .with_exchange(&names.exchange)
                .with_confirmation(ConfirmationLevel::Routed)
        });

        // Given
        let subscriber = prepare_subscriber_with(
            |ingress| {
                ingress
                    .with_exchange(
                        Exchange::builder()
                            .with_name(&names.exchange)
                            .with_kind(ExchangeKind::Headers)
                            .build()
                            .unwrap(),
                    )
                    .with_queue_named(&names.queue)
                    .with_binding_header("test_header_a", "alpha")
                    .with_binding_header("test_header_b", true)
                    .with_binding_header("test_header_c", i32::MIN)
                    .with_binding_header("test_header_d", u32::MAX)
                    .with_headers_behavior(HeadersMatchingBehavior::All)
                    .with_prefetch_count(Some(non_zero(10)))
                    .with_batch_size(non_zero(10))
            },
            Dropbox::new_message_id(),
        )
        .await;

        // Given
        let dispatch_a = make_dispatch(0, "alpha", true, i32::MIN, u32::MAX);
        let dispatch_b = make_dispatch(1, "bravo", true, i32::MIN, u32::MAX);
        let dispatch_c = make_dispatch(2, "alpha", false, i32::MIN, u32::MAX);
        let dispatch_d = make_dispatch(3, "alpha", true, i32::MIN + 1, u32::MAX);
        let dispatch_e = make_dispatch(4, "alpha", true, i32::MIN, u32::MAX - 1);

        // When
        let result_a = publisher.try_publish_dispatch(dispatch_a).await;
        let result_b = publisher.try_publish_dispatch(dispatch_b).await;
        let result_c = publisher.try_publish_dispatch(dispatch_c).await;
        let result_d = publisher.try_publish_dispatch(dispatch_d).await;
        let result_e = publisher.try_publish_dispatch(dispatch_e).await;
        subscriber.ingest_many().await;
        let received = subscriber.finalize();

        // When
        let expected_message_ids = vec![0].into_iter().collect::<Multiset<u64>>();

        // Then
        assert!(result_a.is_ok());
        assert!(result_b.is_err());
        assert!(result_c.is_err());
        assert!(result_d.is_err());
        assert!(result_e.is_err());
        assert_eq!(expected_message_ids, received);
    }

    fn make_publisher() -> TestPublisher {
        prepare_publisher(|egress| {
            egress
                .with_exchange(Exchange::AmqHeaders.name())
                .with_confirmation(ConfirmationLevel::Routed)
        })
    }

    async fn make_subscriber(
        names: &ExQu,
        header_a: &str,
        header_b: u32,
        match_all: bool,
    ) -> PayloadSubscriber {
        prepare_subscriber(|ingress| {
            ingress
                .with_exchange(Exchange::AmqHeaders)
                .with_queue_named(&names.queue)
                .with_binding_header(HEADER_KEY_A, header_a)
                .with_binding_header(HEADER_KEY_B, header_b)
                .with_headers_behavior(if match_all {
                    HeadersMatchingBehavior::All
                } else {
                    HeadersMatchingBehavior::Any
                })
                .with_prefetch_count(Some(non_zero(10)))
                .with_batch_size(non_zero(10))
        })
        .await
    }

    fn make_custom_publisher(names: &ExQu) -> TestPublisher {
        prepare_publisher(|egress| {
            egress
                .with_exchange(&names.exchange)
                .with_confirmation(ConfirmationLevel::Routed)
        })
    }

    async fn make_custom_subscriber(
        names: &ExQu,
        header_a: &str,
        header_b: u32,
        match_all: bool,
    ) -> PayloadSubscriber {
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
                .with_binding_header(HEADER_KEY_A, header_a)
                .with_binding_header(HEADER_KEY_B, header_b)
                .with_headers_behavior(if match_all {
                    HeadersMatchingBehavior::All
                } else {
                    HeadersMatchingBehavior::Any
                })
                .with_prefetch_count(Some(non_zero(10)))
                .with_batch_size(non_zero(10))
        })
        .await
    }

    fn make_dispatch(id: u64, a: &str, b: bool, c: i32, d: u32) -> Dispatch {
        Dispatch::builder()
            .with_message_id(id)
            .with_header("test_header_a", a)
            .with_header("test_header_b", b)
            .with_header("test_header_c", c)
            .with_header("test_header_d", d)
            .build()
    }
}
