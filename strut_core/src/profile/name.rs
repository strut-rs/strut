use std::any::type_name;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;

/// The maximum number of bytes (ASCII characters) allowed for a
/// [custom](crate::AppProfile::Custom) [`AppProfile`](crate::AppProfile) name.
///
/// This is limited quite severely to minimize how much space something as simple
/// as an application profile takes. Additionally, keeping the profile string at
/// 7 bytes allows for a nice layout of the underlying [`Name`] struct.
pub const NAME_MAX_LEN: usize = 7;

/// A compact, stack-allocated ASCII string to represent a custom
/// [`AppProfile`](crate::AppProfile) name.
///
/// This type is designed for strings that are:
///
/// - ASCII alphanumeric (other characters are ignored).
/// - No longer than [`NAME_MAX_LEN`] characters (extra characters are truncated).
/// - Lowercase (uppercase characters are converted).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Name {
    buf: [u8; NAME_MAX_LEN],
    len: u8,
}

impl Name {
    /// Creates a new [`Name`] from an ASCII string slice. Retains only ASCII
    /// alphanumeric characters from the input. Truncates the remaining input to
    /// [`NAME_MAX_LEN`] bytes to ensure it fits within the fixed-size buffer.
    /// Every taken character is forced to lowercase. Performs no heap
    /// allocations.
    pub(crate) fn new(input: impl AsRef<str>) -> Self {
        // Initialize storage variables
        let mut buf = [0u8; NAME_MAX_LEN];
        let mut len = 0usize;

        // Take one input ASCII character at a time while staying within length limit
        for mut b in input
            .as_ref()
            .bytes()
            .filter(u8::is_ascii_alphanumeric)
            .take(NAME_MAX_LEN)
        {
            b.make_ascii_lowercase(); // force input to lowercase
            buf[len] = b;
            len += 1;
        }

        Name {
            buf,
            len: len as u8,
        }
    }

    /// Exposes a view into this [`Name`] as a string slice.
    ///
    /// Guaranteed to return a valid string reference, as the
    /// [constructor](Name::new) enforces ASCII-only characters, and ASCII is a
    /// valid sub-set of UTF-8.
    ///
    /// This method does not allocate or perform any decoding; it simply returns
    /// a `&str` view into the internal buffer up to the stored length.
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.buf[..self.len as usize]).expect(concat!(
            "it should be possible to view the internal buffer as a &str because",
            " the constructor of this struct always interprets the input string",
            " as a sequence of valid UTF-8 characters",
        ))
    }
}

impl Debug for Name {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(type_name::<Self>())
            .field("buf", &self.as_str())
            .finish()
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl AsRef<str> for Name {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Deref for Name {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}
