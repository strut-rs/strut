use crate::util::field_table::push::Push;
use crate::util::morph::Morph;
use lapin::protocol::basic::AMQPProperties;
use lapin::types::{AMQPValue, FieldTable, ShortString};

/// Artificial trait implemented for [`AMQPProperties`] to allow conveniently
/// inserting the content type, coercing it from various types.
pub trait PushContentType<T> {
    /// Inserts the content type into these [`AMQPProperties`], if it can be
    /// coerced from type `T`.
    fn push_content_type(self, value: T) -> Self;
}

/// Implements [`PushContentType`] for every type `T` for which the underlying
/// [`ShortString`] implements [`Morph`].
impl<T> PushContentType<T> for AMQPProperties
where
    ShortString: Morph<T>,
{
    fn push_content_type(self, value: T) -> Self {
        self.with_content_type(ShortString::morph(value))
    }
}

/// Artificial trait implemented for [`AMQPProperties`] to allow conveniently
/// inserting the content encoding, coercing it from various types.
pub trait PushContentEncoding<T> {
    /// Inserts the content encoding into these [`AMQPProperties`], if it can
    /// be coerced from type `T`.
    fn push_content_encoding(self, value: T) -> Self;
}

/// Implements [`PushContentEncoding`] for every type `T` for which the underlying
/// [`ShortString`] implements [`Morph`].
impl<T> PushContentEncoding<T> for AMQPProperties
where
    ShortString: Morph<T>,
{
    fn push_content_encoding(self, value: T) -> Self {
        self.with_content_encoding(ShortString::morph(value))
    }
}

/// Artificial trait implemented for [`AMQPProperties`] to allow conveniently
/// inserting the header value by key, coercing it from various types.
pub trait PushHeader<T> {
    /// Inserts the header value by key into these [`AMQPProperties`], if it
    /// can be coerced from type `T`.
    ///
    /// It is **not** recommended to use this for building up headers one
    /// key-value at a time, because this implementation clones the underlying
    /// [`FieldTable`] every time. Better to build a field table externally and
    /// then set it with a single call to
    /// [`with_headers`](AMQPProperties::with_headers).
    fn push_header(self, key: &str, value: T) -> Self;
}

/// Implements [`PushHeader`] for every type `T` for which the underlying
/// [`AMQPValue`] implements [`Morph`].
impl<T> PushHeader<T> for AMQPProperties
where
    AMQPValue: Morph<T>,
{
    fn push_header(self, key: &str, value: T) -> Self {
        // We can only clone existing headers to add more
        let mut new_headers = if let Some(headers) = self.headers() {
            headers.clone()
        } else {
            FieldTable::default()
        };

        // Push new header
        new_headers.push(key, value);

        self.with_headers(new_headers)
    }
}

/// Artificial trait implemented for [`AMQPProperties`] to allow conveniently
/// inserting the correlation ID, coercing it from various types.
pub trait PushCorrelationId<T> {
    /// Inserts the correlation ID into these [`AMQPProperties`], if it can be
    /// coerced from type `T`.
    fn push_correlation_id(self, value: T) -> Self;
}

/// Implements [`PushCorrelationId`] for every type `T` for which the underlying
/// [`AMQPValue`] implements [`Morph`].
impl<T> PushCorrelationId<T> for AMQPProperties
where
    ShortString: Morph<T>,
{
    fn push_correlation_id(self, value: T) -> Self {
        self.with_correlation_id(ShortString::morph(value))
    }
}

/// Artificial trait implemented for [`AMQPProperties`] to allow conveniently
/// inserting the “reply-to” value, coercing it from various types.
pub trait PushReplyTo<T> {
    /// Inserts the “reply-to” value into these [`AMQPProperties`], if it can
    /// be coerced from type `T`.
    fn push_reply_to(self, value: T) -> Self;
}

/// Implements [`PushReplyTo`] for every type `T` for which the underlying
/// [`AMQPValue`] implements [`Morph`].
impl<T> PushReplyTo<T> for AMQPProperties
where
    ShortString: Morph<T>,
{
    fn push_reply_to(self, value: T) -> Self {
        self.with_reply_to(ShortString::morph(value))
    }
}

/// Artificial trait implemented for [`AMQPProperties`] to allow conveniently
/// inserting the expiration value, coercing it from various types.
pub trait PushExpiration<T> {
    /// Inserts the expiration value into these [`AMQPProperties`], if it can
    /// be coerced from type `T`.
    fn push_expiration(self, value: T) -> Self;
}

/// Implements [`PushExpiration`] for every type `T` for which the underlying
/// [`AMQPValue`] implements [`Morph`].
impl<T> PushExpiration<T> for AMQPProperties
where
    ShortString: Morph<T>,
{
    fn push_expiration(self, value: T) -> Self {
        self.with_expiration(ShortString::morph(value))
    }
}

/// Artificial trait implemented for [`AMQPProperties`] to allow conveniently
/// inserting the message ID, coercing it from various types.
pub trait PushMessageId<T> {
    /// Inserts the message ID into these [`AMQPProperties`], if it can be
    /// coerced from type `T`.
    fn push_message_id(self, value: T) -> Self;
}

/// Implements [`PushMessageId`] for every type `T` for which the underlying
/// [`ShortString`] implements [`Morph`].
impl<T> PushMessageId<T> for AMQPProperties
where
    ShortString: Morph<T>,
{
    fn push_message_id(self, value: T) -> Self {
        self.with_message_id(ShortString::morph(value))
    }
}

/// Artificial trait implemented for [`AMQPProperties`] to allow conveniently
/// inserting the message kind, coercing it from various types.
pub trait PushKind<T> {
    /// Inserts the message kind into these [`AMQPProperties`], if it can be
    /// coerced from type `T`.
    fn push_kind(self, value: T) -> Self;
}

/// Implements [`PushKind`] for every type `T` for which the underlying
/// [`ShortString`] implements [`Morph`].
impl<T> PushKind<T> for AMQPProperties
where
    ShortString: Morph<T>,
{
    fn push_kind(self, value: T) -> Self {
        self.with_type(ShortString::morph(value))
    }
}

/// Artificial trait implemented for [`AMQPProperties`] to allow conveniently
/// inserting the user ID, coercing it from various types.
pub trait PushUserId<T> {
    /// Inserts the user ID into these [`AMQPProperties`], if it is can be
    /// coerced from type `T`.
    fn push_user_id(self, value: T) -> Self;
}

/// Implements [`PushUserId`] for every type `T` for which the underlying
/// [`ShortString`] implements [`Morph`].
impl<T> PushUserId<T> for AMQPProperties
where
    ShortString: Morph<T>,
{
    fn push_user_id(self, value: T) -> Self {
        self.with_user_id(ShortString::morph(value))
    }
}

/// Artificial trait implemented for [`AMQPProperties`] to allow conveniently
/// inserting the app ID, coercing it from various types.
pub trait PushAppId<T> {
    /// Inserts the app ID into these [`AMQPProperties`], if it can be coerced
    /// from type `T`.
    fn push_app_id(self, value: T) -> Self;
}

/// Implements [`PushAppId`] for every type `T` for which the underlying
/// [`ShortString`] implements [`Morph`].
impl<T> PushAppId<T> for AMQPProperties
where
    ShortString: Morph<T>,
{
    fn push_app_id(self, value: T) -> Self {
        self.with_app_id(ShortString::morph(value))
    }
}

/// Artificial trait implemented for [`AMQPProperties`] to allow conveniently
/// inserting the cluster ID, coercing it from various types.
pub trait PushClusterId<T> {
    /// Inserts the cluster ID into these [`AMQPProperties`], if it can be
    /// coerced from type `T`.
    fn push_cluster_id(self, value: T) -> Self;
}

/// Implements [`PushClusterId`] for every type `T` for which the underlying
/// [`ShortString`] implements [`Morph`].
impl<T> PushClusterId<T> for AMQPProperties
where
    ShortString: Morph<T>,
{
    fn push_cluster_id(self, value: T) -> Self {
        self.with_cluster_id(ShortString::morph(value))
    }
}
