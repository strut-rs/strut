use lapin::types::{AMQPValue, ByteArray, LongString, ShortString};

/// The smallest integer that is continuously representable with an `f32`.
pub const MIN_CONT_INT_F32: i32 = -16_777_216;
/// The largest integer that is continuously representable with an `f32`.
pub const MAX_CONT_INT_F32: i32 = 16_777_216;

/// The smallest integer that is continuously representable with an `f64`.
pub const MIN_CONT_INT_F64: i64 = -9_007_199_254_740_992;
/// The largest integer that is continuously representable with an `f64`.
pub const MAX_CONT_INT_F64: i64 = 9_007_199_254_740_992;

/// Artificial trait implemented for a few types like [`AMQPValue`] or
/// [`ShortString`] to allow conveniently coercing them into standard Rust
/// types.
pub trait Coerce<'a, T> {
    /// Flexibly generates a value of type `T` that is inferred from this object.
    fn coerce(&'a self) -> Option<T>;
}

/// Implement [`Coerce`] for [`AMQPValue`].
const _: () = {
    impl<'a> Coerce<'a, &'a str> for AMQPValue {
        fn coerce(&'a self) -> Option<&'a str> {
            match self {
                Self::ShortString(s) => s.coerce(),
                _ => None,
            }
        }
    }

    impl Coerce<'_, String> for AMQPValue {
        fn coerce(&self) -> Option<String> {
            match self {
                Self::Boolean(value) => Some((*value).to_string()),
                Self::ShortShortUInt(u) => Some((*u).to_string()),
                Self::ShortUInt(u) => Some((*u).to_string()),
                Self::LongUInt(u) => Some((*u).to_string()),
                Self::Timestamp(u) => Some((*u).to_string()),
                Self::ShortShortInt(i) => Some((*i).to_string()),
                Self::ShortInt(i) => Some((*i).to_string()),
                Self::LongInt(i) => Some((*i).to_string()),
                Self::LongLongInt(i) => Some((*i).to_string()),
                Self::ShortString(s) => s.coerce(),
                Self::LongString(s) => s.coerce(),
                Self::ByteArray(s) => s.coerce(),
                _ => None,
            }
        }
    }

    impl Coerce<'_, bool> for AMQPValue {
        fn coerce(&self) -> Option<bool> {
            match self {
                Self::Boolean(value) => Some(*value),
                Self::ShortShortUInt(u) => Some(*u != 0),
                Self::ShortUInt(u) => Some(*u != 0),
                Self::LongUInt(u) => Some(*u != 0),
                Self::Timestamp(u) => Some(*u != 0),
                Self::ShortShortInt(i) => Some(*i != 0),
                Self::ShortInt(i) => Some(*i != 0),
                Self::LongInt(i) => Some(*i != 0),
                Self::LongLongInt(i) => Some(*i != 0),
                Self::ShortString(s) => s.coerce(),
                Self::LongString(s) => s.coerce(),
                Self::ByteArray(s) => s.coerce(),
                _ => None,
            }
        }
    }

    impl Coerce<'_, i8> for AMQPValue {
        fn coerce(&self) -> Option<i8> {
            match self {
                Self::ShortShortUInt(u) => (*u).try_into().ok(),
                Self::ShortUInt(u) => (*u).try_into().ok(),
                Self::LongUInt(u) => (*u).try_into().ok(),
                Self::Timestamp(u) => (*u).try_into().ok(),
                Self::ShortShortInt(i) => Some(*i),
                Self::ShortInt(i) => (*i).try_into().ok(),
                Self::LongInt(i) => (*i).try_into().ok(),
                Self::LongLongInt(i) => (*i).try_into().ok(),
                Self::ShortString(s) => s.coerce(),
                Self::LongString(s) => s.coerce(),
                Self::ByteArray(s) => s.coerce(),
                _ => None,
            }
        }
    }

    impl Coerce<'_, i16> for AMQPValue {
        fn coerce(&self) -> Option<i16> {
            match self {
                Self::ShortShortUInt(u) => Some(*u as i16),
                Self::ShortUInt(u) => (*u).try_into().ok(),
                Self::LongUInt(u) => (*u).try_into().ok(),
                Self::Timestamp(u) => (*u).try_into().ok(),
                Self::ShortShortInt(i) => Some(*i as i16),
                Self::ShortInt(i) => Some(*i),
                Self::LongInt(i) => (*i).try_into().ok(),
                Self::LongLongInt(i) => (*i).try_into().ok(),
                Self::ShortString(s) => s.coerce(),
                Self::LongString(s) => s.coerce(),
                Self::ByteArray(s) => s.coerce(),
                _ => None,
            }
        }
    }

    impl Coerce<'_, i32> for AMQPValue {
        fn coerce(&self) -> Option<i32> {
            match self {
                Self::ShortShortUInt(u) => Some(*u as i32),
                Self::ShortUInt(u) => Some(*u as i32),
                Self::LongUInt(u) => (*u).try_into().ok(),
                Self::Timestamp(u) => (*u).try_into().ok(),
                Self::ShortShortInt(i) => Some(*i as i32),
                Self::ShortInt(i) => Some(*i as i32),
                Self::LongInt(i) => Some(*i),
                Self::LongLongInt(i) => (*i).try_into().ok(),
                Self::ShortString(s) => s.coerce(),
                Self::LongString(s) => s.coerce(),
                Self::ByteArray(s) => s.coerce(),
                _ => None,
            }
        }
    }

    impl Coerce<'_, i64> for AMQPValue {
        fn coerce(&self) -> Option<i64> {
            match self {
                Self::ShortShortUInt(u) => Some(*u as i64),
                Self::ShortUInt(u) => Some(*u as i64),
                Self::LongUInt(u) => Some(*u as i64),
                Self::Timestamp(u) => (*u).try_into().ok(),
                Self::ShortShortInt(i) => Some(*i as i64),
                Self::ShortInt(i) => Some(*i as i64),
                Self::LongInt(i) => Some(*i as i64),
                Self::LongLongInt(i) => Some(*i),
                Self::ShortString(s) => s.coerce(),
                Self::LongString(s) => s.coerce(),
                Self::ByteArray(s) => s.coerce(),
                _ => None,
            }
        }
    }

    impl Coerce<'_, isize> for AMQPValue {
        fn coerce(&self) -> Option<isize> {
            match self {
                Self::ShortShortUInt(u) => Some(*u as isize),
                Self::ShortUInt(u) => (*u).try_into().ok(),
                Self::LongUInt(u) => (*u).try_into().ok(),
                Self::Timestamp(u) => (*u).try_into().ok(),
                Self::ShortShortInt(i) => Some(*i as isize),
                Self::ShortInt(i) => Some(*i as isize),
                Self::LongInt(i) => Some(*i as isize),
                Self::LongLongInt(i) => (*i).try_into().ok(),
                Self::ShortString(s) => s.coerce(),
                Self::LongString(s) => s.coerce(),
                Self::ByteArray(s) => s.coerce(),
                _ => None,
            }
        }
    }

    impl Coerce<'_, u8> for AMQPValue {
        fn coerce(&self) -> Option<u8> {
            match self {
                Self::ShortShortUInt(u) => Some(*u),
                Self::ShortUInt(u) => (*u).try_into().ok(),
                Self::LongUInt(u) => (*u).try_into().ok(),
                Self::Timestamp(u) => (*u).try_into().ok(),
                Self::ShortShortInt(i) => (*i).try_into().ok(),
                Self::ShortInt(i) => (*i).try_into().ok(),
                Self::LongInt(i) => (*i).try_into().ok(),
                Self::LongLongInt(i) => (*i).try_into().ok(),
                Self::ShortString(s) => s.coerce(),
                Self::LongString(s) => s.coerce(),
                Self::ByteArray(s) => s.coerce(),
                _ => None,
            }
        }
    }

    impl Coerce<'_, u16> for AMQPValue {
        fn coerce(&self) -> Option<u16> {
            match self {
                Self::ShortShortUInt(u) => Some(*u as u16),
                Self::ShortUInt(u) => Some(*u),
                Self::LongUInt(u) => (*u).try_into().ok(),
                Self::Timestamp(u) => (*u).try_into().ok(),
                Self::ShortShortInt(i) => (*i).try_into().ok(),
                Self::ShortInt(i) => (*i).try_into().ok(),
                Self::LongInt(i) => (*i).try_into().ok(),
                Self::LongLongInt(i) => (*i).try_into().ok(),
                Self::ShortString(s) => s.coerce(),
                Self::LongString(s) => s.coerce(),
                Self::ByteArray(s) => s.coerce(),
                _ => None,
            }
        }
    }

    impl Coerce<'_, u32> for AMQPValue {
        fn coerce(&self) -> Option<u32> {
            match self {
                Self::ShortShortUInt(u) => Some(*u as u32),
                Self::ShortUInt(u) => Some(*u as u32),
                Self::LongUInt(u) => Some(*u),
                Self::Timestamp(u) => (*u).try_into().ok(),
                Self::ShortShortInt(i) => (*i).try_into().ok(),
                Self::ShortInt(i) => (*i).try_into().ok(),
                Self::LongInt(i) => (*i).try_into().ok(),
                Self::LongLongInt(i) => (*i).try_into().ok(),
                Self::ShortString(s) => s.coerce(),
                Self::LongString(s) => s.coerce(),
                Self::ByteArray(s) => s.coerce(),
                _ => None,
            }
        }
    }

    impl Coerce<'_, u64> for AMQPValue {
        fn coerce(&self) -> Option<u64> {
            match self {
                Self::ShortShortUInt(u) => Some(*u as u64),
                Self::ShortUInt(u) => Some(*u as u64),
                Self::LongUInt(u) => Some(*u as u64),
                Self::Timestamp(u) => Some(*u),
                Self::ShortShortInt(i) => (*i).try_into().ok(),
                Self::ShortInt(i) => (*i).try_into().ok(),
                Self::LongInt(i) => (*i).try_into().ok(),
                Self::LongLongInt(i) => (*i).try_into().ok(),
                Self::ShortString(s) => s.coerce(),
                Self::LongString(s) => s.coerce(),
                Self::ByteArray(s) => s.coerce(),
                _ => None,
            }
        }
    }

    impl Coerce<'_, usize> for AMQPValue {
        fn coerce(&self) -> Option<usize> {
            match self {
                Self::ShortShortUInt(u) => Some(*u as usize),
                Self::ShortUInt(u) => Some(*u as usize),
                Self::LongUInt(u) => Some(*u as usize),
                Self::Timestamp(u) => (*u).try_into().ok(),
                Self::ShortShortInt(i) => (*i).try_into().ok(),
                Self::ShortInt(i) => (*i).try_into().ok(),
                Self::LongInt(i) => (*i).try_into().ok(),
                Self::LongLongInt(i) => (*i).try_into().ok(),
                Self::ShortString(s) => s.coerce(),
                Self::LongString(s) => s.coerce(),
                Self::ByteArray(s) => s.coerce(),
                _ => None,
            }
        }
    }

    impl Coerce<'_, f32> for AMQPValue {
        fn coerce(&'_ self) -> Option<f32> {
            match self {
                Self::ShortShortUInt(u) => Some(*u as f32),
                Self::ShortUInt(u) => Some(*u as f32),
                Self::LongUInt(u) => {
                    if (*u) <= (MAX_CONT_INT_F32 as u32) {
                        Some(*u as f32)
                    } else {
                        None
                    }
                }
                Self::Timestamp(u) => {
                    if (*u) <= (MAX_CONT_INT_F32 as u64) {
                        Some(*u as f32)
                    } else {
                        None
                    }
                }
                Self::ShortShortInt(i) => Some(*i as f32),
                Self::ShortInt(i) => Some(*i as f32),
                Self::LongInt(i) => {
                    if (*i) >= MIN_CONT_INT_F32 && (*i) <= MAX_CONT_INT_F32 {
                        Some(*i as f32)
                    } else {
                        None
                    }
                }
                Self::LongLongInt(i) => {
                    if (*i) >= (MIN_CONT_INT_F32 as i64) && (*i) <= (MAX_CONT_INT_F32 as i64) {
                        Some(*i as f32)
                    } else {
                        None
                    }
                }
                Self::ShortString(s) => s.coerce(),
                Self::LongString(s) => s.coerce(),
                Self::ByteArray(s) => s.coerce(),
                _ => None,
            }
        }
    }

    impl Coerce<'_, f64> for AMQPValue {
        fn coerce(&'_ self) -> Option<f64> {
            match self {
                Self::ShortShortUInt(u) => Some(*u as f64),
                Self::ShortUInt(u) => Some(*u as f64),
                Self::LongUInt(u) => Some(*u as f64),
                Self::Timestamp(u) => {
                    if (*u) <= (MAX_CONT_INT_F64 as u64) {
                        Some(*u as f64)
                    } else {
                        None
                    }
                }
                Self::ShortShortInt(i) => Some(*i as f64),
                Self::ShortInt(i) => Some(*i as f64),
                Self::LongInt(i) => Some(*i as f64),
                Self::LongLongInt(i) => {
                    if (*i) >= MIN_CONT_INT_F64 && (*i) <= MAX_CONT_INT_F64 {
                        Some(*i as f64)
                    } else {
                        None
                    }
                }
                Self::ShortString(s) => s.coerce(),
                Self::LongString(s) => s.coerce(),
                Self::ByteArray(s) => s.coerce(),
                _ => None,
            }
        }
    }
};

/// Implement [`Coerce`] for [`ShortString`].
const _: () = {
    impl<'a> Coerce<'a, &'a str> for ShortString {
        fn coerce(&'a self) -> Option<&'a str> {
            Some(self.as_str())
        }
    }

    impl Coerce<'_, String> for ShortString {
        fn coerce(&self) -> Option<String> {
            Some(self.to_string())
        }
    }

    impl Coerce<'_, bool> for ShortString {
        fn coerce(&self) -> Option<bool> {
            parse_bool(self.as_str())
        }
    }

    impl Coerce<'_, i8> for ShortString {
        fn coerce(&self) -> Option<i8> {
            self.as_str().parse::<i8>().ok()
        }
    }

    impl Coerce<'_, i16> for ShortString {
        fn coerce(&self) -> Option<i16> {
            self.as_str().parse::<i16>().ok()
        }
    }

    impl Coerce<'_, i32> for ShortString {
        fn coerce(&self) -> Option<i32> {
            self.as_str().parse::<i32>().ok()
        }
    }

    impl Coerce<'_, i64> for ShortString {
        fn coerce(&self) -> Option<i64> {
            self.as_str().parse::<i64>().ok()
        }
    }

    impl Coerce<'_, isize> for ShortString {
        fn coerce(&self) -> Option<isize> {
            self.as_str().parse::<isize>().ok()
        }
    }

    impl Coerce<'_, u8> for ShortString {
        fn coerce(&self) -> Option<u8> {
            self.as_str().parse::<u8>().ok()
        }
    }

    impl Coerce<'_, u16> for ShortString {
        fn coerce(&self) -> Option<u16> {
            self.as_str().parse::<u16>().ok()
        }
    }

    impl Coerce<'_, u32> for ShortString {
        fn coerce(&self) -> Option<u32> {
            self.as_str().parse::<u32>().ok()
        }
    }

    impl Coerce<'_, u64> for ShortString {
        fn coerce(&self) -> Option<u64> {
            self.as_str().parse::<u64>().ok()
        }
    }

    impl Coerce<'_, usize> for ShortString {
        fn coerce(&self) -> Option<usize> {
            self.as_str().parse::<usize>().ok()
        }
    }

    impl Coerce<'_, f32> for ShortString {
        fn coerce(&self) -> Option<f32> {
            self.as_str().parse::<f32>().ok()
        }
    }

    impl Coerce<'_, f64> for ShortString {
        fn coerce(&self) -> Option<f64> {
            self.as_str().parse::<f64>().ok()
        }
    }
};

/// Implement [`Coerce`] for [`LongString`].
const _: () = {
    impl Coerce<'_, String> for LongString {
        fn coerce(&self) -> Option<String> {
            Some(self.to_string())
        }
    }

    impl Coerce<'_, bool> for LongString {
        fn coerce(&self) -> Option<bool> {
            parse_bool(&self.to_string())
        }
    }

    impl Coerce<'_, i8> for LongString {
        fn coerce(&self) -> Option<i8> {
            self.to_string().parse::<i8>().ok()
        }
    }

    impl Coerce<'_, i16> for LongString {
        fn coerce(&self) -> Option<i16> {
            self.to_string().parse::<i16>().ok()
        }
    }

    impl Coerce<'_, i32> for LongString {
        fn coerce(&self) -> Option<i32> {
            self.to_string().parse::<i32>().ok()
        }
    }

    impl Coerce<'_, i64> for LongString {
        fn coerce(&self) -> Option<i64> {
            self.to_string().parse::<i64>().ok()
        }
    }

    impl Coerce<'_, isize> for LongString {
        fn coerce(&self) -> Option<isize> {
            self.to_string().parse::<isize>().ok()
        }
    }

    impl Coerce<'_, u8> for LongString {
        fn coerce(&self) -> Option<u8> {
            self.to_string().parse::<u8>().ok()
        }
    }

    impl Coerce<'_, u16> for LongString {
        fn coerce(&self) -> Option<u16> {
            self.to_string().parse::<u16>().ok()
        }
    }

    impl Coerce<'_, u32> for LongString {
        fn coerce(&self) -> Option<u32> {
            self.to_string().parse::<u32>().ok()
        }
    }

    impl Coerce<'_, u64> for LongString {
        fn coerce(&self) -> Option<u64> {
            self.to_string().parse::<u64>().ok()
        }
    }

    impl Coerce<'_, usize> for LongString {
        fn coerce(&self) -> Option<usize> {
            self.to_string().parse::<usize>().ok()
        }
    }

    impl Coerce<'_, f32> for LongString {
        fn coerce(&self) -> Option<f32> {
            self.to_string().parse::<f32>().ok()
        }
    }

    impl Coerce<'_, f64> for LongString {
        fn coerce(&self) -> Option<f64> {
            self.to_string().parse::<f64>().ok()
        }
    }
};

/// Implement [`Coerce`] for [`ByteArray`].
const _: () = {
    impl Coerce<'_, String> for ByteArray {
        fn coerce(&self) -> Option<String> {
            Some(String::from_utf8_lossy(self.as_slice()).to_string())
        }
    }

    impl Coerce<'_, bool> for ByteArray {
        fn coerce(&self) -> Option<bool> {
            parse_bool(&String::from_utf8_lossy(self.as_slice()))
        }
    }

    impl Coerce<'_, i8> for ByteArray {
        fn coerce(&self) -> Option<i8> {
            String::from_utf8_lossy(self.as_slice()).parse::<i8>().ok()
        }
    }

    impl Coerce<'_, i16> for ByteArray {
        fn coerce(&self) -> Option<i16> {
            String::from_utf8_lossy(self.as_slice()).parse::<i16>().ok()
        }
    }

    impl Coerce<'_, i32> for ByteArray {
        fn coerce(&self) -> Option<i32> {
            String::from_utf8_lossy(self.as_slice()).parse::<i32>().ok()
        }
    }

    impl Coerce<'_, i64> for ByteArray {
        fn coerce(&self) -> Option<i64> {
            String::from_utf8_lossy(self.as_slice()).parse::<i64>().ok()
        }
    }

    impl Coerce<'_, isize> for ByteArray {
        fn coerce(&self) -> Option<isize> {
            String::from_utf8_lossy(self.as_slice())
                .parse::<isize>()
                .ok()
        }
    }

    impl Coerce<'_, u8> for ByteArray {
        fn coerce(&self) -> Option<u8> {
            String::from_utf8_lossy(self.as_slice()).parse::<u8>().ok()
        }
    }

    impl Coerce<'_, u16> for ByteArray {
        fn coerce(&self) -> Option<u16> {
            String::from_utf8_lossy(self.as_slice()).parse::<u16>().ok()
        }
    }

    impl Coerce<'_, u32> for ByteArray {
        fn coerce(&self) -> Option<u32> {
            String::from_utf8_lossy(self.as_slice()).parse::<u32>().ok()
        }
    }

    impl Coerce<'_, u64> for ByteArray {
        fn coerce(&self) -> Option<u64> {
            String::from_utf8_lossy(self.as_slice()).parse::<u64>().ok()
        }
    }

    impl Coerce<'_, usize> for ByteArray {
        fn coerce(&self) -> Option<usize> {
            String::from_utf8_lossy(self.as_slice())
                .parse::<usize>()
                .ok()
        }
    }

    impl Coerce<'_, f32> for ByteArray {
        fn coerce(&self) -> Option<f32> {
            String::from_utf8_lossy(self.as_slice()).parse::<f32>().ok()
        }
    }

    impl Coerce<'_, f64> for ByteArray {
        fn coerce(&self) -> Option<f64> {
            String::from_utf8_lossy(self.as_slice()).parse::<f64>().ok()
        }
    }
};

/// Internal helper for parsing a human-readable string into a `bool`.
fn parse_bool(input: &str) -> Option<bool> {
    match input.len() {
        0 => Some(false), // empty -> false

        1 => {
            // single-char: 1/0, t/f, y/n
            let b = input.as_bytes()[0];
            match b {
                b'1' => Some(true),
                b'0' => Some(false),
                _ => match b.to_ascii_lowercase() {
                    b't' | b'y' => Some(true),
                    b'f' | b'n' => Some(false),
                    _ => None,
                },
            }
        }

        2 if input.eq_ignore_ascii_case("on") => Some(true),
        2 if input.eq_ignore_ascii_case("no") => Some(false),

        3 if input.eq_ignore_ascii_case("yes") => Some(true),
        3 if input.eq_ignore_ascii_case("off") => Some(false),

        4 if input.eq_ignore_ascii_case("true") => Some(true),
        5 if input.eq_ignore_ascii_case("false") => Some(false),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::parse_bool;

    #[test]
    fn true_variants() {
        let trues = [
            "", //–– actually false, but we test empty separately below
            "true", "TRUE", "True", "t", "T", "1", "yes", "YES", "Yes", "y", "Y", "on", "ON", "On",
        ];
        for &s in &trues[1..] {
            assert_eq!(parse_bool(s), Some(true), "should parse {:?} as true", s);
        }
    }

    #[test]
    fn false_variants() {
        let falses = [
            "", // empty → false
            "false", "FALSE", "False", "f", "F", "0", "no", "NO", "No", "n", "N", "off", "OFF",
            "Off",
        ];
        for &s in &falses {
            assert_eq!(parse_bool(s), Some(false), "should parse {:?} as false", s);
        }
    }

    #[test]
    fn rejects_invalid() {
        let bad = [
            "2", "on?", "off!", "tru", "fals", "yep", "nah", "ye", "nOpe", "yes!", " false ",
        ];
        for &s in &bad {
            assert_eq!(parse_bool(s), None, "should reject {:?}", s);
        }
    }
}
