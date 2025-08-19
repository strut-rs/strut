use crate::Slug;
use std::collections::HashMap;

/// An immutable [`HashMap`] that uses [`Slug`]s as keys and allows efficiently
/// searching for matching entries by string reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlugMap<V> {
    map: HashMap<Slug, V>,
    keys: Vec<Slug>,
}

impl<V> SlugMap<V> {
    /// Creates an empty [`SlugMap`].
    pub fn empty() -> Self {
        Self {
            map: HashMap::new(),
            keys: Vec::new(),
        }
    }

    /// Consumes and transforms the given [`HashMap`] with [`Slug`] keys into
    /// a [`SlugMap`].
    pub fn new(input: HashMap<Slug, V>) -> Self {
        // Collect the keys
        let mut keys = input.keys().cloned().collect::<Vec<_>>();

        // IMPORTANT: store the keys sorted
        keys.sort();

        Self { map: input, keys }
    }

    /// Consumes and transforms the given [`HashMap`] with [`String`] keys into
    /// a [`SlugMap`].
    ///
    /// If any two keys in the input resolve to the same [`Slug`] — only one of
    /// two values is retained.
    pub fn from(input: HashMap<String, V>) -> Self {
        // Prepare storage
        let mut map: HashMap<Slug, V> = HashMap::with_capacity(input.len());

        // Convert string keys to slug keys, consuming the input map
        for (k, v) in input {
            map.insert(Slug::new(k), v);
        }

        Self::new(map)
    }

    /// Consumes and transforms the given [`HashMap`] with [`String`] keys into
    /// a [`SlugMap`].
    ///
    /// If any two keys in the input resolve to the same [`Slug`] — the two
    /// values are consumed and merged using the provided `zipper` function.
    pub fn zip<F>(input: HashMap<String, V>, mut zipper: F) -> Self
    where
        F: FnMut(V, V) -> V,
    {
        // Prepare storage
        let mut map: HashMap<Slug, V> = HashMap::with_capacity(input.len());

        // Convert string keys to slug keys, consuming the input map
        for (k, v) in input {
            let slug = Slug::new(k);

            if let Some(existing) = map.remove(&slug) {
                let merged = zipper(existing, v);
                map.insert(slug, merged);
            } else {
                map.insert(slug, v);
            }
        }

        Self::new(map)
    }
}

impl<V> SlugMap<V> {
    /// Returns `true` if the map contains a value for the specified key,
    /// comparing the keys as [`Slug`]s.
    pub fn contains_key(&self, key: impl AsRef<str>) -> bool {
        self.find_key(key).is_some()
    }

    /// Returns a reference to the value corresponding to the key, comparing the
    /// keys as [`Slug`]s.
    ///
    /// If the map contains a value for the [`Slug`] key `"somekey"`, then
    /// calling this method with anything that compares equally to that slug
    /// (e.g., `"SomeKey"` or `"__some_key"`) will find and return that value.
    pub fn get(&self, key: impl AsRef<str>) -> Option<&V> {
        self.find_key(key)
            .and_then(|found_key| self.map.get(found_key))
    }

    /// Returns a [`Slug`] contained in this map and matching the given `key`,
    /// if one exists.
    fn find_key(&self, key: impl AsRef<str>) -> Option<&Slug> {
        let input = key.as_ref();

        self.keys
            .binary_search_by(|current_key| Slug::cmp_as_slugs(current_key, input))
            .ok()
            .map(|found_index| &self.keys[found_index])
    }

    /// Maps the values of this [`SlugMap`] using the given `mapper` function.
    /// Leaves the keys unchanged.
    pub fn map<NV, F>(self, mut mapper: F) -> SlugMap<NV>
    where
        F: FnMut(&Slug, V) -> NV,
    {
        let mut map = HashMap::with_capacity(self.map.len());

        for (k, v) in self.map {
            let new_value = mapper(&k, v);
            map.insert(k, new_value);
        }

        SlugMap {
            map,
            keys: self.keys,
        }
    }
}

const _: () = {
    impl<V> Default for SlugMap<V> {
        fn default() -> Self {
            Self::empty()
        }
    }

    impl<S, V> FromIterator<(S, V)> for SlugMap<V>
    where
        S: Into<Slug>,
    {
        fn from_iter<T: IntoIterator<Item = (S, V)>>(iter: T) -> Self {
            let handles = iter.into_iter().map(|(k, v)| (k.into(), v)).collect();

            Self::new(handles)
        }
    }

    impl<const N: usize, S, V> From<[(S, V); N]> for SlugMap<V>
    where
        S: Into<Slug>,
    {
        fn from(value: [(S, V); N]) -> Self {
            value.into_iter().collect()
        }
    }
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn new_get() {
        // Given
        let map = HashMap::from([
            (Slug::new("HelloWorld"), 42),
            (Slug::new("test123"), 7),
            (Slug::new("++abc"), 999),
        ]);

        // When
        let slug_map = SlugMap::new(map);

        // Then
        assert!(slug_map.contains_key("abc"));
        assert!(slug_map.contains_key("HELLO_WORLD"));
        assert!(slug_map.contains_key("hello-world"));
        assert!(slug_map.contains_key("test_123"));
        assert!(slug_map.contains_key("TEST123"));
        assert!(!slug_map.contains_key("notfound"));

        assert_eq!(
            slug_map.keys,
            vec![
                Slug::new("abc"),
                Slug::new("helloworld"),
                Slug::new("test123")
            ],
        );
        assert_eq!(slug_map.get("abc"), Some(&999));
        assert_eq!(slug_map.get("HELLO_WORLD"), Some(&42));
        assert_eq!(slug_map.get("hello-world"), Some(&42));
        assert_eq!(slug_map.get("test_123"), Some(&7));
        assert_eq!(slug_map.get("TEST123"), Some(&7));
        assert_eq!(slug_map.get("notfound"), None);
    }

    #[test]
    fn from() {
        // Given
        let map = HashMap::from([
            ("HelloWorld".to_string(), 1),
            ("HELLO_WORLD".to_string(), 2),
            ("test123".to_string(), 3),
        ]);

        // When
        let slug_map = SlugMap::from(map);

        // Then
        assert!(matches!(slug_map.get("HelloWorld"), Some(_)));
        assert!(matches!(slug_map.get("HELLO_WORLD"), Some(_)));
        assert!(matches!(slug_map.get("!HelloWorld+"), Some(_)));
        assert!(matches!(slug_map.get("hellO()World"), Some(_)));
        assert_eq!(slug_map.get("HelloWorld"), slug_map.get("HELLO_WORLD"));
        assert_eq!(slug_map.get("!HelloWorld+"), slug_map.get("hellO()World"));
        assert_eq!(slug_map.get("test123"), Some(&3));
        assert_eq!(slug_map.get("TEST123"), Some(&3));
        assert_eq!(slug_map.get("notfound"), None);
    }

    #[test]
    fn zip() {
        // Given
        let map = HashMap::from([
            ("HelloWorld".to_string(), 1),
            ("HELLO_WORLD".to_string(), 2),
            ("test123".to_string(), 3),
            ("TEST123".to_string(), 5),
        ]);

        // When
        let slug_map = SlugMap::zip(map, |a, b| a + b);

        // Then
        assert_eq!(slug_map.get("hello_world"), Some(&3));
        assert_eq!(slug_map.get("test123"), Some(&8));
        assert_eq!(slug_map.get("notfound"), None);
    }

    #[test]
    fn zip_multiple() {
        // Given
        let map = HashMap::from([
            ("A".to_string(), 1),
            ("a".to_string(), 2),
            ("A!".to_string(), 3),
        ]);

        // When
        let slug_map = SlugMap::zip(map, |a, b| a * b);

        // Then
        assert_eq!(slug_map.get("--a"), Some(&6));
    }

    #[test]
    fn test_empty_map() {
        // Given
        let map: HashMap<Slug, i32> = HashMap::new();

        // When
        let slug_map = SlugMap::new(map);

        // Then
        assert_eq!(slug_map.keys.len(), 0);
        assert_eq!(slug_map.get("anything"), None);
    }

    #[test]
    fn only_non_alphanumeric() {
        // Given
        let map = HashMap::from([("!!!".to_string(), 99), ("***".to_string(), 42)]);

        // When
        let slug_map = SlugMap::from(map);

        // Then
        assert!(matches!(slug_map.get(""), Some(_)));
        assert!(matches!(slug_map.get("!!!"), Some(_)));
        assert!(matches!(slug_map.get("***"), Some(_)));
        assert_eq!(slug_map.get(""), slug_map.get("!!!"));
        assert_eq!(slug_map.get("***"), slug_map.get("!!!"));
    }

    #[test]
    fn map() {
        // Given
        let map = HashMap::from([
            (Slug::new("HelloWorld"), 42),
            (Slug::new("test123"), -7),
            (Slug::new("++abc"), 999),
        ]);
        let slug_map = SlugMap::new(map);
        fn get<'a>(map: &'a SlugMap<String>, key: &str) -> Option<&'a str> {
            map.get(key).map(|v| v.as_str())
        }

        // When
        let mapped = slug_map.map(|key, value| format!("{}-{}", key, value));

        // Then
        assert_eq!(get(&mapped, "hello_world"), Some("helloworld-42"),);
        assert_eq!(get(&mapped, "test123"), Some("test123--7"));
        assert_eq!(get(&mapped, "TEST123"), Some("test123--7"));
        assert_eq!(get(&mapped, "++abc"), Some("abc-999"));
        assert_eq!(get(&mapped, "abc"), Some("abc-999"));
    }
}
