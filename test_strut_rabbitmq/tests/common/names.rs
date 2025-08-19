use std::time::{SystemTime, UNIX_EPOCH};

pub const HEADER_KEY_A: &str = "test_header_a";
pub const HEADER_KEY_B: &str = "test_header_b";

pub const HIT: &str = "hit";
pub const MISS: &str = "miss";

/// Lumps together exchange and queue names.
pub struct ExQu {
    pub exchange: String,
    pub queue: String,
}

impl ExQu {
    pub fn from(v: &str) -> Self {
        Self {
            exchange: mangle(&[v, "exchange"]),
            queue: mangle(&[v, "queue"]),
        }
    }

    pub fn hit(v: &str) -> Self {
        Self {
            exchange: mangle(&[v, "exchange", HIT]),
            queue: mangle(&[v, "queue", HIT]),
        }
    }

    pub fn miss(v: &str) -> Self {
        Self {
            exchange: mangle(&[v, "exchange", MISS]),
            queue: mangle(&[v, "queue", MISS]),
        }
    }
}

/// Generates a random 6-character token to use as a globally unique name or
/// value.
pub fn random_token() -> String {
    use rand::Rng;

    rand::rng()
        .sample_iter(&rand::distr::Alphanumeric)
        .take(6)
        .map(char::from)
        .collect()
}

/// Generates a random **non-zero** `u32`. Zero can be used as a guaranteed
/// wrong value.
pub fn random_u32() -> u32 {
    use rand::Rng;

    loop {
        let v = rand::rng().random();

        if v != 0 {
            break v;
        }
    }
}

/// Adds a randomized suffix to the given name to make it globally unique.
pub fn mangle<T>(v: T) -> String
where
    T: MangleHelper,
{
    format!(
        "{}.{}.{}",
        v.meld(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        random_token(),
    )
}

/// Helper for implementing [`mangle`].
pub trait MangleHelper {
    fn meld(self) -> String;
}

/// Helper implementation for `&str`.
impl MangleHelper for &str {
    fn meld(self) -> String {
        self.replace("::tests::", "::").replace("::", ".")
    }
}

/// Helper implementation for `&[&str; 2]`.
impl MangleHelper for &[&str; 2] {
    fn meld(self) -> String {
        self.iter().map(|s| s.meld()).collect::<Vec<_>>().join(".")
    }
}

/// Helper implementation for `&[&str; 3]`.
impl MangleHelper for &[&str; 3] {
    fn meld(self) -> String {
        self.iter().map(|s| s.meld()).collect::<Vec<_>>().join(".")
    }
}
