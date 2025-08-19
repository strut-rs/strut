#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![cfg_attr(test, deny(warnings))]

/// Slug-related utilities
mod slug;
pub use self::slug::map::SlugMap;
pub use self::slug::Slug;

/// Helper enum for deserializing collections that can optionally be represented
/// by a single member
mod one_or_many;
pub use self::one_or_many::OneOrMany;
