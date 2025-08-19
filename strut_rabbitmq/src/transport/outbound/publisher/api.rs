use crate::transport::outbound::publisher::inner::{
    NotConfirmed, NotTransmitted, PartlyConfirmedBatch,
};
use crate::Dispatch;
use nonempty::NonEmpty;
use std::fmt::{Display, Formatter};
use thiserror::Error;

/// Shorthand for a result of a single publishing attempt.
pub type PublishingResult = Result<Dispatch, PublishingError>;

/// Shorthand for a result of a batch publishing attempt.
pub type BatchPublishingResult = Result<Vec<Dispatch>, BatchPublishingError>;

/// Represents a failed publishing of a single RabbitMQ message.
#[derive(Error, Debug)]
#[error("failed to publish a RabbitMQ message: {failure}")]
pub struct PublishingError {
    /// The message that failed to get published.
    pub dispatch: Dispatch,
    /// The high-level explanation of the failure.
    pub failure: PublishingFailure,
}

/// Represents a (partially) failed publishing of a batch of RabbitMQ messages.
#[derive(Error, Debug)]
#[error(
    "failed to fully publish a batch of {} RabbitMQ messages: {} messages went through, {} messages did not go through",
    published.len() + not_published.len(),
    published.len(),
    not_published.len(),
)]
pub struct BatchPublishingError {
    /// The messages that went through (successfully published).
    pub published: Vec<Dispatch>,
    /// The messages that did not go through (failed to get published).
    pub not_published: NonEmpty<PublishingError>,
}

/// Explains what exactly went wrong in publishing a single RabbitMQ message.
#[derive(Debug)]
pub enum PublishingFailure {
    /// The message was not transmitted to the broker.
    NotTransmitted,
    /// The message was negatively acknowledged by the broker (not routed to an
    /// exchange or a queue, depending on confirmation level).
    NegativelyAcknowledged,
    /// The broker suffered an internal error during acknowledgement of the
    /// message.
    BrokerError,
    /// Failed to retrieve the acknowledgement from the broker.
    CommunicationError,
}

impl Display for PublishingFailure {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PublishingFailure::NotTransmitted => {
                f.write_str("the message was not transmitted to the broker")
            }
            PublishingFailure::NegativelyAcknowledged => f.write_str(concat!(
                "the message was negatively acknowledged by the broker",
                " (not routed to an exchange or a queue, depending on",
                " confirmation level)",
            )),
            PublishingFailure::BrokerError => f.write_str(
                "the broker suffered an internal error during acknowledgement of the message",
            ),
            PublishingFailure::CommunicationError => {
                f.write_str("failed to retrieve the acknowledgement from the broker")
            }
        }
    }
}

impl From<NotTransmitted> for PublishingError {
    fn from(value: NotTransmitted) -> Self {
        let dispatch = match value {
            NotTransmitted::NotAttempted(dispatch) => dispatch,
            NotTransmitted::TransmissionError(dispatch, _) => dispatch,
        };

        Self {
            dispatch,
            failure: PublishingFailure::NotTransmitted,
        }
    }
}

impl From<NotConfirmed> for PublishingError {
    fn from(value: NotConfirmed) -> Self {
        match value {
            NotConfirmed::NotAttempted(dispatch) => Self {
                dispatch,
                failure: PublishingFailure::NotTransmitted,
            },
            NotConfirmed::TransmissionError(dispatch, _error) => Self {
                dispatch,
                failure: PublishingFailure::NotTransmitted,
            },
            NotConfirmed::Negative(dispatch, _return) => Self {
                dispatch,
                failure: PublishingFailure::NegativelyAcknowledged,
            },
            NotConfirmed::BrokerError(dispatch, _return) => Self {
                dispatch,
                failure: PublishingFailure::BrokerError,
            },
            NotConfirmed::ConfirmationError(dispatch, _error) => Self {
                dispatch,
                failure: PublishingFailure::CommunicationError,
            },
        }
    }
}

impl From<PartlyConfirmedBatch> for BatchPublishingError {
    fn from(value: PartlyConfirmedBatch) -> Self {
        BatchPublishingError {
            published: value
                .confirmed_dispatches
                .into_iter()
                .map(Dispatch::from)
                .collect(),
            not_published: value.not_confirmed_dispatches.map(PublishingError::from),
        }
    }
}
