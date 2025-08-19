use serde::Deserialize;
use std::collections::{BTreeSet, HashSet};
use std::hash::Hash;
use std::vec::IntoIter;

/// Represents a deserializable collection of `T` that may be also trivially
/// deserialized from a single instance of `T`.
#[derive(Deserialize)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    /// A single instance of `T`.
    One(T),
    /// A collection of `T`.
    Many(Vec<T>),
}

impl<T> OneOrMany<T> {
    /// Returns the expected number of elements held in this collection,
    /// regardless of representation.
    pub fn len(&self) -> usize {
        match self {
            OneOrMany::One(_) => 1,
            OneOrMany::Many(many) => many.len(),
        }
    }
}

impl<T> IntoIterator for OneOrMany<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::One(one) => vec![one].into_iter(),
            Self::Many(many) => many.into_iter(),
        }
    }
}

impl<T> From<OneOrMany<T>> for Vec<T> {
    fn from(value: OneOrMany<T>) -> Self {
        match value {
            OneOrMany::One(one) => vec![one],
            OneOrMany::Many(many) => many,
        }
    }
}

impl<T> From<OneOrMany<T>> for HashSet<T>
where
    T: Eq + Hash,
{
    fn from(value: OneOrMany<T>) -> Self {
        let mut result = HashSet::with_capacity(value.len());

        for elem in value.into_iter() {
            result.insert(elem);
        }

        result
    }
}

impl<T> From<OneOrMany<T>> for BTreeSet<T>
where
    T: Ord,
{
    fn from(value: OneOrMany<T>) -> Self {
        let mut result = BTreeSet::new();

        for elem in value.into_iter() {
            result.insert(elem);
        }

        result
    }
}
