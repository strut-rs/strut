use crate::transport::outbound::publisher::api::{BatchPublishingResult, PublishingResult};
use crate::transport::outbound::publisher::inner::{
    BatchTransmissionResult, Confirmed, ConfirmedBatch, NotTransmitted, PartlyTransmittedBatch,
    TransmissionResult, Transmitted, TransmittedBatch,
};
use crate::util::RetrievePushMap;
use crate::{Connector, DeliveryMode, Dispatch, Egress, Gateway, Handle};
use lapin::options::{BasicPublishOptions, ConfirmSelectOptions};
use lapin::Channel;
use nonempty::NonEmpty;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{Mutex as AsyncMutex, MutexGuard};
use tracing::error;

pub mod api;
mod inner;

/// Publishes outgoing [`Dispatch`]es to the RabbitMQ cluster.
///
/// Distinguishes single publishing
/// ([`try_publish`](Publisher::try_publish) and [`publish`](Publisher::publish))
/// from batch publishing ([`try_publish_many`](Publisher::try_publish_many) and
/// [`publish_many`](Publisher::publish_many)).
///
/// Also, distinguishes fail-fast publishing
/// ([`try_publish`](Publisher::try_publish) and
/// [`try_publish_many`](Publisher::try_publish_many)) from error-less
/// publishing ([`publish`](Publisher::publish) and
/// [`publish_many`](Publisher::publish_many)).
///
/// ## Connection
///
/// This publisher delegates establishing connection and creation of [`Channel`]s
/// to [`Connector`], which must be [started](Connector::start) before creating
/// a publisher.
///
/// No more than one [`Channel`] is being kept by this publisher, and it is
/// re-fetched whenever a connection issue is suspected.
///
/// ## Configuration
///
/// All publishing configuration is off-loaded to [`Egress`].
///
/// One important part of the egress configuration is
/// [`ConfirmationLevel`](crate::ConfirmationLevel). This level has significant
/// consequences for the publishing process, as described in its documentation.
///
/// ## Publishing
///
/// Publishing a [`Dispatch`] to RabbitMQ is a two-step process:
///
/// 1. **Transmit** the dispatch payload over network to the broker.
/// 2. **Confirm** with the broker the successful reception of the message.
///
/// The transmission step plays out the same way regardless of the configuration.
/// This publisher always transmits one dispatch at a time (there should be no
/// benefit in transmitting dispatches in parallel in a single channel).
///
/// The confirmation step depends a lot on the
/// [confirmation level](crate::ConfirmationLevel) selected on the
/// [egress](Egress).
///
/// ### Batch benefits
///
/// The publishing of a [`Dispatch`] may be a one-step or a two-step process
/// depending on the [confirmation level](crate::ConfirmationLevel) of this
/// publisher’s [`Egress`].
///
/// The first step (transmission of the message to the broker) does not benefit
/// from batching since the messages are transmitted one at a time. However, the
/// second step (confirmation with the broker), if executed, can benefit from
/// the batch approach.
///
/// ### Publishing API
///
/// The following four publishing methods are exposed.
///
/// All publishing methods return all [`Dispatch`]es that were passed into them
/// (both in the happy path and in the case of an error). It is up to the caller
/// to then either drop the dispatches or use them for a different purpose (e.g.,
/// also publish them via a different [`Publisher`]).
///
/// #### [`try_publish`](Publisher::try_publish): single [`Dispatch`], fail-fast
///
/// Attempts once to publish a single dispatch and returns an error as soon as
/// something goes wrong.
///
/// #### [`publish`](Publisher::publish): single [`Dispatch`], error-less
///
/// Repeatedly attempts to publish a single dispatch and returns only once the
/// message is confirmed.
///
/// #### [`try_publish_many`](Publisher::try_publish_many): batch of [`Dispatch`]es, fail-fast
///
/// Attempts once to publish a batch of dispatches and returns an error as soon
/// as something goes wrong.
///
/// #### [`publish_many`](Publisher::publish_many): batch of [`Dispatch`]es, error-less
///
/// Repeatedly attempts to publish a batch of dispatches and returns only once
/// all the messages are confirmed.
pub struct Publisher {
    /// The globally unique name of this publisher, for logging/debugging
    /// purposes.
    name: Arc<str>,
    /// The [`Egress`] used by this publisher to transport outgoing dispatches.
    egress: Egress,
    /// The [`Gateway`] for the RabbitMQ [`Channel`]s, as returned by
    /// [`Connector`].
    gateway: Gateway,
    /// The current [`Channel`] of this publisher.
    channel: AsyncMutex<Option<Channel>>,
}

impl Publisher {
    /// Creates and returns a new [`Publisher`].
    pub fn new(gateway: Gateway, egress: Egress) -> Self {
        let name = Self::compose_name(&egress);
        let channel = AsyncMutex::new(None);

        Self {
            name,
            egress,
            gateway,
            channel,
        }
    }

    /// Starts a new [`Connector`] with the given [`Handle`] and uses it to create
    /// and return a new [`Publisher`] for the given [`Egress`].
    pub fn start(handle: impl AsRef<Handle>, egress: Egress) -> Self {
        let gateway = Connector::start(handle);

        Self::new(gateway, egress)
    }

    /// Composes a globally unique, human-readable name for this [`Publisher`].
    fn compose_name(egress: &Egress) -> Arc<str> {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);

        Arc::from(format!(
            "rabbitmq:pub:{}:{}",
            egress.name(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        ))
    }
}

impl Publisher {
    /// Reports the name of this [`Publisher`].
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Publisher {
    /// Attempts once to publish a single [`Dispatch`] and returns an error as
    /// soon as something goes wrong.
    ///
    /// The provided dispatch is returned back to the caller, both in the happy
    /// path and in the case of an error.
    ///
    /// This is a fail-fast version of the single-dispatch publishing. For the
    /// error-less approach, use [`publish`](Publisher::publish).
    pub async fn try_publish(&self, dispatch: impl Into<Dispatch>) -> PublishingResult {
        // Dive in
        let dispatch = dispatch.into();

        // Transmit and fail fast
        let transmitted = self.try_transmit(dispatch).await?;

        // Confirm and fail fast again
        let confirmed = transmitted.confirm(self.name.as_ref()).await?;

        // Return the confirmed dispatch
        Ok(Dispatch::from(confirmed))
    }

    /// Repeatedly attempts to publish a single [`Dispatch`] and returns only
    /// once the message is confirmed.
    ///
    /// The provided dispatch is returned back to the caller.
    ///
    /// Note that the [`ConfirmationLevel`](crate::ConfirmationLevel) on this
    /// publisher’s [`Egress`] will significantly affect the publishing
    /// semantics.
    ///
    /// This is an error-less version of the single-dispatch publishing. For the
    /// fail-fast approach, use [`try_publish`](Publisher::try_publish).
    pub async fn publish(&self, dispatch: impl Into<Dispatch>) -> Dispatch {
        // Dive in
        let dispatch = dispatch.into();

        // Push the given dispatch until it is confirmed
        let confirmed = self.push_one_until_confirmed(dispatch).await;

        // Return the confirmed dispatch
        Dispatch::from(confirmed)
    }

    /// Attempts once to publish a batch of [`Dispatch`]es and returns an error
    /// as soon as something goes wrong.
    ///
    /// The provided dispatches are all returned back to the caller, both in the
    /// happy path and in the case of an error.
    ///
    /// This is a fail-fast version of the batch publishing. For the error-less
    /// approach, use [`publish_many`](Publisher::publish_many).
    pub async fn try_publish_many<I>(&self, dispatches: I) -> BatchPublishingResult
    where
        I: IntoIterator,
        I::Item: Into<Dispatch>,
    {
        // Package input into a vector
        let dispatches = dispatches.into_iter().map(Into::into).collect::<Vec<_>>();

        // Short circuit
        if dispatches.is_empty() {
            return Ok(dispatches);
        }

        // Transmit the batch but don’t fail fast just yet (give a chance to
        // partially transmitted batches)
        let batch_transmission_result = self.try_transmit_many(dispatches).await;

        // Attempt to confirm the (partially) transmitted dispatches, and fail
        // fast here
        let confirmed_batch = match batch_transmission_result {
            Ok(transmitted_batch) => transmitted_batch.confirm(self.name.as_ref()).await?,
            Err(partly_transmitted_batch) => {
                partly_transmitted_batch.confirm(self.name.as_ref()).await?
            }
        };

        // Return the confirmed dispatches
        Ok(Vec::from(confirmed_batch))
    }

    /// Repeatedly attempts to publish a batch of [`Dispatch`]es and returns
    /// only once all the messages are confirmed.
    ///
    /// The provided dispatches are all returned back to the caller.
    ///
    /// Note that the [`ConfirmationLevel`](crate::ConfirmationLevel) on this
    /// publisher’s [`Egress`] will significantly affect the publishing
    /// semantics.
    ///
    /// This is an error-less version of the batch publishing. For the fail-fast
    /// approach, use [`try_publish_many`](Publisher::try_publish_many).
    pub async fn publish_many<I>(&self, dispatches: I) -> Vec<Dispatch>
    where
        I: IntoIterator,
        I::Item: Into<Dispatch>,
    {
        // Package input into a vector
        let dispatches = dispatches.into_iter().map(Into::into).collect::<Vec<_>>();

        // Short circuit
        if dispatches.is_empty() {
            return dispatches;
        }

        // Push the given dispatches until they are all confirmed
        let confirmed_batch = self.push_many_until_confirmed(dispatches).await;

        // Return the confirmed dispatches
        Vec::from(confirmed_batch)
    }
}

impl Publisher {
    /// Repeatedly publishes the given [`Dispatch`]es until all of them are both
    /// transmitted and confirmed.
    async fn push_many_until_confirmed(&self, dispatches: Vec<Dispatch>) -> ConfirmedBatch {
        // Re-structure dispatches into a deque for easy double-sided access
        let mut pending_dispatches = VecDeque::from(dispatches);

        // Prepare storage for confirmed dispatches
        let mut confirmed_dispatches = Vec::with_capacity(pending_dispatches.len());

        // In a happy path, we will send everything in one iteration of this loop
        while !pending_dispatches.is_empty() {
            // Prepare storage for dispatches transmitted in this iteration
            let mut transmitted_dispatches = Vec::with_capacity(pending_dispatches.len());

            // Transmit all pending dispatches
            for dispatch in pending_dispatches.drain(..) {
                // Transmit the dispatch
                let transmitted = self.push_one_until_transmitted(dispatch).await;

                // Queue up the transmission result
                transmitted_dispatches.push(transmitted);
            }

            // Now, try to confirm all transmitted batches
            for transmitted in transmitted_dispatches {
                // Push for confirmation
                let confirmation_result = transmitted.confirm(self.name.as_ref()).await;

                // Check out the result
                match confirmation_result {
                    // Transmitted and confirmed: all good with this one
                    Ok(confirmed) => {
                        confirmed_dispatches.push(confirmed);
                        continue;
                    }

                    // Transmitted but not confirmed: we’ll have to transmit again
                    Err(not_confirmed) => {
                        // Return the failed dispatch back into the pending collection
                        pending_dispatches.push_back(Dispatch::from(not_confirmed));
                    }
                }
            }
        }

        ConfirmedBatch {
            dispatches: confirmed_dispatches,
        }
    }

    /// Repeatedly publishes the given [`Dispatch`] until it is both transmitted
    /// and confirmed.
    async fn push_one_until_confirmed(&self, mut dispatch: Dispatch) -> Confirmed {
        // Keep trying until transmission is successful and confirmed
        loop {
            // Transmit and confirm
            let transmitted = self.push_one_until_transmitted(dispatch).await;
            let confirmation_result = transmitted.confirm(self.name.as_ref()).await;

            // Inspect the outcome
            match confirmation_result {
                // Transmitted and confirmed: all good with this one
                Ok(confirmed) => {
                    return confirmed;
                }

                // Transmitted but not confirmed: we’ll have to transmit again
                Err(not_confirmed) => {
                    // Put the failed dispatch back into the hot seat
                    dispatch = Dispatch::from(not_confirmed);
                }
            }
        }
    }

    /// Repeatedly transmits the given [`Dispatch`] until it is successfully
    /// transmitted.
    async fn push_one_until_transmitted(&self, mut dispatch: Dispatch) -> Transmitted {
        // Keep trying until transmission is successful
        loop {
            // Send a message
            let outcome = self.try_transmit(dispatch).await;

            // Check out the outcome
            match outcome {
                // Successfully transmitted: return
                Ok(transmitted) => {
                    return transmitted;
                }

                // Failed to transmit: try again
                Err(non_transmitted) => {
                    // Put the non-transmitted dispatch back into hot seat
                    dispatch = Dispatch::from(non_transmitted);
                }
            };
        }
    }

    /// Attempts to transmit the given [`Dispatch`]es once, and gives up as soon
    /// as anything goes wrong.
    async fn try_transmit_many(&self, dispatches: Vec<Dispatch>) -> BatchTransmissionResult {
        // Prepare storage for transmitted and non-transmitted dispatches
        let mut transmitted_dispatches = Vec::new();
        let mut not_transmitted_dispatches = Vec::new();

        // Create an iterator for the dispatches
        let mut remaining = dispatches.into_iter();

        // Follow the happy path and iterate until first failure
        while let Some(dispatch) = remaining.next() {
            // Attempt to transmit
            let transmitted_result = self.try_transmit(dispatch).await;

            // Break on first failure
            match transmitted_result {
                Ok(transmitted) => transmitted_dispatches.push(transmitted),
                Err(not_transmitted) => {
                    not_transmitted_dispatches.push(not_transmitted);
                    break;
                }
            }
        }

        // If there are still dispatches remaining, last sending attempt failed
        while let Some(dispatch) = remaining.next() {
            // Record all remaining dispatches as “not attempted”
            not_transmitted_dispatches.push(NotTransmitted::NotAttempted(dispatch));
        }

        // Check whether there were any issues
        if let Some(not_transmitted_dispatches) = NonEmpty::from_vec(not_transmitted_dispatches) {
            // Issue detected, return the appropriate error
            return Err(PartlyTransmittedBatch {
                transmitted_dispatches,
                not_transmitted_dispatches,
            });
        }

        Ok(TransmittedBatch {
            dispatches: transmitted_dispatches,
        })
    }

    /// Attempts to transmit the given [`Dispatch`] once, and gives up as soon
    /// as anything goes wrong.
    async fn try_transmit(&self, dispatch: Dispatch) -> TransmissionResult {
        // Prepare message properties for publishing, optionally forcing durability
        let mut properties = dispatch.properties();
        if self.egress.force_durable() {
            properties = properties.push_delivery_mode(DeliveryMode::Durable);
        }

        // Grab the channel
        let (mut channel_guard, channel) = self.grab_channel().await;

        // Infer the routing key
        let routing_key = dispatch
            .routing_key()
            .unwrap_or_else(|| self.egress.routing_key());

        // Publish the message and store the initial result
        let result = channel
            .basic_publish(
                self.egress.exchange(),
                routing_key,
                BasicPublishOptions {
                    mandatory: self.egress.requires_mandatory_publish(),
                    immediate: false, // this flag is not supported and ignored by RabbitMQ v3+
                },
                dispatch.bytes(),
                properties,
            )
            .await;

        // If all is good, wrap channel in `Some`, otherwise drop it
        let optional_channel = result.is_ok().then(|| channel);

        // Put the `Option` of channel back
        *channel_guard = optional_channel;
        drop(channel_guard);

        // Inspect whether the message was pushed successfully
        match result {
            // RabbitMQ received the message
            Ok(future_confirm) => {
                // Good hit
                Ok(Transmitted {
                    dispatch,
                    future_confirm,
                })
            }

            // RabbitMQ did not receive the message (likely a connectivity issue)
            Err(error) => {
                error!(
                    alert = true,
                    publisher = self.name.as_ref(),
                    ?error,
                    error_message = %error,
                    byte_preview = String::from_utf8_lossy(dispatch.bytes()).as_ref(),
                    "Failed to publish a message to RabbitMQ (did not transmit over network)",
                );
                Err(NotTransmitted::TransmissionError(dispatch, error))
            }
        }
    }

    /// Encapsulates obtaining a channel either from under the lock, or by
    /// fetching a fresh one.
    async fn grab_channel(&self) -> (MutexGuard<'_, Option<Channel>>, Channel) {
        // Obtain the channel guard
        let mut channel_guard = self.channel.lock().await;

        // Either take the channel or fetch a fresh one
        let channel = match channel_guard.take() {
            Some(channel) => channel,
            None => self.fetch_channel().await,
        };

        // Return the pair
        (channel_guard, channel)
    }

    /// Fetches a fresh channel. If the egress definition requires publisher
    /// confirms, this method will call the appropriate method on the channel
    /// before returning it.
    ///
    /// Fetching of a fresh channel may take a long time (depends on connectivity
    /// to RabbitMQ), but when the channel is returned, it is generally in a
    /// healthy state.
    async fn fetch_channel(&self) -> Channel {
        // Repeat until we manage to both retrieve and configure the channel
        loop {
            // Retrieve a channel
            let channel = self.gateway.channel().await;

            // Check if publisher confirms are required on the channel
            if self.egress.requires_any_confirmation() {
                // Enable publisher confirms
                let result = channel
                    .confirm_select(ConfirmSelectOptions { nowait: false })
                    .await;

                // Check the result
                if let Err(error) = result {
                    // Report
                    error!(
                        alert = true,
                        publisher = self.name.as_ref(),
                        ?error,
                        error_message = %error,
                        "Failed to enable publisher confirms on a RabbitMQ channel",
                    );

                    // Try again with a different channel
                    continue;
                }
            }

            return channel;
        }
    }
}
