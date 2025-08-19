use lapin::acker::Acker;
use lapin::options::{BasicAckOptions, BasicRejectOptions};
use tracing::{error, warn};

/// **Completes** a [`Delivery`](lapin::message::Delivery) by calling `ack` on
/// its [`Acker`]. “Complete” is a form of finalizing an incoming message. A
/// message must be finalized exactly once.
///
/// Failing to complete a delivery is potentially a problem with the application’s
/// logic, so it is logged at the error level.
pub(crate) async fn complete_delivery(subscriber: &str, acker: &Acker, bytes: &[u8]) {
    if let Err(error) = acker.ack(BasicAckOptions { multiple: false }).await {
        error!(
            alert = true,
            subscriber,
            ?error,
            error_message = %error,
            byte_preview = String::from_utf8_lossy(bytes).as_ref(),
            "Failed to complete (acknowledge) an incoming RabbitMQ message",
        );
    }
}

/// **Backwashes** a [`Delivery`](lapin::message::Delivery) by calling `reject`
/// on its [`Acker`], while requesting to re-queue the message. “Backwash” is a
/// form of finalizing an incoming message. A message must be finalized exactly
/// once.
///
/// Backwashing is semantically identical to dropping the message without calling
/// its [`Acker`]. Still, failing to backwash a delivery is potentially a problem
/// with the application’s logic, so it is logged, but at the warning level.
pub(crate) async fn backwash_delivery(subscriber: &str, acker: &Acker, bytes: &[u8]) {
    if let Err(error) = acker.reject(BasicRejectOptions { requeue: true }).await {
        warn!(
            alert = true,
            subscriber,
            ?error,
            error_message = %error,
            byte_preview = String::from_utf8_lossy(bytes).as_ref(),
            "Failed to backwash (reject with re-queueing) an incoming RabbitMQ message",
        );
    }
}

/// **Abandons** a [`Delivery`](lapin::message::Delivery) by calling `reject`
/// on its [`Acker`], without re-queueing the message. “Abandon” is a form of
/// finalizing an incoming message. A message must be finalized exactly once.
///
/// Failing to abandon a delivery is potentially a problem with the application’s
/// logic, so it is logged at the error level.
pub(crate) async fn abandon_delivery(subscriber: &str, acker: &Acker, bytes: &[u8]) {
    if let Err(error) = acker.reject(BasicRejectOptions { requeue: false }).await {
        error!(
            alert = true,
            subscriber,
            ?error,
            error_message = %error,
            byte_preview = String::from_utf8_lossy(bytes).as_ref(),
            "Failed to abandon (reject without re-queueing) an incoming RabbitMQ message",
        );
    }
}
