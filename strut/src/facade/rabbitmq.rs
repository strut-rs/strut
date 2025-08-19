use crate::AppConfig;
use strut_rabbitmq::{
    Decoder, Handle, NoopDecoder, Publisher, RabbitMqConfig, StringDecoder, StringSubscriber,
    Subscriber, UndecodedSubscriber,
};

/// A facade for creating RabbitMQ publishers and subscribers.
///
/// This utility simplifies creating RabbitMQ clients by using connection details
/// and routing topology (ingresses/egresses) from the application's global
/// [`RabbitMqConfig`].
///
/// All clients created through this facade use the **default** broker connection
/// details and manage their own connections.
///
/// [`RabbitMqConfig`]: RabbitMqConfig
pub struct RabbitMq;

impl RabbitMq {
    /// Starts a new `Publisher` for a named egress route.
    ///
    /// The publisher connects to the default RabbitMQ broker and sends messages
    /// to the exchange specified in the egress configuration matching the given
    /// `name`.
    ///
    /// # Panics
    ///
    /// Panics if no egress route with the specified `name` is configured.
    pub fn publisher(name: impl AsRef<str>) -> Publisher {
        // Get the application configuration & the default RabbitMQ handle
        let (config, handle) = Self::config_and_handle();

        // Get the egress
        let egress = config.egress().expect(name);

        // Start the publisher
        let publisher = Publisher::start(handle, egress.clone());

        publisher
    }

    /// Starts a new `Subscriber` for a named ingress route.
    ///
    /// The subscriber connects to the default RabbitMQ broker and consumes messages
    /// from the queue specified in the ingress configuration matching the given
    /// `name`.
    ///
    /// Messages are decoded using the provided `decoder`. For common decoding
    /// strategies, see the specialized methods like [`string_subscriber`] or
    /// [`json_subscriber`].
    ///
    /// # Panics
    ///
    /// Panics if no ingress route with the specified `name` is configured.
    ///
    /// [`string_subscriber`]: RabbitMq::string_subscriber
    /// [`json_subscriber`]: RabbitMq::json_subscriber
    pub fn subscriber_with_decoder<T, D>(name: impl AsRef<str>, decoder: D) -> Subscriber<T, D>
    where
        D: Decoder<Result = T>,
    {
        Self::make_subscriber(name, decoder)
    }

    /// Starts a subscriber that delivers raw, undecoded message payloads.
    ///
    /// This is a convenient alternative to calling [`subscriber_with_decoder`]
    /// with a [`NoopDecoder`].
    ///
    /// [`subscriber_with_decoder`]: RabbitMq::subscriber_with_decoder
    pub fn undecoded_subscriber(name: impl AsRef<str>) -> UndecodedSubscriber {
        Self::subscriber_with_decoder(name, NoopDecoder)
    }

    /// Starts a subscriber that decodes message payloads as UTF-8 strings.
    ///
    /// This is a convenient alternative to calling [`subscriber_with_decoder`]
    /// with a [`StringDecoder`].
    ///
    /// [`subscriber_with_decoder`]: RabbitMq::subscriber_with_decoder
    pub fn string_subscriber(name: impl AsRef<str>) -> StringSubscriber {
        Self::subscriber_with_decoder(name, StringDecoder)
    }

    /// Starts a subscriber that deserializes JSON message payloads into type `T`.
    ///
    /// The type `T` must implement `serde::de::DeserializeOwned`.
    ///
    /// This is a convenient alternative to calling [`subscriber_with_decoder`]
    /// with a [`JsonDecoder`].
    ///
    /// [`subscriber_with_decoder`]: RabbitMq::subscriber_with_decoder
    /// [`JsonDecoder`]: strut_rabbitmq::JsonDecoder
    #[cfg(feature = "rabbitmq-json")]
    pub fn json_subscriber<T>(name: impl AsRef<str>) -> strut_rabbitmq::JsonSubscriber<T>
    where
        T: serde::de::DeserializeOwned,
    {
        Self::subscriber_with_decoder(name, strut_rabbitmq::JsonDecoder::default())
    }

    /// An internal helper for making the RabbitMQ subscriber.
    fn make_subscriber<T, D>(name: impl AsRef<str>, decoder: D) -> Subscriber<T, D>
    where
        D: Decoder<Result = T>,
    {
        // Get the application configuration & the default RabbitMQ handle
        let (config, handle) = Self::config_and_handle();

        // Get the ingress
        let ingress = config.ingress().expect(name);

        // Start the subscriber
        let subscriber = Subscriber::start(handle, ingress.clone(), decoder);

        subscriber
    }

    /// An internal helper for getting the application configuration and the
    /// default RabbitMQ handle in one call.
    fn config_and_handle() -> (&'static RabbitMqConfig, &'static Handle) {
        // Get the application configuration
        let config = AppConfig::get().rabbitmq();

        // Get the default RabbitMQ handle
        let handle = config.default_handle();

        (config, handle)
    }
}
