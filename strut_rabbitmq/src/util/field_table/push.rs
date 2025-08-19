use crate::util::morph::Morph;
use lapin::types::{AMQPValue, FieldTable};

/// Artificial trait implemented for [`FieldTable`] to allow inserting
/// [`AMQPValue`]s infallibly instantiated from most applicable types.
pub trait Push<T> {
    /// Inserts into this [`FieldTable`] under the given `key` an [`AMQPValue`]
    /// instantiated from the given `value`.
    fn push(&mut self, key: &str, value: T);
}

/// Implements [`Push`] for every type `T` for which the underlying
/// [`AMQPValue`] implements [`MapFrom`].
impl<T> Push<T> for FieldTable
where
    AMQPValue: Morph<T>,
{
    fn push(&mut self, key: &str, value: T) {
        self.insert(key.into(), AMQPValue::morph(value));
    }
}
