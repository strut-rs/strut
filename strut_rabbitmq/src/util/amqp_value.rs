use lapin::types::AMQPValue;

/// Artificial trait implemented for [`AMQPValue`] to allow reporting whether it
/// can be considered empty.
pub trait IsEmpty {
    /// Reports whether this [`AMQPValue`] may be considered empty (which most
    /// likely means an empty string). Note that numerical zero values are
    /// **not** empty. Empty collections, however, are empty.
    fn is_empty(&self) -> bool;
}

impl IsEmpty for AMQPValue {
    fn is_empty(&self) -> bool {
        match self {
            AMQPValue::ShortString(s) => s.as_str().is_empty(),
            AMQPValue::LongString(s) => s.as_bytes().is_empty(),
            AMQPValue::FieldArray(a) => a.as_slice().is_empty(),
            AMQPValue::FieldTable(t) => t.inner().is_empty(),
            AMQPValue::ByteArray(a) => a.as_slice().is_empty(),
            AMQPValue::Void => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::coerce::Coerce;
    use crate::util::field_table::push::Push;
    use crate::util::morph::Morph;
    use lapin::types::{DecimalValue, FieldTable};
    use pretty_assertions::assert_eq;

    #[test]
    fn from_methods() {
        assert_eq!(AMQPValue::morph(true), AMQPValue::Boolean(true));
        assert_eq!(
            AMQPValue::morph("test"),
            AMQPValue::LongString("test".as_bytes().into()),
        );
        assert_eq!(
            AMQPValue::morph("test".to_string()),
            AMQPValue::LongString("test".as_bytes().into()),
        );
        assert_eq!(AMQPValue::morph(i8::MIN), AMQPValue::ShortShortInt(i8::MIN));
        assert_eq!(AMQPValue::morph(i16::MIN), AMQPValue::ShortInt(i16::MIN));
        assert_eq!(AMQPValue::morph(i32::MIN), AMQPValue::LongInt(i32::MIN));
        assert_eq!(AMQPValue::morph(i64::MIN), AMQPValue::LongLongInt(i64::MIN));
        assert_eq!(
            AMQPValue::morph(u8::MAX),
            AMQPValue::ShortShortUInt(u8::MAX),
        );
        assert_eq!(AMQPValue::morph(u16::MAX), AMQPValue::ShortUInt(u16::MAX));
        assert_eq!(AMQPValue::morph(u32::MAX), AMQPValue::LongUInt(u32::MAX));
        assert_eq!(AMQPValue::morph(u64::MAX), AMQPValue::Timestamp(u64::MAX));
    }

    #[test]
    fn coerce_to_bool() {
        assert_eq!(AMQPValue::Boolean(true).coerce(), Some(true));
        assert_eq!(AMQPValue::Boolean(false).coerce(), Some(false));
        assert_eq!(AMQPValue::ShortShortUInt(1).coerce(), Some(true));
        assert_eq!(AMQPValue::ShortShortUInt(0).coerce(), Some(false));
        assert_eq!(AMQPValue::ShortString("true".into()).coerce(), Some(true),);
        assert_eq!(AMQPValue::ShortString("false".into()).coerce(), Some(false),);
    }

    #[test]
    fn coerce_to_string() {
        assert_eq!(
            AMQPValue::Boolean(true).coerce(),
            Some::<String>("true".into()),
        );
        assert_eq!(
            AMQPValue::ShortShortUInt(42).coerce(),
            Some::<String>("42".into()),
        );
        assert_eq!(
            AMQPValue::ShortString("hello".into()).coerce(),
            Some::<String>("hello".into()),
        );
    }

    #[test]
    fn coerce_to_integers() {
        assert_eq!(AMQPValue::ShortShortUInt(42).coerce(), Some::<i8>(42));
        assert_eq!(AMQPValue::ShortInt(300).coerce(), Some::<i16>(300));
        assert_eq!(AMQPValue::LongInt(-123456).coerce(), Some::<i32>(-123456));
        assert_eq!(AMQPValue::LongLongInt(i64::MAX).coerce(), Some(i64::MAX),);
        assert_eq!(AMQPValue::ShortShortInt(-42).coerce(), None::<u8>);
    }

    #[test]
    fn coerce_to_unsigned_integers() {
        assert_eq!(AMQPValue::ShortShortUInt(42).coerce(), Some::<u8>(42));
        assert_eq!(AMQPValue::ShortUInt(300).coerce(), Some::<u16>(300));
        assert_eq!(AMQPValue::LongUInt(123456).coerce(), Some::<u32>(123456));
        assert_eq!(
            AMQPValue::Timestamp(i64::MAX as u64).coerce(),
            Some(i64::MAX as u64),
        );
    }

    #[test]
    fn coerce_to_usize_isize() {
        assert_eq!(AMQPValue::ShortShortUInt(42).coerce(), Some::<usize>(42));
        assert_eq!(AMQPValue::ShortShortInt(-42).coerce(), Some::<isize>(-42));
    }

    #[test]
    fn invalid_coercions() {
        assert_eq!(AMQPValue::Boolean(true).coerce(), None::<i32>);
        assert_eq!(AMQPValue::Boolean(false).coerce(), None::<u64>);
        assert_eq!(
            AMQPValue::ShortString("not a number".into()).coerce(),
            None::<i32>,
        );
    }

    #[test]
    fn is_empty() {
        assert_eq!(AMQPValue::Boolean(false).is_empty(), false);
        assert_eq!(AMQPValue::ShortShortInt(0).is_empty(), false);
        assert_eq!(AMQPValue::ShortShortUInt(0).is_empty(), false);
        assert_eq!(AMQPValue::ShortInt(0).is_empty(), false);
        assert_eq!(AMQPValue::ShortUInt(0).is_empty(), false);
        assert_eq!(AMQPValue::LongInt(0).is_empty(), false);
        assert_eq!(AMQPValue::LongUInt(0).is_empty(), false);
        assert_eq!(AMQPValue::LongLongInt(0).is_empty(), false);
        assert_eq!(AMQPValue::Float(0.0).is_empty(), false);
        assert_eq!(AMQPValue::Double(0.0).is_empty(), false);
        assert_eq!(
            AMQPValue::DecimalValue(DecimalValue { scale: 0, value: 0 }).is_empty(),
            false,
        );
        assert_eq!(AMQPValue::Timestamp(0).is_empty(), false);

        assert_eq!(AMQPValue::ShortString("".into()).is_empty(), true);
        assert_eq!(AMQPValue::ShortString(" ".into()).is_empty(), false);

        assert_eq!(AMQPValue::LongString("".into()).is_empty(), true);
        assert_eq!(AMQPValue::LongString(" ".into()).is_empty(), false);

        assert_eq!(AMQPValue::FieldArray(vec![].into()).is_empty(), true);
        assert_eq!(
            AMQPValue::FieldArray(vec![AMQPValue::ShortString("".into())].into()).is_empty(),
            false,
        );

        assert_eq!(
            AMQPValue::FieldTable(FieldTable::default()).is_empty(),
            true,
        );
        let mut table = FieldTable::default();
        table.push("", 0);
        assert_eq!(AMQPValue::FieldTable(table).is_empty(), false);

        assert_eq!(AMQPValue::ByteArray(vec![].into()).is_empty(), true);
        assert_eq!(AMQPValue::ByteArray(vec![0].into()).is_empty(), false);

        assert_eq!(AMQPValue::Void.is_empty(), true);
    }
}
