use crate::common::handle::make_rabbitmq_handle;
use crate::common::names::{HEADER_KEY_A, HEADER_KEY_B};
use strut_rabbitmq::{
    BatchPublishingResult, Dispatch, Egress, EgressBuilder, Publisher, PublishingResult,
};

pub fn prepare_publisher<F>(egress_configurer: F) -> TestPublisher
where
    F: FnOnce(EgressBuilder) -> EgressBuilder,
{
    // Get a connection handle
    let handle = make_rabbitmq_handle();

    // Make an egress
    let mut egress_builder = Egress::builder();
    egress_builder = egress_configurer(egress_builder);
    let egress = egress_builder.build().unwrap();

    // Start a publisher
    let publisher = Publisher::start(&handle, egress);

    TestPublisher { publisher }
}

pub struct TestPublisher {
    publisher: Publisher,
}

impl TestPublisher {
    pub async fn try_publish(&self, payload: &str) -> PublishingResult {
        self.publisher
            .try_publish(Dispatch::from_byte_ref(payload))
            .await
    }

    pub async fn try_publish_dispatch(&self, dispatch: Dispatch) -> PublishingResult {
        self.publisher.try_publish(dispatch).await
    }

    pub async fn try_publish_header(&self, header: &str, payload: &str) -> PublishingResult {
        self.publisher
            .try_publish(
                Dispatch::builder()
                    .with_byte_ref(payload)
                    .with_header(HEADER_KEY_A, header)
                    .build(),
            )
            .await
    }

    pub async fn try_publish_headers(
        &self,
        header_a: &str,
        header_b: u32,
        payload: &str,
    ) -> PublishingResult {
        self.publisher
            .try_publish(
                Dispatch::builder()
                    .with_byte_ref(payload)
                    .with_header(HEADER_KEY_A, header_a)
                    .with_header(HEADER_KEY_B, header_b)
                    .build(),
            )
            .await
    }

    pub async fn publish(&self, payload: &str) {
        self.publisher.publish(payload).await;
    }

    pub async fn try_publish_many(&self, payloads: Vec<&str>) -> BatchPublishingResult {
        self.publisher.try_publish_many(payloads).await
    }

    pub async fn try_publish_many_dispatches(
        &self,
        dispatches: Vec<Dispatch>,
    ) -> BatchPublishingResult {
        self.publisher.try_publish_many(dispatches).await
    }

    pub async fn publish_many(&self, payloads: Vec<&str>) {
        self.publisher.publish_many(payloads).await;
    }
}
