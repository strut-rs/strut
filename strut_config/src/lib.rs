#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![cfg_attr(test, deny(warnings))]

/// Implements a [`Scanner`] for config files in filesystem.
mod scanner;
pub use self::scanner::dir::ConfigDir;
pub use self::scanner::entry::{ConfigEntry, ConfigEntryIter};
pub use self::scanner::file::ConfigFile;
pub use self::scanner::Scanner;

/// Implements an [`Assembler`] for the opinionated [`ConfigBuilder`](config::ConfigBuilder).
mod assembler;
pub use self::assembler::{Assembler, AssemblerChoices};
