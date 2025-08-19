use sentry_tracing::{EventMapping, SentryLayer};
use std::error::Error;
use std::fmt::Debug;
use std::str::FromStr;
use strut_core::ALERT_FIELD_NAME;
use tracing::field::Field;
use tracing::{Event, Subscriber};
use tracing_subscriber::field::Visit;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;

/// Creates a [`SentryLayer`] with a custom event mapper. The goal is to
/// automatically generate a Sentry event for every call to a
/// [`tracing`](tracing) macro (e.g., [`info!`](tracing::info)) with a specially
/// named field [`ALERT_FIELD_NAME`].
///
/// This layer should be included in the global default [`Subscriber`]. Then,
/// any tracing events marked with a field named [`ALERT_FIELD_NAME`] are sent
/// to Sentry (if it is connected and configured) in addition to being processed
/// by other layers of the subscriber.
///
/// Sentry has two kinds of events: [normal](sentry_tracing::event_from_event),
/// and [breadcrumb](sentry_tracing::breadcrumb_from_event).
///
/// When the field named [`ALERT_FIELD_NAME`] contains a string value, the
/// string is further parsed:
///
/// - `"breadcrumb"` or `"crumb"` — generates a breadcrumb-style Sentry event,
/// - any other string literal (as well as any other value type) generates a
/// “normal” Sentry event.
///
/// Note that the level of the tracing call (e.g., [`info!`](tracing::info) or
/// [`error!`](tracing::error)) does not affect the kind of event generated. As
/// of the latest version of [`sentry_tracing`] the stack trace of any attached
/// [`Error`] is attached to the Sentry event as well, so there is no special
/// “exception-style” events.
///
/// ## Examples
///
/// ```
/// // Include an `alert` field in the logged event to also generate an event
/// // for Sentry
/// tracing::error!(
///     alert = true, // boolean value will generate a “normal” Sentry event
///     another_field = 42,
///     "Sample message",
/// );
///
/// // `alert = "breadcrumb"` will generate a breadcrumb-style Sentry event
/// tracing::trace!(alert = "breadcrumb", "Sample message");
///
/// // Events without an `alert` field do not generate Sentry events
/// tracing::info!("Sample message");
/// ```
pub fn make_layer<S>() -> SentryLayer<S>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    sentry_tracing::layer().event_mapper(field_walker)
}

/// Takes a tracing event and visits its fields to make a decision about whether to send the event
/// to Sentry. Returns the appropriate [`EventMapping`].
fn field_walker<S>(event: &Event, ctx: Context<'_, S>) -> EventMapping
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    // Make a visitor and send it inspecting the event’s fields
    let mut sentry_behavior: SentryBehaviorVisitor = SentryBehaviorVisitor { event: None };
    event.record(&mut sentry_behavior); // this will visit every field in the event with the given visitor

    // Check the visitor
    match sentry_behavior.event {
        // Nothing to send to Sentry
        None => EventMapping::Ignore,

        // Generate a normal event
        Some(SentryEventKind::Event) => {
            EventMapping::Event(sentry_tracing::event_from_event(event, &ctx))
        }

        // Generate a breadcrumb-style event
        Some(SentryEventKind::Breadcrumb) => {
            EventMapping::Breadcrumb(sentry_tracing::breadcrumb_from_event(event, &ctx))
        }
    }
}

/// Helper structure for visiting every field on a tracing event. Stores the
/// result of the visit.
struct SentryBehaviorVisitor {
    event: Option<SentryEventKind>,
}

impl Visit for SentryBehaviorVisitor {
    fn record_f64(&mut self, field: &Field, _value: f64) {
        self.match_field(field);
    }

    fn record_i64(&mut self, field: &Field, _value: i64) {
        self.match_field(field);
    }

    fn record_u64(&mut self, field: &Field, _value: u64) {
        self.match_field(field);
    }

    fn record_i128(&mut self, field: &Field, _value: i128) {
        self.match_field(field);
    }

    fn record_u128(&mut self, field: &Field, _value: u128) {
        self.match_field(field);
    }

    fn record_bool(&mut self, field: &Field, _value: bool) {
        self.match_field(field);
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == ALERT_FIELD_NAME {
            self.event = SentryEventKind::from_str(value).ok();
        }
    }

    fn record_bytes(&mut self, field: &Field, _value: &[u8]) {
        self.match_field(field);
    }

    fn record_error(&mut self, field: &Field, _value: &(dyn Error + 'static)) {
        self.match_field(field);
    }

    /// Has to be implemented, but not needed in scope of this visitor.
    fn record_debug(&mut self, field: &Field, _value: &dyn Debug) {
        self.match_field(field);
    }
}

impl SentryBehaviorVisitor {
    #[inline(always)]
    fn match_field(&mut self, field: &Field) {
        if field.name() == ALERT_FIELD_NAME {
            self.event = Some(SentryEventKind::Event);
        }
    }
}

/// Represents the supported kinds of Sentry events.
enum SentryEventKind {
    Event,
    Breadcrumb,
}

impl FromStr for SentryEventKind {
    type Err = ();

    /// Parses the recognized string literals assigned to the
    /// [`ALERT_FIELD_NAME`] field.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "breadcrumb" | "crumb" => Ok(SentryEventKind::Breadcrumb),
            _ => Ok(SentryEventKind::Event),
        }
    }
}
