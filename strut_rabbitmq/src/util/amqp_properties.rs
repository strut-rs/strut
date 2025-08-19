use crate::util::field_table::Attempt;
use crate::util::{PushHeader, HEADER_ATTEMPT};
use crate::DeliveryMode;
use lapin::protocol::basic::AMQPProperties;
use lapin::types::FieldTable;
use std::time::{SystemTime, UNIX_EPOCH};

pub mod push;
pub mod retrieve;

/// Convenience layer around [`AMQPProperties`] that allows easier getting and
/// setting of the common properties.
pub trait RetrievePushMap {
    /// Extracts the attempt number from these [`AMQPProperties`], if it is
    /// present and can be coerced to a `u32`.
    ///
    /// The attempt number is not a recognized AMQP property, and is stored in
    /// the headers under the key [`HEADER_ATTEMPT`].
    fn retrieve_attempt(&self) -> Option<u32>;

    /// Extracts the [`DeliveryMode`] from these [`AMQPProperties`], if it is
    /// present.
    fn retrieve_delivery_mode(&self) -> Option<DeliveryMode>;

    /// Extracts the priority from these [`AMQPProperties`], if it is present
    /// and can be coerced to a `u8`.
    fn retrieve_priority(&self) -> Option<u8>;

    /// Extracts the timestamp from these [`AMQPProperties`], if it is present
    /// and can be coerced to a `u64` (a UNIX timestamp).
    fn retrieve_timestamp(&self) -> Option<u64>;

    /// Sets the attempt number in these [`AMQPProperties`] to the given value.
    ///
    /// The attempt number is not a recognized AMQP property, and is stored in
    /// the headers under the key [`HEADER_ATTEMPT`].
    fn push_attempt(self, attempt: u32) -> Self;

    /// Increments the attempt number in these [`AMQPProperties`] by one. If no
    /// value exists, sets the new value to one.
    ///
    /// The attempt number is not a recognized AMQP property, and is stored in
    /// the headers under the key [`HEADER_ATTEMPT`].
    fn increment_attempt(self) -> Self;

    /// Sets the [`DeliveryMode`] in these [`AMQPProperties`] to the given value.
    fn push_delivery_mode(self, delivery_mode: DeliveryMode) -> Self;

    /// Sets the priority in these [`AMQPProperties`] to the given `u8` value.
    fn push_priority(self, priority: u8) -> Self;

    /// Sets the timestamp in these [`AMQPProperties`] to the given `u64` value
    /// (UNIX timestamp).
    fn push_timestamp(self, timestamp: u64) -> Self;

    /// Sets the message ID in these [`AMQPProperties`] to the current timestamp
    /// as reported by [`SystemTime`] and converted to a `u64` value (UNIX
    /// timestamp).
    fn push_current_timestamp(self) -> Self;
}

impl RetrievePushMap for AMQPProperties {
    fn retrieve_attempt(&self) -> Option<u32> {
        self.headers()
            .as_ref()
            .and_then(FieldTable::retrieve_attempt)
    }

    fn retrieve_delivery_mode(&self) -> Option<DeliveryMode> {
        self.delivery_mode().map(DeliveryMode::from)
    }

    fn retrieve_priority(&self) -> Option<u8> {
        *self.priority()
    }

    fn retrieve_timestamp(&self) -> Option<u64> {
        *self.timestamp()
    }

    fn push_attempt(self, attempt: u32) -> Self {
        self.push_header(HEADER_ATTEMPT, attempt)
    }

    fn increment_attempt(self) -> Self {
        let current_attempt = self.retrieve_attempt().unwrap_or(0);

        self.push_header(HEADER_ATTEMPT, current_attempt + 1)
    }

    fn push_delivery_mode(self, delivery_mode: DeliveryMode) -> Self {
        self.with_delivery_mode(delivery_mode.rabbitmq_value())
    }

    fn push_priority(self, priority: u8) -> Self {
        self.with_priority(priority)
    }

    fn push_timestamp(self, timestamp: u64) -> Self {
        self.with_timestamp(timestamp)
    }

    fn push_current_timestamp(self) -> Self {
        self.with_timestamp(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default() // If system time is somehow before UNIX epoch, set to default of zero
                .as_secs(),
        )
    }
}
