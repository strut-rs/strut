use crate::Scanner;
use config::builder::{AsyncState, DefaultState};
use config::{ConfigBuilder, Environment};

/// A small facade for assembling the opinionated version of [`ConfigBuilder`].
pub struct Assembler;

/// A simple preference collection accepted by the [`Assembler`] facade.
pub struct AssemblerChoices {
    /// The dir name/path passed to the [`Scanner::find_config_files`] method.
    pub dir_name: Option<String>,
    /// Whether to add the [`Environment`] source to the [`ConfigBuilder`].
    pub env_enabled: bool,
    /// If given, defines the prefix to pass to the [`Environment::prefix`]
    /// method.
    pub env_prefix: Option<String>,
    /// If given, defines which separator to pass to the [`Environment::prefix`]
    /// method.
    pub env_separator: Option<String>,
}

impl Default for AssemblerChoices {
    fn default() -> Self {
        Self {
            dir_name: Some("config".to_string()),
            env_enabled: true,
            env_prefix: Some("APP".to_string()),
            env_separator: Some("_".to_string()),
        }
    }
}

macro_rules! bootstrap_builder {
    ($builder:expr, $choices:expr) => {{
        let mut builder = $builder;

        // Find and add all config files as sources
        for config_file in Scanner::find_config_files($choices.dir_name.as_deref()) {
            builder = builder.add_source(config::File::from(config_file));
        }

        // Conditionally add an environment-based source
        if $choices.env_enabled {
            // Create the base source
            let mut env_source = Environment::default();

            // Set the prefix
            if let Some(prefix) = $choices.env_prefix.as_deref() {
                env_source = env_source.prefix(prefix);
            }

            // Set the separator
            if let Some(separator) = $choices.env_separator.as_deref() {
                env_source = env_source.separator(separator);
            }

            // Add to the builder
            builder = builder.add_source(env_source);
        }

        builder
    }};
}

impl Assembler {
    /// Creates and returns the opinionated [`ConfigBuilder`] in [`DefaultState`].
    ///
    /// By default, the `choices` is `config`, but a custom name may be
    /// given.
    pub fn make_sync_builder(choices: &AssemblerChoices) -> ConfigBuilder<DefaultState> {
        bootstrap_builder!(ConfigBuilder::<DefaultState>::default(), choices)
    }

    /// Creates and returns the opinionated [`ConfigBuilder`] in [`AsyncState`].
    ///
    /// By default, the `choices` is `config`, but a custom name may be
    /// given.
    pub fn make_async_builder(choices: &AssemblerChoices) -> ConfigBuilder<AsyncState> {
        bootstrap_builder!(ConfigBuilder::<AsyncState>::default(), choices)
    }
}
