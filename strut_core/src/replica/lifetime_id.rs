use std::any::type_name;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::SystemTime;

/// Ten pseudo-random lower-case ASCII letters with a few visually digestible
/// string representations. **Not** cryptographically secure.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LifetimeId {
    bytes: [u8; 10],
}

impl LifetimeId {
    /// Generates a pseudo-randomized, non-secure [`LifetimeId`].
    pub fn random() -> Self {
        // Prepare storage for the pseudo-random bytes
        let mut bytes = [0u8; 10];

        // Prepare the hash holder variable
        let mut hash = 0u64;

        // Static set for picking random characters
        static CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz";

        // Repeat for each random character in lifetime ID
        for i in 0..bytes.len() {
            // Refresh the hash when necessary
            if hash == 0 {
                hash = Self::make_hash();
            }

            // Pick a pseudo-random index
            let idx = (hash % CHARSET.len() as u64) as usize;

            // Push the character at the pseudo-random index
            bytes[i] = CHARSET[idx];

            // Shift a few bits to get new data
            hash >>= 5;
        }

        Self { bytes }
    }

    /// Returns a [`Hyphenated`] version of this [`LifetimeId`].
    pub fn hyphenated(&self) -> Hyphenated<'_> {
        Hyphenated(self)
    }

    /// Returns an [`Underscored`] version of this [`LifetimeId`].
    pub fn underscored(&self) -> Underscored<'_> {
        Underscored(self)
    }

    /// Returns an [`Dotted`] version of this [`LifetimeId`].
    pub fn dotted(&self) -> Dotted<'_> {
        Dotted(self)
    }

    /// Returns a [`Glued`] version of this [`LifetimeId`].
    pub fn glued(&self) -> Glued<'_> {
        Glued(self)
    }
}

impl LifetimeId {
    /// Generates a pseudo-random `u64` that is **not** cryptographically secure.
    fn make_hash() -> u64 {
        // Make a seed such as the current time in nanoseconds
        let seed = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("current time should always be after UNIX epoch")
            .as_nanos(); // nanoseconds provide more entropy

        // Hash the nanoseconds
        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);

        hasher.finish()
    }

    /// Exposes an immutable view of the internally held bytes.
    pub fn view_bytes(&self) -> &[u8; 10] {
        &self.bytes
    }

    /// Exposes an immutable view of the internally held bytes as a single
    /// ten-character string reference.
    pub fn view_glued(&self) -> &str {
        std::str::from_utf8(&self.bytes).expect(concat!(
            "it should be possible to view the internal buffer as a &str because",
            " the constructor of this struct always interprets the input string",
            " as a sequence of valid UTF-8 characters",
        ))
    }

    /// Exposes an immutable view of the internally held bytes as three string
    /// references: three, four, and three character long.
    pub fn view_chunks(&self) -> (&str, &str, &str) {
        let chunk_a = std::str::from_utf8(&self.bytes[0..3]).expect(concat!(
            "it should be possible to view the internal buffer as a &str because",
            " the constructor of this struct always interprets the input string",
            " as a sequence of valid UTF-8 characters",
        ));
        let chunk_b = std::str::from_utf8(&self.bytes[3..7]).expect(concat!(
            "it should be possible to view the internal buffer as a &str because",
            " the constructor of this struct always interprets the input string",
            " as a sequence of valid UTF-8 characters",
        ));
        let chunk_c = std::str::from_utf8(&self.bytes[7..10]).expect(concat!(
            "it should be possible to view the internal buffer as a &str because",
            " the constructor of this struct always interprets the input string",
            " as a sequence of valid UTF-8 characters",
        ));

        (chunk_a, chunk_b, chunk_c)
    }
}

impl Display for LifetimeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.hyphenated(), f)
    }
}

impl Debug for LifetimeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(type_name::<Self>())
            .field("bytes", &self.hyphenated())
            .finish()
    }
}

/// A wrapped [`LifetimeId`] that implements [`Display`] by writing the ID in
/// three chunks, separated by the hyphen characters. Writes exactly twelve
/// ASCII characters.
pub struct Hyphenated<'a>(&'a LifetimeId);

impl<'a> Display for Hyphenated<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (chunk_a, chunk_b, chunk_c) = self.0.view_chunks();

        write!(f, "{}-{}-{}", chunk_a, chunk_b, chunk_c)
    }
}

impl<'a> Debug for Hyphenated<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

/// A wrapped [`LifetimeId`] that implements [`Display`] by writing the ID in
/// three chunks, separated by the underscore characters. Writes exactly twelve
/// ASCII characters.
pub struct Underscored<'a>(&'a LifetimeId);

impl<'a> Display for Underscored<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (chunk_a, chunk_b, chunk_c) = self.0.view_chunks();

        write!(f, "{}_{}_{}", chunk_a, chunk_b, chunk_c)
    }
}

impl<'a> Debug for Underscored<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

/// A wrapped [`LifetimeId`] that implements [`Display`] by writing the ID in
/// three chunks, separated by the dot (full stop) characters. Writes exactly
/// twelve ASCII characters.
pub struct Dotted<'a>(&'a LifetimeId);

impl<'a> Display for Dotted<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (chunk_a, chunk_b, chunk_c) = self.0.view_chunks();

        write!(f, "{}.{}.{}", chunk_a, chunk_b, chunk_c)
    }
}

impl<'a> Debug for Dotted<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

/// A wrapped [`LifetimeId`] that implements [`Display`] by writing the ID in an
/// unbroken chunk. Writes exactly twelve ASCII characters.
pub struct Glued<'a>(&'a LifetimeId);

impl<'a> Display for Glued<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.view_glued())
    }
}

impl<'a> Debug for Glued<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    fn generate_lifetime_id() {
        // When
        let lifetime_id = super::LifetimeId::random().to_string();

        // Then
        assert_eq!(lifetime_id.len(), 12);
        assert!(
            lifetime_id
                .chars()
                .all(|c| c.is_ascii_alphabetic() || c == '-')
        );
        assert!(
            lifetime_id
                .chars()
                .all(|c| c.is_ascii_lowercase() || c == '-')
        );
        assert!(
            matches!(lifetime_id.chars().nth(0), Some(c) if c.is_ascii_lowercase() && c.is_ascii_lowercase())
        );
        assert!(
            matches!(lifetime_id.chars().nth(1), Some(c) if c.is_ascii_lowercase() && c.is_ascii_lowercase())
        );
        assert!(
            matches!(lifetime_id.chars().nth(2), Some(c) if c.is_ascii_lowercase() && c.is_ascii_lowercase())
        );
        assert!(matches!(lifetime_id.chars().nth(3), Some(c) if c == '-'));
        assert!(
            matches!(lifetime_id.chars().nth(4), Some(c) if c.is_ascii_lowercase() && c.is_ascii_lowercase())
        );
        assert!(
            matches!(lifetime_id.chars().nth(5), Some(c) if c.is_ascii_lowercase() && c.is_ascii_lowercase())
        );
        assert!(
            matches!(lifetime_id.chars().nth(6), Some(c) if c.is_ascii_lowercase() && c.is_ascii_lowercase())
        );
        assert!(
            matches!(lifetime_id.chars().nth(7), Some(c) if c.is_ascii_lowercase() && c.is_ascii_lowercase())
        );
        assert!(matches!(lifetime_id.chars().nth(8), Some(c) if c == '-'));
        assert!(
            matches!(lifetime_id.chars().nth(9), Some(c) if c.is_ascii_lowercase() && c.is_ascii_lowercase())
        );
        assert!(
            matches!(lifetime_id.chars().nth(10), Some(c) if c.is_ascii_lowercase() && c.is_ascii_lowercase())
        );
        assert!(
            matches!(lifetime_id.chars().nth(11), Some(c) if c.is_ascii_lowercase() && c.is_ascii_lowercase())
        );
    }

    #[test]
    fn generate_lifetime_ids() {
        // When
        let lifetime_id_a = super::LifetimeId::random().to_string();
        let lifetime_id_b = super::LifetimeId::random().to_string();

        // Then
        assert_ne!(lifetime_id_a, lifetime_id_b);
    }
}
