use crate::Envelope;
use async_trait::async_trait;
use futures::future::join_all;
use nonempty::NonEmpty;

/// Represents a collection of [`Envelope`]s that can be acted upon with included
/// convenience methods.
#[async_trait]
pub trait EnvelopeStack {
    /// Efficiently calls [`complete`](Envelope::complete) on every envelope in
    /// this stack.
    async fn complete_all(self);

    /// Efficiently calls [`backwash`](Envelope::backwash) on every envelope in
    /// this stack.
    async fn backwash_all(self);

    /// Efficiently calls [`abandon`](Envelope::abandon) on every envelope in
    /// this stack.
    async fn abandon_all(self);
}

/// Implements [`EnvelopeStack`] for a vector of [`Envelope`]s.
#[async_trait]
impl<T> EnvelopeStack for Vec<Envelope<T>>
where
    T: Send,
{
    async fn complete_all(self) {
        join_all(self.into_iter().map(|envelope| envelope.complete())).await;
    }

    async fn backwash_all(self) {
        join_all(self.into_iter().map(|envelope| envelope.backwash())).await;
    }

    async fn abandon_all(self) {
        join_all(self.into_iter().map(|envelope| envelope.abandon())).await;
    }
}

/// Implements [`EnvelopeStack`] for a [`NonEmpty`] collection of [`Envelope`]s.
#[async_trait]
impl<T> EnvelopeStack for NonEmpty<Envelope<T>>
where
    T: Send,
{
    async fn complete_all(self) {
        join_all(self.into_iter().map(|envelope| envelope.complete())).await;
    }

    async fn backwash_all(self) {
        join_all(self.into_iter().map(|envelope| envelope.backwash())).await;
    }

    async fn abandon_all(self) {
        join_all(self.into_iter().map(|envelope| envelope.abandon())).await;
    }
}
