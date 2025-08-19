use crate::util::field_table::push::Push;
use crate::util::field_table::retrieve::Retrieve;
use lapin::types::FieldTable;

pub mod push;
pub mod retrieve;

/// Special name of the RabbitMQ header that stores the attempt number (how many
/// times the processing of this message has been attempted).
pub const HEADER_ATTEMPT: &str = "x-attempt";

/// Artificial trait implemented for [`FieldTable`] to allow convenient handling
/// of the header [`HEADER_ATTEMPT`] that carries the processing attempt number.
pub trait Attempt {
    /// Extracts the header value at the key [`HEADER_ATTEMPT`] in this
    /// [`FieldTable`], if it can be inferred as a valid `u32`.
    ///
    /// This method does not do any logical inference: e.g., if no attempt value
    /// is present, [`None`] is promptly returned.
    fn retrieve_attempt(&self) -> Option<u32>;

    /// Sets the header value at the key [`HEADER_ATTEMPT`] in this
    /// [`FieldTable`] to the given `u32` value.
    fn push_attempt(&mut self, attempt: u32);

    /// Increments the existing header value at the key [`HEADER_ATTEMPT`] in
    /// this [`FieldTable`] by one. If no value exists at that key, sets the new
    /// value to one.
    fn increment_attempt(&mut self);
}

impl Attempt for FieldTable {
    fn retrieve_attempt(&self) -> Option<u32> {
        self.retrieve(HEADER_ATTEMPT)
    }

    fn push_attempt(&mut self, attempt: u32) {
        self.push(HEADER_ATTEMPT, attempt);
    }

    fn increment_attempt(&mut self) {
        let current_attempt = self.retrieve_attempt().unwrap_or(0);

        self.push_attempt(current_attempt + 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn field_table_attempts() {
        let mut table = FieldTable::default();
        assert_eq!(table.retrieve_attempt(), None);

        table.increment_attempt();
        assert_eq!(table.retrieve_attempt(), Some(1));

        table.increment_attempt();
        assert_eq!(table.retrieve_attempt(), Some(2));

        table.push_attempt(0);
        assert_eq!(table.retrieve_attempt(), Some(0));
    }

    #[test]
    fn field_table_set_and_get() {
        let mut table = FieldTable::default();
        table.push("bool_key", true);
        assert_eq!(table.retrieve("bool_key"), Some(true));

        table.push("str_key", "hello");
        assert_eq!(table.retrieve("str_key"), Some("hello".to_string()));

        table.push("i8_key", -8);
        assert_eq!(table.retrieve("i8_key"), Some(-8));

        table.push("i16_key", -16);
        assert_eq!(table.retrieve("i16_key"), Some(-16));

        table.push("i32_key", -32);
        assert_eq!(table.retrieve("i32_key"), Some(-32));

        table.push("i64_key", -64);
        assert_eq!(table.retrieve("i64_key"), Some(-64));

        table.push("u8_key", 8);
        assert_eq!(table.retrieve("u8_key"), Some(8));

        table.push("u16_key", 16);
        assert_eq!(table.retrieve("u16_key"), Some(16));

        table.push("u32_key", 32);
        assert_eq!(table.retrieve("u32_key"), Some(32));

        table.push("u64_key", 64);
        assert_eq!(table.retrieve("u64_key"), Some(64));
    }

    #[test]
    fn field_table_invalid_gets() {
        let table = FieldTable::default();
        assert_eq!(table.retrieve("missing_key"), None::<bool>);
        assert_eq!(table.retrieve("missing_key"), None::<i32>);
        assert_eq!(table.retrieve("missing_key"), None::<u64>);
    }
}
