use serde::de::{Error, MapAccess};
use serde_value::Value;
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Deref;

pub mod map;

/// An owned [`String`] slug that retains only ASCII alphanumeric characters and
/// forces the retained characters to lowercase.
///
/// This is intended as a normalized identifier that only stores the relevant
/// textual part of the input. This makes inputs like `"SECTION_TITLE"`,
/// `"sectiontitle"`, `"++SectionTitle"`, etc. all equivalent.
///
/// ## Ambiguity
///
/// The accepted drawback is that this leaves no way to differentiate slugs only
/// by punctuation: e.g. `"re-sign"` (sign again) and `"resign"` (quit) are
/// treated as the same slug. Same for `"Unit_S"` and `"units"`.
///
/// It is expected that the slugs are chosen with their normalization rules in
/// mind.
#[derive(Debug, Clone)]
pub struct Slug {
    original: String,
    normalized: String,
}

impl Slug {
    /// Creates a new [`Slug`] from the given [`String`]-like input, possibly
    /// modifying the input in-place. The original input is also retained.
    pub fn new(slug: impl Into<String>) -> Self {
        let original = slug.into();
        let mut normalized = original.clone();

        normalized.retain(|c| c.is_ascii_alphanumeric());
        normalized.make_ascii_lowercase();

        Self {
            original,
            normalized,
        }
    }

    /// Exposes the original string from which this [`Slug`] was created.
    pub fn original(&self) -> &str {
        &self.original
    }

    /// Exposes the normalized string version of this [`Slug`], which is its
    /// primary string representation.
    pub fn normalized(&self) -> &str {
        &self.normalized
    }
}

impl Slug {
    /// Produces the same result as checking two [`Slug`]s for equivalence
    /// without allocating any [`Slug`]s.
    ///
    /// Two slugs are equivalent when their ASCII alphanumeric characters match
    /// pairwise (case-insensitively), while all other characters in both slugs
    /// are ignored. Under these rules, the following versions of the same slug
    /// are all equivalent:
    ///
    /// - `"MULTI_WORD_SLUG"`
    /// - `"MultiWordSlug"`
    /// - `"multiwordslug"`
    /// - `"++multi-word-slug!"`
    /// - etc.
    pub fn eq_as_slugs(a: &str, b: &str) -> bool {
        let mut iter_a = a.chars().filter(|&c| c.is_ascii_alphanumeric());
        let mut iter_b = b.chars().filter(|&c| c.is_ascii_alphanumeric());

        loop {
            match (iter_a.next(), iter_b.next()) {
                (Some(c1), Some(c2)) => {
                    if !c1.eq_ignore_ascii_case(&c2) {
                        return false;
                    }
                }
                (None, None) => return true, // both sides reached the end
                _ => return false, // one side reached the end, but the other still has valid characters
            }
        }
    }

    /// Produces the same result as comparing two [`Slug`]s for total ordering
    /// without allocating any [`Slug`]s.
    ///
    /// Two slugs are totally ordered only by their ASCII alphanumeric
    /// characters, pairwise (case-insensitively), while all other characters in
    /// both slugs are ignored.
    pub fn cmp_as_slugs(a: &str, b: &str) -> Ordering {
        let mut iter_a = a.chars().filter(|&c| c.is_ascii_alphanumeric());
        let mut iter_b = b.chars().filter(|&c| c.is_ascii_alphanumeric());

        loop {
            match (iter_a.next(), iter_b.next()) {
                (Some(mut c1), Some(mut c2)) => {
                    // Force copied chars on both sides to lowercase in-place
                    c1.make_ascii_lowercase();
                    c2.make_ascii_lowercase();

                    // Now compare
                    match c1.cmp(&c2) {
                        Ordering::Equal => continue,
                        non_eq => return non_eq,
                    }
                }
                (None, None) => return Ordering::Equal,
                (None, Some(_)) => return Ordering::Less,
                (Some(_), None) => return Ordering::Greater,
            }
        }
    }
}

impl Slug {
    /// Consumes the given [`MapAccess`] and materializes it into a [`HashMap`].
    /// Merges the map’s [`Value`]s at colliding [`Slug`] keys, but only if both
    /// nested values are [maps](Value::Map); otherwise an error is returned.
    ///
    /// This is intended as a helper method to group up deserialization inputs
    /// before deserializing the values and (presumably) putting them into a
    /// [`SlugMap`](crate::SlugMap).
    ///
    /// ## Error
    ///
    /// Returns an error if the given input cannot be deserialized as a map, or
    /// if at least one of the values at colliding keys is not a nested map.
    pub fn group_map<'de, A>(mut input: A) -> Result<HashMap<Slug, Value>, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut merged_values: HashMap<Slug, Value> = if let Some(len) = input.size_hint() {
            HashMap::with_capacity(len)
        } else {
            HashMap::new()
        };

        while let Some((next_key, next_value)) = input.next_entry::<String, Value>()? {
            let next_slug = Slug::new(next_key);

            if let Some((existing_slug, existing_value)) = merged_values.remove_entry(&next_slug) {
                // Merge the values
                let merged_value = match (existing_value, next_value) {
                    // If both values are maps, merge them
                    (Value::Map(mut existing_map), Value::Map(mut next_map)) => {
                        // Order matters
                        if existing_slug.original.len() >= next_slug.original.len() {
                            existing_map.extend(next_map);
                            Value::Map(existing_map)
                        } else {
                            next_map.extend(existing_map);
                            Value::Map(next_map)
                        }
                    }

                    // Otherwise, return an error
                    (a, b) => {
                        return Err(Error::custom(format!(
                            "collision for key {}: cannot merge values of type {:?} and {:?}",
                            existing_slug, a, b,
                        )));
                    }
                };

                // Pick the slug with the longest value
                let merged_slug = if existing_slug.original.len() >= next_slug.original.len() {
                    existing_slug
                } else {
                    next_slug
                };

                // Insert the merged value back
                merged_values.insert(merged_slug, merged_value);
            } else {
                // Insert the next value
                merged_values.insert(next_slug, next_value);
            }
        }

        Ok(merged_values)
    }
}

const _: () = {
    impl PartialEq for Slug {
        fn eq(&self, other: &Self) -> bool {
            self.normalized.eq(&other.normalized)
        }
    }

    impl Eq for Slug {}

    impl PartialOrd for Slug {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            self.normalized.partial_cmp(&other.normalized)
        }
    }

    impl Ord for Slug {
        fn cmp(&self, other: &Self) -> Ordering {
            self.normalized.cmp(&other.normalized)
        }
    }

    impl Hash for Slug {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.normalized.hash(state);
        }
    }

    impl From<String> for Slug {
        fn from(value: String) -> Self {
            Self::new(value)
        }
    }

    impl From<&str> for Slug {
        fn from(value: &str) -> Self {
            Self::new(value)
        }
    }

    impl Borrow<str> for Slug {
        fn borrow(&self) -> &str {
            &self.normalized
        }
    }

    impl AsRef<str> for Slug {
        fn as_ref(&self) -> &str {
            &self.normalized
        }
    }

    impl Deref for Slug {
        type Target = str;

        fn deref(&self) -> &Self::Target {
            &self.normalized
        }
    }

    impl From<Slug> for String {
        fn from(value: Slug) -> Self {
            value.normalized
        }
    }

    impl Display for Slug {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            f.write_str(&self.normalized)
        }
    }
};

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn normalized() {
        // Identity for already-normalized lowercase
        assert_eq!(Slug::new("input").as_ref(), "input");

        // Uppercase is lowercased
        assert_eq!(Slug::new("INPUT").as_ref(), "input");

        // Mixed case is lowercased
        assert_eq!(Slug::new("InPuT").as_ref(), "input");

        // Underscores removed
        assert_eq!(Slug::new("in_put").as_ref(), "input");

        // Dashes removed
        assert_eq!(Slug::new("in-put").as_ref(), "input");

        // Spaces removed
        assert_eq!(Slug::new("in put").as_ref(), "input");

        // Mixed punctuation removed
        assert_eq!(Slug::new("in!p@u#t$").as_ref(), "input");

        // Numbers retained
        assert_eq!(Slug::new("in123put").as_ref(), "in123put");

        // Mixed numbers and symbols
        assert_eq!(Slug::new("i!n@1#2$3%p^u&t*").as_ref(), "in123put");

        // Only symbols -> empty
        assert_eq!(Slug::new("!@#$%^&*()").as_ref(), "");

        // Only whitespace -> empty
        assert_eq!(Slug::new("     ").as_ref(), "");

        // Only underscores/dashes -> empty
        assert_eq!(Slug::new("___---").as_ref(), "");

        // Unicode letters removed
        assert_eq!(Slug::new("áβç").as_ref(), "");

        // Mixed ASCII and Unicode, keep ASCII
        assert_eq!(Slug::new("aβc").as_ref(), "ac");

        // Leading/trailing punctuation
        assert_eq!(Slug::new("!!input!!").as_ref(), "input");

        // Leading/trailing whitespace
        assert_eq!(Slug::new("  input  ").as_ref(), "input");

        // Mixed everything
        assert_eq!(Slug::new("++In_PuT-123!@#").as_ref(), "input123");

        // Empty string
        assert_eq!(Slug::new("").as_ref(), "");

        // Long string with mixed content
        assert_eq!(
            Slug::new("++SectionTitle_2025! Rust-lang --_β_ä_ç_1234").as_ref(),
            "sectiontitle2025rustlang1234",
        );
    }

    #[test]
    fn std_vs_custom() {
        assert_eq_and_cmp("", "", Ordering::Equal);
        assert_eq_and_cmp("a", "A", Ordering::Equal);
        assert_eq_and_cmp("A", "a", Ordering::Equal);
        assert_eq_and_cmp("abc", "ABC", Ordering::Equal);
        assert_eq_and_cmp("abc", "a_b_c", Ordering::Equal);
        assert_eq_and_cmp("abc", "a-b-c", Ordering::Equal);
        assert_eq_and_cmp("abc", "a b c", Ordering::Equal);
        assert_eq_and_cmp("abc", "a!b@c#", Ordering::Equal);
        assert_eq_and_cmp("abc", "a_b-c!", Ordering::Equal);
        assert_eq_and_cmp("sectiontitle", "SECTION_TITLE", Ordering::Equal);
        assert_eq_and_cmp("sectiontitle", "sectiontitle", Ordering::Equal);
        assert_eq_and_cmp("sectiontitle", "++SectionTitle", Ordering::Equal);
        assert_eq_and_cmp("multiwordslug", "MULTI_WORD_SLUG", Ordering::Equal);
        assert_eq_and_cmp("multiwordslug", "MultiWordSlug", Ordering::Equal);
        assert_eq_and_cmp("multiwordslug", "multiwordslug", Ordering::Equal);
        assert_eq_and_cmp("multiwordslug", "++multi-word-slug!", Ordering::Equal);
        assert_eq_and_cmp("units", "Unit_S", Ordering::Equal);
        assert_eq_and_cmp("resign", "re-sign", Ordering::Equal);
        assert_eq_and_cmp("abc123", "a_b_c_1_2_3", Ordering::Equal);
        assert_eq_and_cmp("abc123", "A B C 1 2 3", Ordering::Equal);
        assert_eq_and_cmp("abc123", "abc123", Ordering::Equal);
        assert_eq_and_cmp("abc123", "ABC123", Ordering::Equal);
        assert_eq_and_cmp("abc", "a!@#$%^&*()_+b{}:\"|?><c", Ordering::Equal);
        assert_eq_and_cmp("abc", "A!@#$%^&*()_+B{}:\"|?><C", Ordering::Equal);
        assert_eq_and_cmp("abc", "a_b_c", Ordering::Equal);
        assert_eq_and_cmp("abc", "A-B-C", Ordering::Equal);
        assert_eq_and_cmp("abc", "a b c", Ordering::Equal);
        assert_eq_and_cmp("abc", "A B C", Ordering::Equal);
        assert_eq_and_cmp("abc", "a!b@c#", Ordering::Equal);

        assert_eq_and_cmp("", "a", Ordering::Less);
        assert_eq_and_cmp("a", "b", Ordering::Less);
        assert_eq_and_cmp("a", "A", Ordering::Equal);
        assert_eq_and_cmp("abc", "abd", Ordering::Less);
        assert_eq_and_cmp("abc", "abc1", Ordering::Less);
        assert_eq_and_cmp("abc1", "abc2", Ordering::Less);
        assert_eq_and_cmp("abc", "abcd", Ordering::Less);
        assert_eq_and_cmp("abcd", "abc", Ordering::Greater);
        assert_eq_and_cmp("abc", "a_b_c_d", Ordering::Less);
        assert_eq_and_cmp("abc", "a-b-c-d", Ordering::Less);
        assert_eq_and_cmp("abc", "a b c d", Ordering::Less);
        assert_eq_and_cmp("abc1", "abc", Ordering::Greater);
        assert_eq_and_cmp("abc", "abc", Ordering::Equal);

        assert_eq_and_cmp("abc", "abd", Ordering::Less);
        assert_eq_and_cmp("abd", "abc", Ordering::Greater);
        assert_eq_and_cmp("abc", "ab", Ordering::Greater);
        assert_eq_and_cmp("ab", "abc", Ordering::Less);

        assert_eq_and_cmp("abc", "aβc", Ordering::Less);
        assert_eq_and_cmp("abc", "açc", Ordering::Less);
        assert_eq_and_cmp("abc", "ábć", Ordering::Less);
        assert_eq_and_cmp("abc", "äbć", Ordering::Less);

        assert_eq_and_cmp("β", "", Ordering::Equal);
        assert_eq_and_cmp("ç", "", Ordering::Equal);
        assert_eq_and_cmp("á", "", Ordering::Equal);
        assert_eq_and_cmp("ä", "", Ordering::Equal);

        assert_eq_and_cmp("!!!", "", Ordering::Equal);
        assert_eq_and_cmp("___", "", Ordering::Equal);
        assert_eq_and_cmp("---", "", Ordering::Equal);
        assert_eq_and_cmp("   ", "", Ordering::Equal);

        assert_eq_and_cmp("123", "1_2_3", Ordering::Equal);
        assert_eq_and_cmp("123", "1-2-3", Ordering::Equal);
        assert_eq_and_cmp("123", "1 2 3", Ordering::Equal);

        assert_eq_and_cmp("abc123", "a_b_c_1_2_3", Ordering::Equal);
        assert_eq_and_cmp("abc123", "A B C 1 2 3", Ordering::Equal);

        assert_eq_and_cmp("a", "a!", Ordering::Equal);
        assert_eq_and_cmp("a", "!a", Ordering::Equal);
        assert_eq_and_cmp("a", "a_", Ordering::Equal);
        assert_eq_and_cmp("a", "_a", Ordering::Equal);

        assert_eq_and_cmp("A", "ab", Ordering::Less);
        assert_eq_and_cmp("ab", "A", Ordering::Greater);
    }

    fn assert_eq_and_cmp(a: &str, b: &str, ordering: Ordering) {
        assert_eq(a, b, ordering == Ordering::Equal);
        assert_cmp(a, b, ordering);
    }

    fn assert_eq(a: &str, b: &str, expected_eq: bool) {
        // Make slugs from both inputs
        let a_slug = Slug::new(a);
        let b_slug = Slug::new(b);

        // Borrow &str from both slugs
        let a_slug_str = a_slug.as_ref();
        let b_slug_str = b_slug.as_ref();

        // Compare slug versions using standard library
        let std_eq = a_slug_str.eq(b_slug_str);

        // Compare using custom logic
        let custom_eq_str = Slug::eq_as_slugs(a, b);
        let custom_eq_slug = Slug::eq_as_slugs(a_slug_str, b_slug_str);

        // All equalities should be the same
        assert_eq!(
            expected_eq, std_eq,
            "Failed eq check: '{}' vs '{}': expected {:?}, found {:?}",
            a_slug_str, b_slug_str, expected_eq, std_eq,
        );
        assert_eq!(
            std_eq, custom_eq_str,
            "Failed eq check: std eq: '{}' vs '{}' = {:?}; custom eq: '{}' vs '{}' = {:?}",
            a_slug_str, b_slug_str, std_eq, a, b, custom_eq_str,
        );
        assert_eq!(
            custom_eq_str, custom_eq_slug,
            "Failed eq check: custom eq (on originals): '{}' vs '{}' = {:?}; custom eq (on slugs): '{}' vs '{}' = {:?}",
            a, b, custom_eq_str, a_slug_str, b_slug_str, custom_eq_slug,
        );
    }

    fn assert_cmp(a: &str, b: &str, expected_cmp: Ordering) {
        // Make slugs from both inputs
        let a_slug = Slug::new(a);
        let b_slug = Slug::new(b);

        // Borrow &str from both slugs
        let a_slug_str = a_slug.as_ref();
        let b_slug_str = b_slug.as_ref();

        // Compare slug versions using standard library
        let std_cmp = a_slug_str.cmp(b_slug_str);

        // Compare using custom logic
        let custom_cmp_str = Slug::cmp_as_slugs(a, b);
        let custom_cmp_slug = Slug::cmp_as_slugs(a_slug_str, b_slug_str);

        // All comparisons should be the same
        assert_eq!(
            expected_cmp, std_cmp,
            "Failed cmp check: '{}' vs '{}': expected {:?}, found {:?}",
            a_slug_str, b_slug_str, expected_cmp, std_cmp,
        );
        assert_eq!(
            std_cmp, custom_cmp_str,
            "Failed cmp check: std cmp: '{}' vs '{}' = {:?}; custom cmp: '{}' vs '{}' = {:?}",
            a_slug_str, b_slug_str, std_cmp, a, b, custom_cmp_str,
        );
        assert_eq!(
            custom_cmp_str, custom_cmp_slug,
            "Failed cmp check: custom cmp (on originals): '{}' vs '{}' = {:?}; custom cmp (on slugs): '{}' vs '{}' = {:?}",
            a, b, custom_cmp_str, a_slug_str, b_slug_str, custom_cmp_slug,
        );
    }
}

#[cfg(test)]
mod group_by_value_tests {
    use super::*;
    use serde::de::{DeserializeSeed, IntoDeserializer, MapAccess};
    use serde_value::{DeserializerError, Value};
    use std::collections::BTreeMap;

    struct TestMapAccess {
        items: Vec<(String, Value)>,
        pos: usize,
    }

    impl TestMapAccess {
        fn new(items: Vec<(String, Value)>) -> Self {
            Self { items, pos: 0 }
        }
    }

    impl<'de> MapAccess<'de> for TestMapAccess {
        type Error = DeserializerError;

        fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
        where
            K: DeserializeSeed<'de>,
        {
            if self.pos < self.items.len() {
                let (ref key, _) = self.items[self.pos];
                let de = key.clone().into_deserializer();
                seed.deserialize(de).map(Some)
            } else {
                Ok(None)
            }
        }

        fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
        where
            V: DeserializeSeed<'de>,
        {
            let (_, ref value) = self.items[self.pos];
            self.pos += 1;
            seed.deserialize(value.clone())
        }
    }

    fn map_entry(key: &str, nested_key: &str, nested_value: i32) -> (String, Value) {
        (
            key.to_string(),
            Value::Map(BTreeMap::from([(
                Value::String(nested_key.to_string()),
                Value::I32(nested_value),
            )])),
        )
    }

    fn map_entry_i32(key: &str, value: i32) -> (String, Value) {
        (key.to_string(), Value::I32(value))
    }

    #[test]
    fn simple() {
        // Given
        let items = vec![
            map_entry("foo", "a", 1),
            map_entry("bar", "b", 2),
            map_entry("__BAR", "c", 3),
        ];
        let input = TestMapAccess::new(items);

        // When
        let grouped = Slug::group_map(input).unwrap();

        // Then
        assert_eq!(grouped.len(), 2);
        assert!(grouped.get("foo").is_some());
        assert!(grouped.get("bar").is_some());
    }

    #[test]
    fn collision() {
        // Given
        let items = vec![
            map_entry("A", "x", 1),
            map_entry("a___", "y", 2),
            map_entry("b", "z", 3),
        ];
        let input = TestMapAccess::new(items);

        // When
        let mut grouped = Slug::group_map(input).unwrap();

        // Then
        let (merged_slug, merged_value) = grouped.remove_entry("a").unwrap();
        assert_eq!(merged_slug.original(), "a___"); // must be the longest original
        match merged_value {
            Value::Map(map) => {
                assert_eq!(
                    map.get(&Value::String("x".to_string())),
                    Some(&Value::I32(1)),
                );
                assert_eq!(
                    map.get(&Value::String("y".to_string())),
                    Some(&Value::I32(2)),
                );
            }
            _ => panic!("Expected merged value to be a Map"),
        }
    }

    #[test]
    fn nested_non_map() {
        // Given: a map with a non-map value
        let items = vec![map_entry_i32("foo", 1)];
        let input = TestMapAccess::new(items);

        // When
        let result = Slug::group_map(input).unwrap();

        // Then
        assert_eq!(result.get("foo"), Some(&Value::I32(1)));
    }

    #[test]
    fn merge_non_map() {
        // Given
        let items = vec![map_entry("foo", "a", 1), map_entry_i32("__FOO", 2)];
        let input = TestMapAccess::new(items);

        // When
        let result = Slug::group_map(input);

        // Then
        assert!(result.is_err());
    }

    #[test]
    fn empty() {
        // Given
        let items = vec![];
        let input = TestMapAccess::new(items);

        // When
        let grouped = Slug::group_map(input).unwrap();

        // Then
        assert!(grouped.is_empty());
    }
}
