use std::num::*;

/// Transforms an unsized integer into its corresponding `NonZero` type.
pub fn non_zero<T: NonZeroHelper>(v: T::Primitive) -> T {
    T::new(v).expect("hard-coded value should be non-zero")
}

/// Helper for implementing [`non_zero`].
pub trait NonZeroHelper {
    type Primitive;
    fn new(n: Self::Primitive) -> Option<Self>
    where
        Self: Sized;
}

/// Helper for implementing [`NonZeroHelper`] for multiple similar types.
macro_rules! impl_nonzero_helper {
    ($nz:ty, $prim:ty) => {
        impl NonZeroHelper for $nz {
            type Primitive = $prim;
            fn new(n: Self::Primitive) -> Option<Self> {
                <$nz>::new(n)
            }
        }
    };
}

impl_nonzero_helper!(NonZeroU8, u8);
impl_nonzero_helper!(NonZeroU16, u16);
impl_nonzero_helper!(NonZeroU32, u32);
impl_nonzero_helper!(NonZeroU64, u64);
impl_nonzero_helper!(NonZeroU128, u128);
impl_nonzero_helper!(NonZeroUsize, usize);
