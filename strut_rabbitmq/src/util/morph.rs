use lapin::types::{AMQPValue, ShortString};

/// Artificial trait implemented for a few types like [`AMQPValue`] or
/// [`ShortString`] to allow infallibly morphing (instantiating) them from
/// standard Rust types.
pub trait Morph<T> {
    /// Flexibly generates an instance of `Self` from a value of type `T`.
    fn morph(from: T) -> Self;
}

/// Implement [`Morph`] for [`AMQPValue`].
const _: () = {
    impl Morph<bool> for AMQPValue {
        fn morph(from: bool) -> Self {
            Self::Boolean(from)
        }
    }

    impl Morph<&str> for AMQPValue {
        fn morph(from: &str) -> Self {
            Self::LongString(from.as_bytes().into())
        }
    }

    impl Morph<String> for AMQPValue {
        fn morph(from: String) -> Self {
            Self::LongString(from.into_bytes().into())
        }
    }

    impl Morph<i8> for AMQPValue {
        fn morph(from: i8) -> Self {
            Self::ShortShortInt(from)
        }
    }

    impl Morph<i16> for AMQPValue {
        fn morph(from: i16) -> Self {
            Self::ShortInt(from)
        }
    }

    impl Morph<i32> for AMQPValue {
        fn morph(from: i32) -> Self {
            Self::LongInt(from)
        }
    }

    impl Morph<i64> for AMQPValue {
        fn morph(from: i64) -> Self {
            Self::LongLongInt(from)
        }
    }

    impl Morph<isize> for AMQPValue {
        fn morph(from: isize) -> Self {
            Self::LongLongInt(from as i64)
        }
    }

    impl Morph<u8> for AMQPValue {
        fn morph(from: u8) -> Self {
            Self::ShortShortUInt(from)
        }
    }

    impl Morph<u16> for AMQPValue {
        fn morph(from: u16) -> Self {
            Self::ShortUInt(from)
        }
    }

    impl Morph<u32> for AMQPValue {
        fn morph(from: u32) -> Self {
            Self::LongUInt(from)
        }
    }

    impl Morph<u64> for AMQPValue {
        fn morph(from: u64) -> Self {
            Self::Timestamp(from)
        }
    }

    impl Morph<usize> for AMQPValue {
        fn morph(from: usize) -> Self {
            Self::Timestamp(from as u64)
        }
    }

    impl Morph<f32> for AMQPValue {
        fn morph(from: f32) -> Self {
            Self::Float(from)
        }
    }

    impl Morph<f64> for AMQPValue {
        fn morph(from: f64) -> Self {
            Self::Double(from)
        }
    }
};

/// Implement [`Morph`] for [`ShortString`].
const _: () = {
    impl Morph<bool> for ShortString {
        fn morph(from: bool) -> Self {
            from.to_string().into()
        }
    }

    impl Morph<&str> for ShortString {
        fn morph(from: &str) -> Self {
            from.into()
        }
    }

    impl Morph<String> for ShortString {
        fn morph(from: String) -> Self {
            from.into()
        }
    }

    impl Morph<i8> for ShortString {
        fn morph(from: i8) -> Self {
            from.to_string().into()
        }
    }

    impl Morph<i16> for ShortString {
        fn morph(from: i16) -> Self {
            from.to_string().into()
        }
    }

    impl Morph<i32> for ShortString {
        fn morph(from: i32) -> Self {
            from.to_string().into()
        }
    }

    impl Morph<i64> for ShortString {
        fn morph(from: i64) -> Self {
            from.to_string().into()
        }
    }

    impl Morph<isize> for ShortString {
        fn morph(from: isize) -> Self {
            from.to_string().into()
        }
    }

    impl Morph<u8> for ShortString {
        fn morph(from: u8) -> Self {
            from.to_string().into()
        }
    }

    impl Morph<u16> for ShortString {
        fn morph(from: u16) -> Self {
            from.to_string().into()
        }
    }

    impl Morph<u32> for ShortString {
        fn morph(from: u32) -> Self {
            from.to_string().into()
        }
    }

    impl Morph<u64> for ShortString {
        fn morph(from: u64) -> Self {
            from.to_string().into()
        }
    }

    impl Morph<usize> for ShortString {
        fn morph(from: usize) -> Self {
            from.to_string().into()
        }
    }

    impl Morph<f32> for ShortString {
        fn morph(from: f32) -> Self {
            from.to_string().into()
        }
    }

    impl Morph<f64> for ShortString {
        fn morph(from: f64) -> Self {
            from.to_string().into()
        }
    }
};
