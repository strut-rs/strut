use crate::common::decoder::TrivialDecoder;
use crate::common::handle::make_rabbitmq_handle;
use crate::common::multiset::{
    Dropbox, Extractor, MessageIdExtractor, Multiset, PayloadExtractor, RoutingKeyExtractor,
};
use std::time::Duration;
use strut_rabbitmq::{Ingress, IngressBuilder, Subscriber};

pub type PayloadSubscriber = TestSubscriber<PayloadExtractor<String>>;
pub type MessageIdSubscriber = TestSubscriber<MessageIdExtractor<String>>;
pub type RoutingKeySubscriber = TestSubscriber<RoutingKeyExtractor<String>>;

const BATCH_TIMEOUT: Duration = Duration::from_millis(150);

pub async fn prepare_subscriber<F>(ingress_configurer: F) -> PayloadSubscriber
where
    F: FnOnce(IngressBuilder) -> IngressBuilder,
{
    let subscriber = prepare_subscriber_inner(ingress_configurer).await;
    let dropbox = Dropbox::new_payload();

    TestSubscriber {
        subscriber,
        dropbox,
    }
}

pub async fn prepare_subscriber_with<F, X>(
    ingress_configurer: F,
    dropbox: Dropbox<X>,
) -> TestSubscriber<X>
where
    F: FnOnce(IngressBuilder) -> IngressBuilder,
    X: Extractor<Payload = String>,
{
    let subscriber = prepare_subscriber_inner(ingress_configurer).await;

    TestSubscriber {
        subscriber,
        dropbox,
    }
}

async fn prepare_subscriber_inner<F>(ingress_configurer: F) -> Subscriber<String, TrivialDecoder>
where
    F: FnOnce(IngressBuilder) -> IngressBuilder,
{
    // Get a connection handle
    let handle = make_rabbitmq_handle();

    // Make an ingress
    let mut ingress_builder = Ingress::builder();
    ingress_builder = ingress_builder.with_batch_timeout(BATCH_TIMEOUT);
    ingress_builder = ingress_configurer(ingress_builder);
    let ingress = ingress_builder.build().unwrap();

    // Start a subscriber
    let subscriber = Subscriber::start(&handle, ingress, TrivialDecoder);

    // Issue declarations
    subscriber.declare().await;

    subscriber
}

pub struct TestSubscriber<X>
where
    X: Extractor<Payload = String>,
{
    subscriber: Subscriber<String, TrivialDecoder>,
    dropbox: Dropbox<X>,
}

impl<X> TestSubscriber<X>
where
    X: Extractor<Payload = String>,
{
    pub async fn receive(&self) -> String {
        let envelope = self.subscriber.receive().await;
        let payload = envelope.payload().to_string();
        envelope.complete().await;

        payload
    }

    pub async fn ingest(&self) {
        self.dropbox.add(self.subscriber.receive().await).await;
    }

    pub async fn receive_many(&self) -> Multiset<String> {
        let dropbox = Dropbox::new_payload();
        dropbox.add_many(self.subscriber.receive_many().await).await;

        dropbox.to_multiset()
    }

    pub async fn ingest_many(&self) {
        self.dropbox
            .add_many(self.subscriber.receive_many().await)
            .await;
    }

    pub fn finalize(self) -> Multiset<X::Extracted> {
        self.dropbox.to_multiset()
    }
}
