use crate::transport::inbound::envelope::DecoderError;
use crate::util::Push;
use crate::{
    Connector, Decoder, Envelope, ExchangeKind, Gateway, Handle, Header, Ingress, NoopDecoder,
    StringDecoder,
};
use futures::StreamExt;
use lapin::message::Delivery;
use lapin::options::{
    BasicConsumeOptions, BasicQosOptions, ExchangeDeclareOptions, QueueBindOptions,
    QueueDeclareOptions,
};
use lapin::types::FieldTable;
use lapin::{
    Channel, Consumer as LapinConsumer, Error as LapinError, Queue as LapinQueue,
    Result as LapinResult,
};
use nonempty::NonEmpty;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use strut_util::Backoff;
use thiserror::Error;
use tokio::select;
use tokio::sync::{Mutex as AsyncMutex, MutexGuard, Notify};
use tracing::{debug, error, warn};

/// Shorthand for a [`Subscriber`] that does not decode consumed messages.
pub type UndecodedSubscriber = Subscriber<(), NoopDecoder>;

/// Shorthand for a [`Subscriber`] that decodes messages into [`String`]s.
pub type StringSubscriber = Subscriber<String, StringDecoder>;

/// Shorthand for a [`Subscriber`] that decodes messages using
/// [`JsonDecoder`](crate::JsonDecoder).
#[cfg(feature = "json")]
pub type JsonSubscriber<T> = Subscriber<T, crate::JsonDecoder<T>>;

/// Receives incoming [`Envelope`]s from the RabbitMQ cluster, passing them
/// through a pre-set [`Decoder`] before returning to the caller.
pub struct Subscriber<T, D>
where
    D: Decoder<Result = T>,
{
    name: Arc<str>,
    ingress: Ingress,
    gateway: Gateway,
    consumer: AsyncMutex<Option<LapinConsumer>>,
    decoder: D,
    batch_tail_size: usize,
}

/// Represents the outcome of polling a single message from a [`LapinConsumer`].
enum PollOutcome<T> {
    /// Successfully polled and decoded an [`Envelope`].
    Envelope(Envelope<T>),
    /// A [`LapinError`] delivered from the [`LapinConsumer`].
    ConsumerError,
    /// A message is successfully polled, but could not be decoded.
    Gibberish,
    /// The [`LapinConsumer`] is permanently out of messages and cannot be used
    /// any further.
    OutOfMessages,
}

/// Represents the outcome of assembling a batch of incoming messages.
enum BatchState {
    /// Keep polling.
    InProgress,
    /// Polled enough messages.
    Completed,
    /// Timed out: no more.
    TimedOut,
    /// [`LapinConsumer`] signaled it’s out of messages.
    DriedOut,
}

/// Represents failure to issue at least one of the declarations that are
/// required before the subscriber can start consuming messages.
///
/// The following types of declarations can potentially fail:
///
/// - Declare an exchange.
/// - Declare a queue.
/// - Bind a queue to an exchange.
///
/// The declarations are based on the [`Ingress`] definition.
#[derive(Error, Debug)]
#[error("failed to issue RabbitMQ declarations from the subscriber '{subscriber}': {error}")]
pub struct DeclarationError {
    subscriber: String,
    error: String,
}

impl<T, D> Subscriber<T, D>
where
    D: Decoder<Result = T>,
{
    /// Creates and returns a new [`Subscriber`] for the given [`Ingress`] and
    /// [`Decoder`].
    pub fn new(gateway: Gateway, ingress: Ingress, decoder: D) -> Self {
        let name = Self::compose_name(&ingress);
        let consumer = AsyncMutex::new(None);
        let batch_tail_size = usize::from(ingress.batch_size()) - 1;

        Self {
            name,
            ingress,
            gateway,
            consumer,
            decoder,
            batch_tail_size,
        }
    }

    /// Starts a new [`Connector`] with the given [`Handle`] and uses it to create
    /// and return a new [`Subscriber`] for the given [`Ingress`] and [`Decoder`].
    pub fn start(handle: impl AsRef<Handle>, ingress: Ingress, decoder: D) -> Self {
        let gateway = Connector::start(handle);

        Self::new(gateway, ingress, decoder)
    }

    /// Composes a globally unique, human-readable name for this [`Subscriber`].
    fn compose_name(ingress: &Ingress) -> Arc<str> {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);

        Arc::from(format!(
            "rabbitmq:sub:{}:{}",
            ingress.name(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        ))
    }
}

impl<T, D> Subscriber<T, D>
where
    D: Decoder<Result = T>,
{
    /// Reports the name of this [`Subscriber`].
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Subscriber<(), NoopDecoder> {
    /// A shorthand for calling [`new`](Subscriber::new) with a [`NoopDecoder`].
    pub fn new_undecoded(gateway: Gateway, ingress: Ingress) -> Self {
        Self::new(gateway, ingress, NoopDecoder)
    }

    /// A shorthand for calling [`start`](Subscriber::start) with a
    /// [`NoopDecoder`].
    pub fn start_undecoded(handle: &Handle, ingress: Ingress) -> Self {
        Self::start(handle, ingress, NoopDecoder)
    }
}

#[cfg(feature = "json")]
impl<T> Subscriber<T, crate::JsonDecoder<T>>
where
    T: serde::de::DeserializeOwned,
{
    /// A shorthand for calling [`new`](Subscriber::new) with a
    /// [`JsonDecoder`](crate::JsonDecoder).
    pub fn new_json(gateway: Gateway, ingress: Ingress) -> Self {
        Self::new(gateway, ingress, crate::JsonDecoder::default())
    }

    /// A shorthand for calling [`start`](Subscriber::start) with a
    /// [`JsonDecoder`](crate::JsonDecoder).
    pub fn start_json(handle: &Handle, ingress: Ingress) -> Self {
        Self::start(handle, ingress, crate::JsonDecoder::default())
    }
}

impl<T, D> Subscriber<T, D>
where
    D: Decoder<Result = T>,
{
    /// Waits for the connection to RabbitMQ to become available, then issues
    /// the declarations necessary for consuming messages with the [`Ingress`]
    /// configured on this subscriber. The declarations include declaring an
    /// exchange (if not a built-in exchange), declaring a queue, and binding
    /// the queue to the exchange in some way. Such declarations are repeatable
    /// (assuming the configuration options don’t change), so it shouldn’t hurt
    /// to call this method any number of times.
    ///
    /// If and when this method returns [`Ok`], it can be reasonably expected
    /// that the following calls to [`receive`](Subscriber::receive) or
    /// [`receive_many`](Subscriber::receive_many) will be able to eventually
    /// deliver incoming messages, assuming the connectivity to RabbitMQ
    /// remains.
    ///
    /// If any of the declarations fail (e.g., a queue by that name already
    /// exists with different configuration), this method returns a
    /// [`DeclarationError`].
    pub async fn try_declare(&self) -> Result<(), DeclarationError> {
        // Patiently wait for a fresh channel
        let channel = self.gateway.channel().await;

        // Try to issue declarations (exchange, queue, bindings)
        match self.issue_declarations(&channel).await {
            // Success: queue is irrelevant, we just return
            Ok(_queue) => Ok(()),

            // Error: return the error
            Err(error) => Err(DeclarationError::new(self.name.as_ref(), error)),
        }
    }

    /// Repeatedly calls [`try_declare`](Subscriber::try_declare) until it
    /// succeeds, with an exponential backoff.
    ///
    /// Most declaration errors can only be fixed outside the application, by
    /// changing the broker configuration (e.g., deleting a conflicting queue).
    /// In such cases, this method may be used, to keep the subscriber spinning
    /// (and alerting about the declaration failure) until the issue is fixed
    /// externally, at which point the declarations will eventually succeed, and
    /// this method will return.
    pub async fn declare(&self) {
        // Prepare to backoff
        let backoff = Backoff::default();

        loop {
            match self.try_declare().await {
                // Success: we just return
                Ok(()) => return,

                // Error: alert and wait a bit
                Err(error) => {
                    warn!(
                        alert = true,
                        subscriber = self.name.as_ref(),
                        ?error,
                        error_message = %error,
                        "Failed to declare an exchange or a queue",
                    );

                    backoff.sleep_next().await;
                }
            }
        }
    }

    /// Receives a single, decode-able message from the broker. Will wait as
    /// long as it takes for the first decode-able message to arrive.
    pub async fn receive(&self) -> Envelope<T> {
        // Grab the consumer (keep the guard until we return)
        let (mut consumer_guard, mut consumer) = self.grab_consumer().await;

        // Poll until an envelope is received
        let envelope = self.poll(&mut consumer).await;

        // Put the consumer back under lock
        *consumer_guard = Some(consumer);

        envelope
    }

    /// Receives a batch of up to [batch_size](Ingress::batch_size) of
    /// decode-able messages from the broker. Will wait as long as it takes for
    /// the first decode-able message to arrive, after which will take no longer
    /// than [`BATCH_TIMEOUT`] to complete the batch before returning. The final
    /// batch will thus always contain at least one message.
    pub async fn receive_many(&self) -> NonEmpty<Envelope<T>> {
        // Grab the consumer (keep the guard until the batch is assembled)
        let (mut consumer_guard, mut consumer) = self.grab_consumer().await;

        // Poll the head of the batch (the first envelope)
        let batch_head = self.poll(&mut consumer).await;

        // Now that we’ve got the first message of the batch, set a timeout for receiving the full batch

        // Create the notification mechanism for the batch timer
        let notify_in = Arc::new(Notify::new());
        let notify_out = Arc::clone(&notify_in);

        // Start the batch timer (within this time limit additional messages may fall into the same batch)
        let batch_timeout = self.ingress.batch_timeout();
        let handle = tokio::spawn(async move {
            tokio::time::sleep(batch_timeout).await;
            notify_in.notify_one();
        });

        // Prepare storage
        let mut batch_tail = Vec::with_capacity(self.batch_tail_size);

        // Complete batch within timeout
        let batch_state = self
            .complete_batch(&mut consumer, &mut batch_tail, notify_out)
            .await;

        // Check if the batch state spells success
        if batch_state.represents_healthy_consumer() {
            // Put the consumer back and release the lock
            *consumer_guard = Some(consumer);
        }

        // Drop the consumer guard
        drop(consumer_guard);

        // Whether or not the timer completed, we don’t need it anymore
        handle.abort();

        NonEmpty::from((batch_head, batch_tail))
    }
}

impl<T, D> Subscriber<T, D>
where
    D: Decoder<Result = T>,
{
    /// Infinitely polls the given [`LapinConsumer`] until it yields a
    /// decode-able [`Envelope`], then returns it. This method will re-fetch the
    /// consumer as many times as needed.
    async fn poll(&self, consumer: &mut LapinConsumer) -> Envelope<T> {
        // Keep trying until we poll the first message
        loop {
            // Try to poll a message
            let outcome = self.try_poll(consumer).await;

            // Check if we have a good envelope
            if let PollOutcome::Envelope(envelope) = outcome {
                // Good envelope: return
                return envelope;
            }

            // No luck with last outcome: go toward retrying

            // If the consumer dried out, re-fetch it
            if outcome.represents_empty_consumer() {
                *consumer = self.fetch_consumer().await;
            }
        }
    }

    /// Completes the given batch tail within the given timeout by repeatedly
    /// polling the given consumer.
    async fn complete_batch(
        &self,
        consumer: &mut LapinConsumer,
        batch_tail: &mut Vec<Envelope<T>>,
        timeout: Arc<Notify>,
    ) -> BatchState {
        // Start collecting additional messages into the batch (within both the time and the count limits)
        while batch_tail.len() < self.batch_tail_size {
            let state = select! {
                biased;
                _ = timeout.notified() => BatchState::TimedOut,
                outcome = self.try_poll(consumer) => self.receive_outcome(outcome, batch_tail),
            };

            match state {
                BatchState::InProgress => continue,
                BatchState::Completed | BatchState::TimedOut | BatchState::DriedOut => {
                    return state;
                }
            }
        }

        BatchState::Completed
    }

    /// Abstracts two asynchronous calls (next delivery from the consumer, and
    /// unwrapping of the delivery) into a single asynchronous call for convenient
    /// use in a `select` block.
    async fn try_poll(&self, consumer: &mut LapinConsumer) -> PollOutcome<T> {
        // Fetch and unwrap next delivery
        self.unwrap_delivery(consumer.next().await).await
    }

    /// Pushes a [`PollOutcome`] onto the given batch, and translates that outcome
    /// into the current [`BatchState`].
    fn receive_outcome(&self, outcome: PollOutcome<T>, batch: &mut Vec<Envelope<T>>) -> BatchState {
        match outcome {
            PollOutcome::Envelope(envelope) => {
                batch.push(envelope);
                BatchState::InProgress
            }
            PollOutcome::OutOfMessages => BatchState::DriedOut,
            PollOutcome::ConsumerError | PollOutcome::Gibberish => BatchState::InProgress,
        }
    }

    /// Peels the layers off the given incoming delivery.
    async fn unwrap_delivery(
        &self,
        option_delivery_result: Option<LapinResult<Delivery>>,
    ) -> PollOutcome<T> {
        // Unwrap the outer option
        let delivery_result = match option_delivery_result {
            Some(delivery_result) => delivery_result,
            None => {
                debug!(
                    subscriber = self.name.as_ref(),
                    "Ran out of messages on a RabbitMQ consumer",
                );

                return PollOutcome::OutOfMessages;
            }
        };

        // Unwrap the inner result
        let delivery = match delivery_result {
            Ok(delivery) => delivery,
            Err(error) => {
                warn!(
                    alert = true,
                    subscriber = self.name.as_ref(),
                    ?error,
                    error_message = %error,
                    "Received an error from a RabbitMQ consumer",
                );

                return PollOutcome::ConsumerError;
            }
        };

        // Decode an envelope
        let envelope_result = Envelope::try_from(
            self.name.clone(),
            &self.decoder,
            delivery,
            self.ingress.delivers_pending(),
        );

        // Inspect the result
        match envelope_result {
            Ok(envelope) => PollOutcome::Envelope(envelope),
            Err(error) => {
                self.discard_gibberish(error).await;

                PollOutcome::Gibberish
            }
        }
    }

    /// Handles and discards the given un-decodable inbound message.
    async fn discard_gibberish(&self, decoder_error: DecoderError<D>) {
        // Destruct the decoder error
        let DecoderError {
            bytes,
            mut acker,
            error,
        } = decoder_error;

        // Report the un-decodable message
        error!(
            alert = true,
            subscriber = self.name.as_ref(),
            ?error,
            error_message = %error,
            byte_preview = String::from_utf8_lossy(&bytes).as_ref(),
            "Failed to decode an inbound RabbitMQ message",
        );

        // Finalize the message
        if let Some(acker) = acker.take() {
            self.ingress
                .gibberish_behavior()
                .apply(self.name.as_ref(), &acker, &bytes)
                .await;
        }
    }

    /// Obtains the lock on the consumer and returns both the guard and the consumer
    async fn grab_consumer(&self) -> (MutexGuard<'_, Option<LapinConsumer>>, LapinConsumer) {
        // Obtain the consumer guard
        let mut consumer_guard = self.consumer.lock().await;

        // Either take the consumer or fetch a fresh one
        let consumer = match consumer_guard.take() {
            Some(consumer) => consumer,
            None => self.fetch_consumer().await,
        };

        // Return the pair
        (consumer_guard, consumer)
    }

    /// Fetches a fresh [`LapinConsumer`], building it from scratch on a fresh
    /// [`Channel`].
    async fn fetch_consumer(&self) -> LapinConsumer {
        loop {
            // Patiently wait for a fresh channel
            let channel = self.gateway.channel().await;

            // Try to build a consumer
            match self.build_consumer(&channel).await {
                // Successfully built a consumer: return
                Ok(consumer) => return consumer,

                // Failed to build a consumer for some reason: report and retry
                Err(error) => {
                    warn!(
                        alert = true,
                        subscriber = self.name.as_ref(),
                        ?error,
                        error_message = %error,
                        "Failed to build a RabbitMQ message consumer",
                    );
                }
            }
        }
    }

    /// Builds a [`LapinConsumer`] on top of the given [`Channel`].
    async fn build_consumer(&self, channel: &Channel) -> LapinResult<LapinConsumer> {
        // Declare the RabbitMQ queue
        let queue = self.issue_declarations(channel).await?;

        // Initiate consuming of messages
        channel
            .basic_consume(
                queue.name().as_str(),
                &self.name,
                BasicConsumeOptions {
                    no_local: false,
                    no_ack: self.ingress.no_ack(),
                    exclusive: false,
                    nowait: false,
                },
                FieldTable::default(),
            )
            .await
    }

    /// Declares the exchange (if necessary), the queue, and binds the queue to
    /// the exchange.
    async fn issue_declarations(&self, channel: &Channel) -> LapinResult<LapinQueue> {
        // Set prefetch count on the channel if relevant
        if let Some(prefetch_count) = self.ingress.prefetch_count() {
            channel
                .basic_qos(prefetch_count.into(), BasicQosOptions { global: false })
                .await?;
        }

        // If the exchange is not built-in, declare it first
        if self.ingress.exchange().is_custom() {
            // Extract custom exchange reference for convenience
            let exchange = self.ingress.exchange();

            // Prepare args
            let mut args = FieldTable::default();

            // Check special exchange case
            if exchange.kind() == ExchangeKind::HashId {
                // For routing on message ID, we need a custom arg
                args.push("hash-property", "message_id");
            }

            channel
                .exchange_declare(
                    exchange.name(),
                    exchange.kind().lapin_value(),
                    ExchangeDeclareOptions {
                        passive: false,
                        durable: exchange.durable(),
                        auto_delete: exchange.auto_delete(),
                        internal: false,
                        nowait: false,
                    },
                    args,
                )
                .await?;
        }

        // Explicitly indicate desired queue kind
        let mut args = FieldTable::default();
        args.push("x-queue-type", self.ingress.queue().kind().rabbitmq_value());

        // Declare the queue
        let queue = channel
            .queue_declare(
                self.ingress.queue().name().as_ref(),
                QueueDeclareOptions {
                    passive: false,
                    durable: self.ingress.durable(),
                    exclusive: self.ingress.exclusive(),
                    auto_delete: self.ingress.auto_delete(),
                    nowait: false,
                },
                args,
            )
            .await?;

        // Bind the queue to the exchange
        self.bind_queue_to_exchange(channel, &queue).await?;

        Ok(queue)
    }

    /// Binds the given [`LapinQueue`] to the exchange configured on this
    /// subscriber’s [`Ingress`].
    async fn bind_queue_to_exchange(
        &self,
        channel: &Channel,
        queue: &LapinQueue,
    ) -> LapinResult<()> {
        // Default exchange forbids any kind of binding
        if self.ingress.exchange().is_default() {
            return Ok(());
        }

        // Proceed according to the exchange kind
        match self.ingress.exchange().kind() {
            ExchangeKind::Direct | ExchangeKind::Topic => {
                self.bind_queue_to_key_exchange(channel, queue).await
            }
            ExchangeKind::Fanout => self.bind_queue_to_fanout_exchange(channel, queue).await,
            ExchangeKind::Headers => self.bind_queue_to_headers_exchange(channel, queue).await,
            ExchangeKind::HashKey | ExchangeKind::HashId => {
                self.bind_queue_to_hash_exchange(channel, queue).await
            }
        }
    }

    /// Binds the given [`LapinQueue`] to the **key-based exchange**
    /// ([direct](ExchangeKind::Direct) or [topic](ExchangeKind::Topic))
    /// configured on this subscriber’s [`Ingress`].
    async fn bind_queue_to_key_exchange(
        &self,
        channel: &Channel,
        queue: &LapinQueue,
    ) -> LapinResult<()> {
        // Bind the same queue to the same exchange once for every binding key
        for binding_key in self.ingress.binding_keys() {
            channel
                .queue_bind(
                    queue.name().as_str(),
                    self.ingress.exchange().name(),
                    binding_key,
                    QueueBindOptions { nowait: false },
                    FieldTable::default(),
                )
                .await?;
        }

        Ok(())
    }

    /// Binds the given [`LapinQueue`] to the
    /// [**fanout exchange**](ExchangeKind::Fanout) configured on this
    /// subscriber’s [`Ingress`].
    async fn bind_queue_to_fanout_exchange(
        &self,
        channel: &Channel,
        queue: &LapinQueue,
    ) -> LapinResult<()> {
        channel
            .queue_bind(
                queue.name().as_str(),
                self.ingress.exchange().name(),
                "", // Routing key is irrelevant for fanout exchanges
                QueueBindOptions { nowait: false },
                FieldTable::default(),
            )
            .await?;

        Ok(())
    }

    /// Binds the given [`LapinQueue`] to the
    /// [**headers exchange**](ExchangeKind::Headers) configured on this
    /// subscriber’s [`Ingress`].
    async fn bind_queue_to_headers_exchange(
        &self,
        channel: &Channel,
        queue: &LapinQueue,
    ) -> LapinResult<()> {
        let mut args = FieldTable::default();

        args.push("x-match", self.ingress.headers_behavior().rabbitmq_value());

        for (key, value) in self.ingress.binding_headers() {
            match value {
                Header::Boolean(value) => args.push(key, *value),
                Header::Int(value) => args.push(key, *value),
                Header::UInt(value) => args.push(key, *value),
                Header::String(value) => args.push(key, value.as_ref()),
            }
        }

        channel
            .queue_bind(
                queue.name().as_str(),
                self.ingress.exchange().name(),
                "", // Routing key is irrelevant for header-based matching
                QueueBindOptions { nowait: false },
                args,
            )
            .await?;

        Ok(())
    }

    /// Binds the given [`LapinQueue`] to the **consistent hash exchange**
    /// ([key-based](ExchangeKind::HashKey) or
    /// [message ID-based](ExchangeKind::HashId)) configured on this
    /// subscriber’s [`Ingress`].
    async fn bind_queue_to_hash_exchange(
        &self,
        channel: &Channel,
        queue: &LapinQueue,
    ) -> LapinResult<()> {
        channel
            .queue_bind(
                queue.name().as_str(),
                self.ingress.exchange().name(),
                "1", // Always bind with an equal weight of 1
                QueueBindOptions { nowait: false },
                FieldTable::default(),
            )
            .await?;

        Ok(())
    }
}

impl<T> PollOutcome<T> {
    /// Reports whether the [`LapinConsumer`] can be judged as “empty” based
    /// on this outcome alone. An empty consumer will not yield any more messages
    /// if polled.
    fn represents_empty_consumer(&self) -> bool {
        match self {
            Self::OutOfMessages => true,
            Self::Envelope(_) | Self::Gibberish | Self::ConsumerError => true,
        }
    }
}

impl BatchState {
    /// Reports whether the [`LapinConsumer`] can be judged as “healthy” based
    /// on this batch state alone. A healthy consumer is a consumer that we can
    /// still use to try and poll for more messages.
    fn represents_healthy_consumer(&self) -> bool {
        match self {
            BatchState::InProgress | BatchState::Completed | BatchState::TimedOut => true,
            BatchState::DriedOut => false,
        }
    }
}

impl<T> From<PollOutcome<T>> for Option<Envelope<T>> {
    fn from(value: PollOutcome<T>) -> Self {
        match value {
            PollOutcome::Envelope(envelope) => Some(envelope),
            PollOutcome::ConsumerError | PollOutcome::Gibberish | PollOutcome::OutOfMessages => {
                None
            }
        }
    }
}

impl DeclarationError {
    fn new(subscriber: &str, error: LapinError) -> Self {
        Self {
            subscriber: subscriber.to_string(),
            error: error.to_string(),
        }
    }
}
