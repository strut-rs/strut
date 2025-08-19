mod common;

#[cfg(test)]
mod tests {
    use crate::common::multiset::Dropbox;
    use crate::common::names::ExQu;
    use crate::common::publisher::{prepare_publisher, TestPublisher};
    use crate::common::subscriber::{
        prepare_subscriber_with, MessageIdSubscriber, RoutingKeySubscriber,
    };
    use crate::common::util::non_zero;
    use std::any::type_name_of_val;
    use std::sync::atomic::AtomicUsize;
    use strut_rabbitmq::{strut_shutdown, ConfirmationLevel, Dispatch, Exchange, ExchangeKind};

    const MESSAGE_PERIOD: usize = 50; // 50 different routing keys / message IDs
    const MESSAGE_COUNT: usize = 2500; // 50 repeated messages for each routing key / message ID

    #[tokio::test]
    #[ignore]
    async fn key() {
        // Given
        let names = ExQu::from(type_name_of_val(&key));

        // Given
        let publisher = make_publisher(&names);
        let subscriber_a = make_key_subscriber(&names).await;
        let subscriber_b = make_key_subscriber(&names).await;
        let subscriber_c = make_key_subscriber(&names).await;

        // When
        let dispatches = make_key_dispatches();
        let result = publisher.try_publish_many_dispatches(dispatches).await;
        assert!(result.is_ok());

        // When
        tokio::join!(
            subscriber_a.ingest_many(),
            subscriber_b.ingest_many(),
            subscriber_c.ingest_many(),
        );

        // When
        let multiset_a = subscriber_a.finalize();
        let multiset_b = subscriber_b.finalize();
        let multiset_c = subscriber_c.finalize();
        let count = multiset_a.count() + multiset_b.count() + multiset_c.count();

        // Then
        assert_eq!(count, MESSAGE_COUNT);
        assert!(multiset_a.is_disjoint_from(&multiset_b));
        assert!(multiset_b.is_disjoint_from(&multiset_c));
        assert!(multiset_c.is_disjoint_from(&multiset_a));

        // Finally
        strut_shutdown().await;
    }

    #[tokio::test]
    #[ignore]
    async fn id() {
        // Given
        let names = ExQu::from(type_name_of_val(&id));

        // Given
        let publisher = make_publisher(&names);
        let subscriber_a = make_id_subscriber(&names).await;
        let subscriber_b = make_id_subscriber(&names).await;
        let subscriber_c = make_id_subscriber(&names).await;

        // When
        let dispatches = make_id_dispatches();
        let result = publisher.try_publish_many_dispatches(dispatches).await;
        assert!(result.is_ok());

        // When
        tokio::join!(
            subscriber_a.ingest_many(),
            subscriber_b.ingest_many(),
            subscriber_c.ingest_many(),
        );

        // When
        let multiset_a = subscriber_a.finalize();
        let multiset_b = subscriber_b.finalize();
        let multiset_c = subscriber_c.finalize();
        let count = multiset_a.count() + multiset_b.count() + multiset_c.count();

        // Then
        assert_eq!(count, MESSAGE_COUNT);
        assert!(multiset_a.is_disjoint_from(&multiset_b));
        assert!(multiset_b.is_disjoint_from(&multiset_c));
        assert!(multiset_c.is_disjoint_from(&multiset_a));

        // Finally
        strut_shutdown().await;
    }

    fn make_publisher(names: &ExQu) -> TestPublisher {
        prepare_publisher(|egress| {
            egress
                .with_exchange(&names.exchange)
                .with_routing_key("unused") // we’ll specify routing key per dispatch
                .with_confirmation(ConfirmationLevel::Routed)
        })
    }

    async fn make_key_subscriber(names: &ExQu) -> RoutingKeySubscriber {
        prepare_subscriber_with(
            |ingress| {
                ingress
                    .with_exchange(
                        Exchange::builder()
                            .with_name(&names.exchange)
                            .with_kind(ExchangeKind::HashKey)
                            .build()
                            .unwrap(),
                    )
                    .with_queue_named(next_queue_name(names))
                    .with_prefetch_count(Some(non_zero(MESSAGE_COUNT as u16)))
                    .with_batch_size(non_zero(MESSAGE_COUNT))
            },
            Dropbox::new_routing_key(),
        )
        .await
    }

    async fn make_id_subscriber(names: &ExQu) -> MessageIdSubscriber {
        prepare_subscriber_with(
            |ingress| {
                ingress
                    .with_exchange(
                        Exchange::builder()
                            .with_name(&names.exchange)
                            .with_kind(ExchangeKind::HashId)
                            .build()
                            .unwrap(),
                    )
                    .with_queue_named(next_queue_name(names))
                    .with_prefetch_count(Some(non_zero(MESSAGE_COUNT as u16)))
                    .with_batch_size(non_zero(MESSAGE_COUNT))
            },
            Dropbox::new_message_id(),
        )
        .await
    }

    fn next_queue_name(names: &ExQu) -> String {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let queue_name = format!(
            "{}.{}",
            &names.queue,
            COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        );

        queue_name
    }

    fn make_key_dispatches() -> Vec<Dispatch> {
        let mut dispatches = Vec::with_capacity(MESSAGE_COUNT);
        for i in 0..MESSAGE_COUNT {
            let routing_key = format!("routing_key_{}", i % MESSAGE_PERIOD);
            let dispatch = Dispatch::builder().with_routing_key(routing_key).build();
            dispatches.push(dispatch);
        }
        dispatches
    }

    fn make_id_dispatches() -> Vec<Dispatch> {
        let mut dispatches = Vec::with_capacity(MESSAGE_COUNT);
        for i in 0..MESSAGE_COUNT {
            let dispatch = Dispatch::builder()
                .with_message_id(i % MESSAGE_PERIOD)
                .build();
            dispatches.push(dispatch);
        }
        dispatches
    }
}
