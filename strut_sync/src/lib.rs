#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![cfg_attr(test, deny(warnings))]

mod conduit;
pub use self::conduit::{Conduit, Retriever};

mod latch;
pub use self::latch::{Gate, Latch};
