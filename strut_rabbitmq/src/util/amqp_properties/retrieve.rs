use crate::util::field_table::retrieve::Retrieve;
use crate::util::Coerce;
use lapin::protocol::basic::AMQPProperties;
use lapin::types::{AMQPValue, ShortString};

/// Artificial trait implemented for [`AMQPProperties`] to allow conveniently
/// extracting the content type, coercing it into various types.
pub trait RetrieveContentType<'a, T> {
    /// Extracts the content type from these [`AMQPProperties`], if it is present
    /// and can be coerced to type `T`.
    fn retrieve_content_type(&'a self) -> Option<T>;
}

/// Implements [`RetrieveContentType`] for every type `T` for which the underlying
/// [`ShortString`] implements [`Coerce`].
impl<'a, T> RetrieveContentType<'a, T> for AMQPProperties
where
    ShortString: Coerce<'a, T>,
{
    fn retrieve_content_type(&'a self) -> Option<T> {
        self.content_type()
            .as_ref()
            .map(|short_string| short_string.coerce())
            .flatten()
    }
}

/// Artificial trait implemented for [`AMQPProperties`] to allow conveniently
/// extracting the content encoding, coercing it into various types.
pub trait RetrieveContentEncoding<'a, T> {
    /// Extracts the content encoding from these [`AMQPProperties`], if it is
    /// present and can be coerced to type `T`.
    fn retrieve_content_encoding(&'a self) -> Option<T>;
}

/// Implements [`RetrieveContentEncoding`] for every type `T` for which the underlying
/// [`ShortString`] implements [`Coerce`].
impl<'a, T> RetrieveContentEncoding<'a, T> for AMQPProperties
where
    ShortString: Coerce<'a, T>,
{
    fn retrieve_content_encoding(&'a self) -> Option<T> {
        self.content_encoding()
            .as_ref()
            .map(|short_string| short_string.coerce())
            .flatten()
    }
}

/// Artificial trait implemented for [`AMQPProperties`] to allow conveniently
/// extracting the header value by key, coercing it into various types.
pub trait RetrieveHeader<'a, T> {
    /// Extracts the header value by key from these [`AMQPProperties`], if it is
    /// present and can be coerced to type `T`.
    fn retrieve_header(&'a self, key: &str) -> Option<T>;
}

/// Implements [`RetrieveHeader`] for every type `T` for which the underlying
/// [`AMQPValue`] implements [`Coerce`].
impl<'a, T> RetrieveHeader<'a, T> for AMQPProperties
where
    AMQPValue: Coerce<'a, T>,
{
    fn retrieve_header(&'a self, key: &str) -> Option<T> {
        self.headers()
            .as_ref()
            .map(|field_table| field_table.retrieve(key))
            .flatten()
    }
}

/// Artificial trait implemented for [`AMQPProperties`] to allow conveniently
/// extracting the correlation ID, coercing it into various types.
pub trait RetrieveCorrelationId<'a, T> {
    /// Extracts the correlation ID from these [`AMQPProperties`], if it is
    /// present and can be coerced to type `T`.
    fn retrieve_correlation_id(&'a self) -> Option<T>;
}

/// Implements [`RetrieveCorrelationId`] for every type `T` for which the underlying
/// [`AMQPValue`] implements [`Coerce`].
impl<'a, T> RetrieveCorrelationId<'a, T> for AMQPProperties
where
    ShortString: Coerce<'a, T>,
{
    fn retrieve_correlation_id(&'a self) -> Option<T> {
        self.correlation_id()
            .as_ref()
            .map(|short_string| short_string.coerce())
            .flatten()
    }
}

/// Artificial trait implemented for [`AMQPProperties`] to allow conveniently
/// extracting the “reply-to” value, coercing it into various types.
pub trait RetrieveReplyTo<'a, T> {
    /// Extracts the “reply-to” value from these [`AMQPProperties`], if it is
    /// present and can be coerced to type `T`.
    fn retrieve_reply_to(&'a self) -> Option<T>;
}

/// Implements [`RetrieveReplyTo`] for every type `T` for which the underlying
/// [`AMQPValue`] implements [`Coerce`].
impl<'a, T> RetrieveReplyTo<'a, T> for AMQPProperties
where
    ShortString: Coerce<'a, T>,
{
    fn retrieve_reply_to(&'a self) -> Option<T> {
        self.reply_to()
            .as_ref()
            .map(|short_string| short_string.coerce())
            .flatten()
    }
}

/// Artificial trait implemented for [`AMQPProperties`] to allow conveniently
/// extracting the expiration value, coercing it into various types.
pub trait RetrieveExpiration<'a, T> {
    /// Extracts the expiration value from these [`AMQPProperties`], if it is
    /// present and can be coerced to type `T`.
    fn retrieve_expiration(&'a self) -> Option<T>;
}

/// Implements [`RetrieveExpiration`] for every type `T` for which the underlying
/// [`AMQPValue`] implements [`Coerce`].
impl<'a, T> RetrieveExpiration<'a, T> for AMQPProperties
where
    ShortString: Coerce<'a, T>,
{
    fn retrieve_expiration(&'a self) -> Option<T> {
        self.expiration()
            .as_ref()
            .map(|short_string| short_string.coerce())
            .flatten()
    }
}

/// Artificial trait implemented for [`AMQPProperties`] to allow conveniently
/// extracting the message ID, coercing it into various types.
pub trait RetrieveMessageId<'a, T> {
    /// Extracts the message ID from these [`AMQPProperties`], if it is present
    /// and can be coerced to type `T`.
    fn retrieve_message_id(&'a self) -> Option<T>;
}

/// Implements [`RetrieveMessageId`] for every type `T` for which the underlying
/// [`ShortString`] implements [`Coerce`].
impl<'a, T> RetrieveMessageId<'a, T> for AMQPProperties
where
    ShortString: Coerce<'a, T>,
{
    fn retrieve_message_id(&'a self) -> Option<T> {
        self.message_id()
            .as_ref()
            .map(|short_string| short_string.coerce())
            .flatten()
    }
}

/// Artificial trait implemented for [`AMQPProperties`] to allow conveniently
/// extracting the message kind, coercing it into various types.
pub trait RetrieveKind<'a, T> {
    /// Extracts the message kind from these [`AMQPProperties`], if it is present
    /// and can be coerced to type `T`.
    fn retrieve_kind(&'a self) -> Option<T>;
}

/// Implements [`RetrieveKind`] for every type `T` for which the underlying
/// [`ShortString`] implements [`Coerce`].
impl<'a, T> RetrieveKind<'a, T> for AMQPProperties
where
    ShortString: Coerce<'a, T>,
{
    fn retrieve_kind(&'a self) -> Option<T> {
        self.kind()
            .as_ref()
            .map(|short_string| short_string.coerce())
            .flatten()
    }
}

/// Artificial trait implemented for [`AMQPProperties`] to allow conveniently
/// extracting the user ID, coercing it into various types.
pub trait RetrieveUserId<'a, T> {
    /// Extracts the user ID from these [`AMQPProperties`], if it is present
    /// and can be coerced to type `T`.
    fn retrieve_user_id(&'a self) -> Option<T>;
}

/// Implements [`RetrieveUserId`] for every type `T` for which the underlying
/// [`ShortString`] implements [`Coerce`].
impl<'a, T> RetrieveUserId<'a, T> for AMQPProperties
where
    ShortString: Coerce<'a, T>,
{
    fn retrieve_user_id(&'a self) -> Option<T> {
        self.user_id()
            .as_ref()
            .map(|short_string| short_string.coerce())
            .flatten()
    }
}

/// Artificial trait implemented for [`AMQPProperties`] to allow conveniently
/// extracting the app ID, coercing it into various types.
pub trait RetrieveAppId<'a, T> {
    /// Extracts the app ID from these [`AMQPProperties`], if it is present
    /// and can be coerced to type `T`.
    fn retrieve_app_id(&'a self) -> Option<T>;
}

/// Implements [`RetrieveAppId`] for every type `T` for which the underlying
/// [`ShortString`] implements [`Coerce`].
impl<'a, T> RetrieveAppId<'a, T> for AMQPProperties
where
    ShortString: Coerce<'a, T>,
{
    fn retrieve_app_id(&'a self) -> Option<T> {
        self.app_id()
            .as_ref()
            .map(|short_string| short_string.coerce())
            .flatten()
    }
}

/// Artificial trait implemented for [`AMQPProperties`] to allow conveniently
/// extracting the cluster ID, coercing it into various types.
pub trait RetrieveClusterId<'a, T> {
    /// Extracts the cluster ID from these [`AMQPProperties`], if it is present
    /// and can be coerced to type `T`.
    fn retrieve_cluster_id(&'a self) -> Option<T>;
}

/// Implements [`RetrieveClusterId`] for every type `T` for which the underlying
/// [`ShortString`] implements [`Coerce`].
impl<'a, T> RetrieveClusterId<'a, T> for AMQPProperties
where
    ShortString: Coerce<'a, T>,
{
    fn retrieve_cluster_id(&'a self) -> Option<T> {
        self.cluster_id()
            .as_ref()
            .map(|short_string| short_string.coerce())
            .flatten()
    }
}
