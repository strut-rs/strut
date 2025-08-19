use crate::{FormatFlavor, TracingConfig, Verbosity};
use std::collections::BTreeMap;
use tracing_core::Subscriber;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::fmt::format::{
    Compact, DefaultFields, Format as EventFormatter, Format, Pretty,
};
use tracing_subscriber::fmt::Layer as FmtLayer;
use tracing_subscriber::fmt::{layer as make_fmt_layer, FormatFields};
use tracing_subscriber::layer::Filter;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

/// Creates a [formatted `Layer`](FmtLayer) based on the given
/// [config](TracingConfig).
pub fn make_layer<S>(config: impl AsRef<TracingConfig>) -> Box<dyn Layer<S> + Send + Sync>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    let config = config.as_ref();
    let targets = make_targets(config);

    match config.flavor() {
        FormatFlavor::Full => make_full_layer(config, targets),
        FormatFlavor::Compact => make_compact_layer(config, targets),
        FormatFlavor::Pretty => make_pretty_layer(config, targets),
        #[cfg(feature = "json")]
        FormatFlavor::Json => make_json_layer(config, targets),
    }
}

/// Creates the default [`Full`](tracing_subscriber::fmt::format::Full) event
/// formatting layer.
fn make_full_layer<S>(config: &TracingConfig, targets: Targets) -> Box<dyn Layer<S> + Send + Sync>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    Targets: Filter<S>,
{
    let base_layer: FmtLayer<S> = preconfigure_base_layer(make_fmt_layer(), config);

    if config.show_timestamp() {
        Box::new(base_layer.with_filter(targets))
    } else {
        Box::new(base_layer.without_time().with_filter(targets))
    }
}

/// Creates the [`Compact`](tracing_subscriber::fmt::format::Compact) event
/// formatting layer.
fn make_compact_layer<S>(
    config: &TracingConfig,
    targets: Targets,
) -> Box<dyn Layer<S> + Send + Sync>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    Targets: Filter<S>,
{
    let base_layer: FmtLayer<S, DefaultFields, Format<Compact>> =
        preconfigure_base_layer(make_fmt_layer().compact(), config);

    if config.show_timestamp() {
        Box::new(base_layer.with_filter(targets))
    } else {
        Box::new(base_layer.without_time().with_filter(targets))
    }
}

/// Creates the multi-line [`Pretty`](tracing_subscriber::fmt::format::Pretty)
/// event formatting layer.
fn make_pretty_layer<S>(config: &TracingConfig, targets: Targets) -> Box<dyn Layer<S> + Send + Sync>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    Targets: Filter<S>,
{
    let base_layer: FmtLayer<S, Pretty, Format<Pretty>> =
        preconfigure_base_layer(make_fmt_layer().pretty(), config);

    if config.show_timestamp() {
        Box::new(base_layer.with_filter(targets))
    } else {
        Box::new(base_layer.without_time().with_filter(targets))
    }
}

/// Creates the [`Json`](tracing_subscriber::fmt::format::Json) event formatting
/// layer.
#[cfg(feature = "json")]
fn make_json_layer<S>(config: &TracingConfig, targets: Targets) -> Box<dyn Layer<S> + Send + Sync>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    Targets: Filter<S>,
{
    use tracing_subscriber::fmt::format::{Json, JsonFields};

    let base_layer: FmtLayer<S, JsonFields, Format<Json>> =
        preconfigure_base_layer(make_fmt_layer().json(), config);

    if config.show_timestamp() {
        Box::new(base_layer.with_filter(targets))
    } else {
        Box::new(base_layer.without_time().with_filter(targets))
    }
}

/// Takes a generic base [formatted `Layer`](FmtLayer) and applies
/// transformations to it, as chosen in the given [`config`](TracingConfig).
fn preconfigure_base_layer<S, N, L, T, W>(
    mut layer: FmtLayer<S, N, EventFormatter<L, T>, W>,
    config: &TracingConfig,
) -> FmtLayer<S, N, EventFormatter<L, T>, W>
where
    N: for<'writer> FormatFields<'writer> + 'static,
{
    let mut no_color = false;

    if !config.color() {
        no_color = true;
    }

    #[cfg(feature = "json")]
    if config.flavor() == FormatFlavor::Json {
        no_color = true;
    }

    if no_color {
        layer = layer.with_ansi(false)
    }

    layer
        .with_target(config.show_target())
        .with_file(config.show_file())
        .with_line_number(config.show_line_number())
        .with_level(config.show_level())
        .with_thread_ids(config.show_thread_id())
        .with_thread_names(config.show_thread_name())
}

/// Creates [per-target filter](Targets) based on the choices in the given
/// [`config`](TracingConfig).
fn make_targets(config: &TracingConfig) -> Targets {
    let mut targets = Targets::new();

    targets = targets.with_default(config.verbosity());
    targets = add_custom_targets(targets, config.targets());

    targets
}

/// Composes custom targets, as configured in [`TracingConfig`].
fn add_custom_targets(targets: Targets, custom_targets: &BTreeMap<String, Verbosity>) -> Targets {
    targets.with_targets(custom_targets)
}
