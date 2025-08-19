use crate::Handle;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use lapin::{Channel, Connection, ConnectionProperties, Error as LapinError};
use secure_string::SecureString;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use strut_core::{AppContext, AppSpindown, AppSpindownToken};
use strut_sync::{Conduit, Retriever};
use strut_util::Backoff;
use thiserror::Error;
use tokio::select;
use tokio::sync::{oneshot, Mutex as AsyncMutex};
use tokio::task::JoinHandle;
use tracing::{info, warn};

/// Runs in the background, maintains no more than one active connection to a
/// RabbitMQ cluster (referred to herein as **current connection**) identified
/// by the given [`Handle`]. Exposes a cheaply clone-able [`Gateway`], which
/// any number of asynchronous tasks can use to retrieve a fresh [`Channel`]
/// created in the current connection.
///
/// Fully encapsulates reconnection and clean-up logic. Reconnection is
/// triggered whenever a channel is requested and this connector is unable to
/// produce it (likely, because there is no connectivity to the RabbitMQ
/// cluster). Reconnections are performed with an exponential backoff strategy.
/// All connections are properly closed in the background before discarding.
///
/// The clients should keep their copy of [`Gateway`] and re-use it to request
/// a new [`Channel`] whenever the previous channel seems to be no longer
/// working (e.g., the underlying connection was lost). The clients should
/// expect that the gateway may take a long or even indefinite time, depending
/// on the RabbitMQ cluster availability.
///
/// This connector is integrated with [`AppSpindown`]: once the global
/// [`AppContext`] is terminated, this connector will stop serving channels and
/// will attempt to gracefully close the current connection.
pub struct Connector {
    /// The globally unique name of this connector, for logging/debugging
    /// purposes.
    name: Arc<str>,
    /// The identifier of this connector’s [`Handle`], for logging/debugging
    /// purposes.
    identifier: Arc<str>,
    /// The DSN of the RabbitMQ cluster, to which this connector connects.
    dsn: SecureString,
    /// The current [`Connection`] to the RabbitMQ cluster, if present.
    connection: AsyncMutex<Option<Connection>>,
    /// The collection of previous connections that are being closed in the
    /// background.
    discarded_connections: AsyncMutex<FuturesUnordered<JoinHandle<()>>>,
    /// The counter for keeping track how many times we discarded a connection.
    discarded_count: AtomicUsize,
    /// The backoff algorithm to be used in repeated connection attempts.
    backoff: Backoff,
    /// The conduit for receiving [`Channel`] requests.
    conduit: Conduit<Channel>,
    /// The canary token, which (once it goes out of scope) will inform the
    /// application that this connector gracefully completed.
    _spindown_token: AppSpindownToken,
}

/// An asynchronous gateway to creating and retrieving fresh [`Channel`]s on an
/// internally maintained [`Connection`].
///
/// A gateway is created by [starting](Connector::start) a [`Connector`].
pub struct Gateway {
    retriever: Retriever<Channel>,
}

impl Connector {
    /// Creates a new [`Connector`] for the given [`Handle`] and sends it into
    /// background to lazily serve [`Channel`] requests via the returned
    /// [`Gateway`], which can be cheaply cloned and shared across
    /// asynchronous tasks.
    pub fn start(handle: impl AsRef<Handle>) -> Gateway {
        let handle = handle.as_ref();
        let name = Self::compose_name(handle);
        let identifier = Arc::from(handle.identifier());
        let dsn = handle.dsn().clone();
        let connection = AsyncMutex::new(None);
        let discarded_connections = AsyncMutex::new(FuturesUnordered::new());
        let discarded_count = AtomicUsize::new(0);
        let backoff = Backoff::new(handle.backoff());
        let conduit = Conduit::new();
        let retriever = conduit.retriever();
        let _spindown_token = AppSpindown::register(&name);

        let connector = Self {
            name,
            identifier,
            dsn,
            connection,
            discarded_connections,
            discarded_count,
            backoff,
            conduit,
            _spindown_token,
        };

        tokio::spawn(connector.serve());

        Gateway { retriever }
    }

    /// Composes a human-readable name for this connector.
    fn compose_name(handle: &Handle) -> Arc<str> {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);

        Arc::from(format!(
            "rabbitmq:connector:{}:{}",
            handle.name(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        ))
    }
}

impl Connector {
    /// Main, long-running serving function that serves the incoming [`Channel`]
    /// requests until it hears that the global [`AppContext`] has been
    /// terminated. After that it falls into the spindown phase, where it cleans
    /// up before returning.
    async fn serve(self) {
        // Listen to incoming requests and serve them in an infinite loop
        loop {
            // Repeatedly wait for either the application context to terminate,
            // or for an incoming request.
            let state = select! {
                biased;
                _ = AppContext::terminated() => ServingState::Interrupted,
                request = self.conduit.requested() => { // request received
                    // Serving an incoming request is also an asynchronous operation,
                    // so we have to monitor the application context here as well.
                    select! {
                        biased;
                        _ = AppContext::terminated() => ServingState::Interrupted,
                        state = self.receive_request(request) => state,
                    }
                }
            };

            // Check whether we can proceed or should break out
            match state {
                ServingState::Ongoing => continue,
                ServingState::Interrupted => break,
            }
        }

        // Announce spindown
        info!(
            name = self.name.as_ref(),
            identifier = self.identifier.as_ref(),
            "Closing the RabbitMQ connection",
        );

        // Disconnect from RabbitMQ
        self.disconnect().await;

        // Wait for all previously discarded connections to be closed before returning
        self.drain_discarded_connections().await;
    }
}

impl Gateway {
    /// Asynchronously requests the linked [`Connector`] to create a fresh
    /// [`Channel`] on its internally maintained [`Connection`] and return said
    /// channel when ready.
    ///
    /// Depending on the connectivity to RabbitMQ this method may take
    /// arbitrarily long to return. Use the
    /// [`channel_with_timeout`](Gateway::channel_with_timeout) method to limit
    /// the waiting time.
    pub async fn channel(&self) -> Channel {
        self.retriever.anticipate().await
    }

    /// Same as the [`channel`](Gateway::channel) method, but returns [`None`]
    /// if waiting for the [`Channel`] exceeds the given `timeout`.
    pub async fn channel_with_timeout(&self, timeout: Duration) -> Option<Channel> {
        self.retriever.request_with_timeout(timeout).await
    }
}

/// Internal marker that indicates the state of this connector.
enum ServingState {
    Ongoing,
    Interrupted,
}

impl Connector {
    /// Serves a single incoming request asynchronously.
    async fn receive_request(&self, request: oneshot::Sender<Channel>) -> ServingState {
        // Retrieve the channel, which may take any amount of time depending on connectivity
        let channel = self.anticipate_channel().await;

        // Send the channel back to the requester
        let result = request.send(channel);

        // Check the result
        if result.is_err() {
            // An error likely indicates that the requester didn’t wait long enough for the result
            warn!(
                name = self.name.as_ref(),
                identifier = self.identifier.as_ref(),
                "Too late to send the requested RabbitMQ channel",
            );
        }

        // Error or not, we continue serving other requests
        ServingState::Ongoing
    }

    /// Takes and discards the current connection to RabbitMQ, if any.
    async fn disconnect(&self) {
        // Grab the connection
        let mut connection_guard = self.connection.lock().await;
        let optional_connection = connection_guard.take();

        // Discard the connection
        if let Some(connection) = optional_connection {
            self.discard_connection(connection).await;
        }
    }

    /// Sequentially waits for and pops off all futures that are busy closing
    /// discarded connections in the background. Returns when the collection of
    /// futures is empty.
    ///
    /// The idea of this method is to be called periodically during repeated
    /// reconnection attempts, in order to avoid infinitely accumulating clean-up
    /// futures.
    async fn drain_discarded_connections(&self) {
        let mut discarded_connections = self.discarded_connections.lock().await;

        while discarded_connections.next().await.is_some() {}
    }
}

impl Connector {
    /// Repeatedly attempts to create a channel out of an active connection,
    /// infinitely re-connects if necessary (with a backoff strategy), returns a
    /// channel upon first success.
    async fn anticipate_channel(&self) -> Channel {
        // Grab the connection
        let mut connection_guard = self.connection.lock().await;
        let mut optional_connection = connection_guard.take();

        // Repeat until success
        loop {
            // Try to make a channel on the given connection
            match self.try_create_channel(optional_connection).await {
                // Success: we have a good connection and a fresh channel
                Ok(CreatedChannel {
                    connection,
                    channel,
                }) => {
                    // Put the connection back under lock
                    *connection_guard = Some(connection);

                    // Return the fresh channel
                    return channel;
                }

                // Error: either we didn’t have a connection to begin with, or it has gone bad
                Err(_) => {
                    // Attempt to establish a fresh connection before continuing
                    optional_connection = self.establish_connection().await;
                }
            };
        }
    }

    /// Tries to create a connection with the given channel. If successful,
    /// returns both the channel and the connection. If not successful, performs
    /// reporting and clean-up.
    async fn try_create_channel(
        &self,
        optional_connection: Option<Connection>,
    ) -> Result<CreatedChannel, ConnectorError> {
        // Unwrap the connection
        let connection = match optional_connection {
            Some(connection) => connection,
            None => return Err(ConnectorError::NoConnection),
        };

        // Try to create a channel
        let channel_result = connection.create_channel().await;

        // Inspect the result
        match channel_result {
            // Failed to create a channel
            Err(error) => {
                // Log the connection error
                warn!(
                    name = self.name.as_ref(),
                    identifier = self.identifier.as_ref(),
                    ?error,
                    error_message = %error,
                    "Failed to create a RabbitMQ channel",
                );

                // Discard the obviously bad connection
                self.discard_connection(connection).await;

                // Wait a bit
                self.backoff.sleep_next().await;

                Err(ConnectorError::ChannelCreationError)
            }

            // Successfully created a channel
            Ok(channel) => {
                // Reset backoff
                self.backoff.reset();

                Ok(CreatedChannel {
                    channel,
                    connection,
                })
            }
        }
    }

    /// Attempts to establish a fresh connection to the RabbitMQ cluster behind
    /// this connector’s [`Handle`].
    async fn establish_connection(&self) -> Option<Connection> {
        // Set up the connection properties to use the current Tokio context
        let connection_properties = ConnectionProperties::default()
            .with_executor(tokio_executor_trait::Tokio::current())
            .with_reactor(tokio_reactor_trait::Tokio);

        // Establish a connection
        let connection_result =
            Connection::connect(self.dsn.unsecure(), connection_properties).await;

        // Check the result
        match connection_result {
            // Success: return the connection
            Ok(connection) => Some(connection),

            // Error: likely no connectivity with RabbitMQ
            Err(error) => {
                // Log the connection error
                warn!(
                    name = self.name.as_ref(),
                    identifier = self.identifier.as_ref(),
                    ?error,
                    error_message = %error,
                    "Failed to establish a RabbitMQ connection",
                );

                // Wait a bit
                self.backoff.sleep_next().await;

                // Return
                None
            }
        }
    }

    /// Initiates discarding of the given channel. Every once in a while, this
    /// method will also drain discarded connections, so they don’t accumulate
    /// indefinitely.
    async fn discard_connection(&self, connection: Connection) {
        // Connection existed: create a clean-up future, then send it to the background
        let future = Self::close_connection(self.name.clone(), self.identifier.clone(), connection);
        let handle = tokio::spawn(future);

        // Grab the collection of discarded connections; push a new one onto it
        self.discarded_connections.lock().await.push(handle);

        // Periodically drain discarded connections
        const DISCARDED_COUNT_BETWEEN_CLEANUPS: usize = 10;
        let discarded_count = self.discarded_count.fetch_add(1, Ordering::Relaxed);
        if discarded_count % DISCARDED_COUNT_BETWEEN_CLEANUPS == 0 {
            self.drain_discarded_connections().await;
        }
    }

    /// Works on closing the given connection, uses the given name and identifier
    /// for logging the outcome.
    async fn close_connection(name: Arc<str>, identifier: Arc<str>, connection: Connection) {
        // Close the given connection
        let result = connection.close(0, "Discarded connection").await;

        // Check and report the outcome
        match result {
            Ok(_) => info!(
                name = name.as_ref(),
                identifier = identifier.as_ref(),
                "Closed a discarded RabbitMQ connection",
            ),
            Err(LapinError::InvalidConnectionState(_)) => info!(
                name = name.as_ref(),
                identifier = identifier.as_ref(),
                "Discarded a previously lost RabbitMQ connection",
            ),
            Err(LapinError::InvalidChannelState(state)) => info!(
                name = name.as_ref(),
                identifier = identifier.as_ref(),
                "Ignored a channel in the invalid state '{:?}' within a discarded RabbitMQ connection",
                state,
            ),
            Err(error) => warn!(
                name = name.as_ref(),
                identifier = identifier.as_ref(),
                ?error,
                error_message = %error,
                "Failed to cleanly close a discarded RabbitMQ connection",
            ),
        }
    }
}

/// A little wrapper to conveniently pass both the fresh channel and the connection
/// from which it originated.
struct CreatedChannel {
    connection: Connection,
    channel: Channel,
}

/// Internal error representing the reasons while creating a channel may fail.
#[derive(Error, Debug)]
enum ConnectorError {
    #[error("failed to create a channel: no connection provided")]
    NoConnection,
    #[error("failed to create a channel on the given connection")]
    ChannelCreationError,
}
