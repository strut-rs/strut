use crate::util::Coerce;
use lapin::types::{AMQPValue, FieldTable};

/// Artificial trait implemented for [`FieldTable`] to allow optional retrieval
/// of inferred, typed values from the underlying map.
pub trait Retrieve<'a, T> {
    /// Optionally retrieves a value of type `T` inferred from the value stored
    /// under the given `key`, provided the key is set **and** a value of type
    /// `T` can be inferred from its value.
    fn retrieve(&'a self, key: &str) -> Option<T>;
}

/// Implements [`Retrieve`] for every type `T` for which the underlying
/// [`AMQPValue`] implements [`Coerce`].
impl<'a, T> Retrieve<'a, T> for FieldTable
where
    AMQPValue: Coerce<'a, T>,
{
    fn retrieve(&'a self, key: &str) -> Option<T> {
        self.inner().get(key).and_then(AMQPValue::coerce)
    }
}
