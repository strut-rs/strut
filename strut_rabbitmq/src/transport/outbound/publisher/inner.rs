use crate::Dispatch;
use lapin::message::BasicReturnMessage as LapinReturn;
use lapin::publisher_confirm::{
    Confirmation as LapinConfirm, PublisherConfirm as LapinFutureConfirm,
};
use lapin::Error as LapinError;
use nonempty::NonEmpty;
use std::fmt::{Display, Formatter};
use thiserror::Error;
use tracing::error;

/// Shorthand for a result of a single transmission attempt.
pub(crate) type TransmissionResult = Result<Transmitted, NotTransmitted>;

/// Shorthand for a result of a batch transmission attempt.
pub(crate) type BatchTransmissionResult = Result<TransmittedBatch, PartlyTransmittedBatch>;

/// Shorthand for a result of a single confirmation.
pub(crate) type ConfirmationResult = Result<Confirmed, NotConfirmed>;

/// Shorthand for a result of a batch confirmation.
pub(crate) type BatchConfirmationResult = Result<ConfirmedBatch, PartlyConfirmedBatch>;

/// Represents the positive outcome of a single transmission attempt.
///
/// The main purpose of this struct is to hold together the original [`Dispatch`]
/// and the [`LapinFutureConfirm`] obtained from the successful transmission.
#[derive(Debug)]
pub(crate) struct Transmitted {
    pub(crate) dispatch: Dispatch,
    pub(crate) future_confirm: LapinFutureConfirm,
}

/// Represents the negative outcome of a single transmission attempt.
#[derive(Error, Debug)]
pub(crate) enum NotTransmitted {
    #[error("refused to attempt to transmit an outgoing RabbitMQ message")]
    NotAttempted(Dispatch),
    #[error("failed to transmit an outgoing RabbitMQ message: {1}")]
    TransmissionError(Dispatch, LapinError),
}

/// Represents the fully positive outcome of a batch transmission attempt.
#[derive(Debug)]
pub(crate) struct TransmittedBatch {
    pub(crate) dispatches: Vec<Transmitted>,
}

/// Represents the partially (or even fully) negative outcome of a batch
/// transmission attempt.
///
/// This struct guarantees that there was at least one [`NotTransmitted`]
/// dispatch.
#[derive(Error, Debug)]
#[error(
    "failed to transmit {} out of {} outgoing RabbitMQ messages",
    not_transmitted_dispatches.len(),
    not_transmitted_dispatches.len() + transmitted_dispatches.len(),
)]
pub(crate) struct PartlyTransmittedBatch {
    pub(crate) transmitted_dispatches: Vec<Transmitted>,
    pub(crate) not_transmitted_dispatches: NonEmpty<NotTransmitted>,
}

/// Represents the positive outcome of a single confirmation.
///
/// Distinguishes between the confirmation having been not even requested from
/// the positive acknowledgement from the broker. However, both cases are treated
/// as a success.
#[derive(Debug)]
pub(crate) enum Confirmed {
    NotRequested(Dispatch),
    Positive(Dispatch),
}

/// Represents the negative outcome of a single confirmation.
#[derive(Error, Debug)]
pub(crate) enum NotConfirmed {
    #[error("refused to attempt to transmit an outgoing RabbitMQ message")]
    NotAttempted(Dispatch),
    #[error("failed to transmit an outgoing RabbitMQ message: {1}")]
    TransmissionError(Dispatch, LapinError),
    #[error(
        "failed to confirm an outgoing RabbitMQ message: the broker negatively acknowledged the message"
    )]
    Negative(Dispatch, Return),
    #[error(
        "failed to confirm an outgoing RabbitMQ message: the broker suffered an internal error"
    )]
    BrokerError(Dispatch, Return),
    #[error(
        "failed to confirm an outgoing RabbitMQ message: communication with the broker broke down"
    )]
    ConfirmationError(Dispatch, LapinError),
}

/// Represents the fully positive outcome of a batch confirmation.
#[derive(Debug)]
pub(crate) struct ConfirmedBatch {
    pub(crate) dispatches: Vec<Confirmed>,
}

/// Represents the partially (or even fully) negative outcome of a batch
/// confirmation.
///
/// This struct guarantees that there was at least one [`NotConfirmed`]
/// dispatch.
#[derive(Error, Debug)]
#[error(
    "failed to confirm {} out of {} outgoing RabbitMQ messages",
    not_confirmed_dispatches.len(),
    not_confirmed_dispatches.len() + confirmed_dispatches.len(),
)]
pub(crate) struct PartlyConfirmedBatch {
    pub(crate) confirmed_dispatches: Vec<Confirmed>,
    pub(crate) not_confirmed_dispatches: NonEmpty<NotConfirmed>,
}

/// Represents the message returned back to publisher by the broker.
///
/// Since the original dispatch is retained by the publisher until the publishing
/// is complete, this type is largely informative.
#[derive(Debug)]
pub(crate) enum Return {
    NotReturned,
    Returned(LapinReturn),
}

/// Useful for extracting a non-transmitted dispatch while disregarding the
/// reason for the failed transmission.
impl From<NotTransmitted> for Dispatch {
    fn from(value: NotTransmitted) -> Self {
        match value {
            NotTransmitted::NotAttempted(dispatch) => dispatch,
            NotTransmitted::TransmissionError(dispatch, _) => dispatch,
        }
    }
}

/// Useful for returning the original dispatches after confirmation.
impl From<Confirmed> for Dispatch {
    fn from(value: Confirmed) -> Self {
        match value {
            Confirmed::NotRequested(dispatch) => dispatch,
            Confirmed::Positive(dispatch) => dispatch,
        }
    }
}

/// Useful for extracting a non-confirmed dispatch while disregarding the reason
/// for the failed confirmation.
impl From<NotConfirmed> for Dispatch {
    fn from(value: NotConfirmed) -> Self {
        match value {
            NotConfirmed::NotAttempted(dispatch) => dispatch,
            NotConfirmed::TransmissionError(dispatch, _) => dispatch,
            NotConfirmed::Negative(dispatch, _) => dispatch,
            NotConfirmed::BrokerError(dispatch, _) => dispatch,
            NotConfirmed::ConfirmationError(dispatch, _) => dispatch,
        }
    }
}

/// Useful for returning the original dispatches after confirmation.
impl From<ConfirmedBatch> for Vec<Dispatch> {
    fn from(value: ConfirmedBatch) -> Self {
        value.dispatches.into_iter().map(Dispatch::from).collect()
    }
}

impl Transmitted {
    /// Pushes for the confirmation by the broker of a previously [`Transmitted`]
    /// message. Depending on the configuration of the
    /// [`Publisher`](crate::Publisher) that originally transmitted the message,
    /// this method may or may not involve network communication with the
    /// broker.
    pub(crate) async fn confirm(self, publisher: &str) -> ConfirmationResult {
        // Await the publisher confirm from RabbitMQ
        let confirmation_result = self.future_confirm.await;

        // Interpret the confirmation
        let confirmed = match confirmation_result {
            // Publisher confirm was not requested
            Ok(LapinConfirm::NotRequested) => Confirmed::NotRequested(self.dispatch),

            // RabbitMQ acknowledged our transmission
            Ok(LapinConfirm::Ack(None)) => Confirmed::Positive(self.dispatch),

            // RabbitMQ acknowledged our transmission, but returned the message
            Ok(LapinConfirm::Ack(Some(returned))) => {
                Self::report_negative(self.dispatch, returned, publisher)?
            }

            // RabbitMQ negatively acknowledged our transmission, which only
            // happens on internal error
            Ok(LapinConfirm::Nack(optional_returned)) => {
                Self::report_broker_error(self.dispatch, optional_returned, publisher)?
            }

            // Errored out on receiving publisher confirm
            Err(error) => Self::report_communication_error(self.dispatch, error, publisher)?,
        };

        Ok(confirmed)
    }

    /// Reports a negative acknowledgement by the broker and returns the
    /// appropriate [`ConfirmationResult`].
    pub(crate) fn report_negative(
        dispatch: Dispatch,
        returned: Box<LapinReturn>,
        publisher: &str,
    ) -> ConfirmationResult {
        // Positively confirmed, but message is returned (un-routed)
        let broker_return = Return::from(Some(returned));
        error!(
            alert = true,
            publisher,
            byte_preview =
            String::from_utf8_lossy(dispatch.bytes()).as_ref(),
            %broker_return,
            "Failed to publish a message to RabbitMQ (negatively acknowledged by the broker)",
        );

        Err(NotConfirmed::Negative(dispatch, broker_return))
    }

    /// Reports a broker error and returns the appropriate
    /// [`ConfirmationResult`].
    pub(crate) fn report_broker_error(
        dispatch: Dispatch,
        optional_returned: Option<Box<LapinReturn>>,
        publisher: &str,
    ) -> ConfirmationResult {
        let broker_return = Return::from(optional_returned);
        error!(
            alert = true,
            publisher,
            byte_preview = String::from_utf8_lossy(dispatch.bytes()).as_ref(),
            %broker_return,
            "Failed to publish a message to RabbitMQ (internal broker error)",
        );

        Err(NotConfirmed::BrokerError(dispatch, broker_return))
    }

    /// Reports a communication error and returns the appropriate
    /// [`ConfirmationResult`].
    pub(crate) fn report_communication_error(
        dispatch: Dispatch,
        error: LapinError,
        publisher: &str,
    ) -> ConfirmationResult {
        error!(
            alert = true,
            publisher,
            ?error,
            error_message = %error,
            byte_preview = String::from_utf8_lossy(dispatch.bytes()).as_ref(),
            "Failed to publish a message to RabbitMQ (failed to retrieve a publisher confirm)",
        );

        Err(NotConfirmed::ConfirmationError(dispatch, error))
    }
}

impl TransmittedBatch {
    /// [Confirms](Transmitted::confirm) every transmitted dispatch in this
    /// batch.
    pub(crate) async fn confirm(self, publisher: &str) -> BatchConfirmationResult {
        let mut confirmed_dispatches = Vec::with_capacity(self.dispatches.len());
        let mut not_confirmed_dispatches = Vec::with_capacity(self.dispatches.len());

        for transmitted in self.dispatches {
            let confirmation_result = transmitted.confirm(publisher).await;

            match confirmation_result {
                Ok(confirmed) => {
                    confirmed_dispatches.push(confirmed);
                }
                Err(not_confirmed) => {
                    not_confirmed_dispatches.push(not_confirmed);
                }
            }
        }

        if let Some(not_confirmed_dispatches) = NonEmpty::from_vec(not_confirmed_dispatches) {
            return Err(PartlyConfirmedBatch {
                confirmed_dispatches,
                not_confirmed_dispatches,
            });
        }

        Ok(ConfirmedBatch {
            dispatches: confirmed_dispatches,
        })
    }
}

impl PartlyTransmittedBatch {
    /// [Confirms](Transmitted::confirm) every transmitted dispatch in this
    /// batch.
    pub(crate) async fn confirm(self, publisher: &str) -> BatchConfirmationResult {
        let mut confirmed_dispatches = Vec::with_capacity(self.transmitted_dispatches.len());
        let mut not_confirmed_dispatches = Vec::with_capacity(self.transmitted_dispatches.len());

        for not_transmitted in self.not_transmitted_dispatches {
            not_confirmed_dispatches.push(NotConfirmed::from(not_transmitted));
        }

        for transmitted in self.transmitted_dispatches {
            let confirmation_result = transmitted.confirm(publisher).await;

            match confirmation_result {
                Ok(confirmed) => {
                    confirmed_dispatches.push(confirmed);
                }
                Err(not_confirmed) => {
                    not_confirmed_dispatches.push(not_confirmed);
                }
            }
        }

        if let Some(not_confirmed_dispatches) = NonEmpty::from_vec(not_confirmed_dispatches) {
            return Err(PartlyConfirmedBatch {
                confirmed_dispatches,
                not_confirmed_dispatches,
            });
        }

        Ok(ConfirmedBatch {
            dispatches: confirmed_dispatches,
        })
    }
}

/// Useful for “upgrading” [`NotTransmitted`] to [`NotConfirmed`] when we don’t
/// want to actually go through the trouble of confirming an un-transmitted
/// dispatch.
impl From<NotTransmitted> for NotConfirmed {
    fn from(value: NotTransmitted) -> Self {
        match value {
            NotTransmitted::NotAttempted(dispatch) => NotConfirmed::NotAttempted(dispatch),
            NotTransmitted::TransmissionError(dispatch, error) => {
                NotConfirmed::TransmissionError(dispatch, error)
            }
        }
    }
}

/// Useful for wrapping a [`LapinReturn`].
impl From<Option<Box<LapinReturn>>> for Return {
    fn from(value: Option<Box<LapinReturn>>) -> Self {
        match value {
            None => Self::NotReturned,
            Some(returned) => Self::Returned(*returned),
        }
    }
}

/// Useful for displaying/debugging a [`Return`].
impl Display for Return {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Return::NotReturned => f.write_str("<message not returned>"),
            Return::Returned(returned) => {
                write!(
                    f,
                    "reply code {} '{}'",
                    returned.reply_code, returned.reply_text,
                )
            }
        }
    }
}
