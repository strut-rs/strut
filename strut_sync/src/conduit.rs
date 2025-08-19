use std::future::pending;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::{mpsc, oneshot, Mutex as AsyncMutex, Notify};

/// A conduit for a simplified request-response communication between asynchronous
/// tasks that allows an **owner task** to listen for requests for a resource `T` from
/// any number of **requester tasks**, and to then asynchronously serve such requests.
///
/// The [`Conduit`] should belong to the owner task, as it allows to listen for and
/// serve the requests. The conduit can spawn any number of linked [`Retriever`]s,
/// which the requester tasks may use to asynchronously retrieve the resource `T`.
///
/// ## Error handling
///
/// The conduit-retriever pair implements a communication method between asynchronous
/// tasks, so it is only natural that communication errors may arise. Suppose two
/// tasks are communicating: one owns a conduit, and the other owns a linked retriever.
/// Requester task then initiates retrieving of the resource `T` from the owner task.
/// This process is not atomic:
///
/// - the requester task sends a request,
/// - the owner task receives the request,
/// - the owner task sends a response,
/// - the requester task receives the response.
///
/// Anywhere in this process one of the two asynchronous tasks may stop existing.
/// Furthermore, even if both tasks remain in existence, the owner task may choose
/// to not listen to incoming requests, or to not serve them (e.g., there is an
/// error producing the resource `T`).
///
/// This implementation chooses a largely error-less approach to dealing with these
/// uncertainties:
///
/// 1. The requester task may choose to either
/// [wait for the response indefinitely](Retriever::anticipate) or
/// [to deal with lack of response](Retriever::request). Another option is to wait
/// for a response [within a timeout](Retriever::request_with_timeout).
/// 2. The owner task may choose whether to [listen to requests](Conduit::requested)
/// or whether to send responses.
///
/// Any breakage of communication caused by either of the tasks exiting is consciously
/// treated as a normal part of the application lifecycle: it does not cause panics
/// and is not logged.
///
/// # Example
///
/// ```rust
/// use strut_sync::Conduit;
/// use strut_sync::Retriever;
/// use tokio::sync::oneshot;
///
/// #[tokio::main]
/// async fn main() {
///     // Create a conduit and a related retriever
///     let conduit: Conduit<String> = Conduit::new();
///     let retriever: Retriever<String> = conduit.retriever();
///
///     // Spawn owner task
///     let owner = tokio::spawn(async move {
///         // Stand-in for a valuable resource
///         let resource: String = "valuable_resource".to_string();
///
///         // Await a request
///         let sender: oneshot::Sender<String> = conduit.requested().await;
///
///         // Send a response
///         sender.send(resource).unwrap();
///     });
///
///     // Spawn requester task
///     let requester = tokio::spawn(async move {
///         // Acquire the resource
///         let acquired_resource: String = retriever.request().await.unwrap();
///
///         // Assert it is what is expected
///         assert_eq!(&acquired_resource, "valuable_resource");
///     });
///
///     requester.await.unwrap();
///     owner.await.unwrap();
/// }
/// ```
#[derive(Debug)]
pub struct Conduit<T> {
    listener: AsyncMutex<mpsc::Receiver<oneshot::Sender<T>>>,
    requester_template: mpsc::Sender<oneshot::Sender<T>>,
}

/// Allows asynchronously retrieving the resource `T` from the owner of the
/// linked [`Conduit`].
///
/// These retrievers are light-weight and may be freely cloned and passed
/// between asynchronous tasks.
#[derive(Debug, Clone)]
pub struct Retriever<T> {
    requester: mpsc::Sender<oneshot::Sender<T>>,
}

impl<T> Conduit<T> {
    /// Creates and returns a new [`Conduit`] that may [spawn](Self::retriever)
    /// any number of connected [`Retriever`]s. The retrievers may be used to
    /// request the resource `T` from the owner of the conduit.
    pub fn new() -> Self {
        // There is no point buffering requests: we wait for response immediately
        // after sending request
        let (requester_template, listener): (
            mpsc::Sender<oneshot::Sender<T>>,
            mpsc::Receiver<oneshot::Sender<T>>,
        ) = mpsc::channel(1);

        Self {
            listener: AsyncMutex::new(listener),
            requester_template,
        }
    }

    /// Spawns and returns a [`Retriever`] that is linked to this [`Conduit`] and
    /// can be used to request the resource `T` from the conduit’s owner. The returned
    /// retriever may be cloned and shared among multiple asynchronous tasks.
    pub fn retriever(&self) -> Retriever<T> {
        Retriever {
            requester: self.requester_template.clone(),
        }
    }

    /// Waits until the resource `T` is requested from any of the connected
    /// [`Retriever`]s. Upon first such request, returns the one-off sender through
    /// which the resource `T` should be sent back to whoever requested it.
    ///
    /// This method should be repeatedly awaited on by the asynchronous task that
    /// owns the resource `T`. Only one asynchronous task may listen for and
    /// serve requests at any given moment, which is the limitation that comes
    /// from the nature of the `mpsc` channels (see [`mpsc::Receiver::recv`]).
    ///
    /// ## Return type
    ///
    /// It is notable that this method returns a [`oneshot::Sender`], not an [`Option`]
    /// of it.
    ///
    /// Technically, it is possible to receive [`None`] from [`mpsc::Receiver::recv`].
    /// This happens iff there are no buffered messages in the `mpsc` channel **and**
    /// the `mpsc` channel is closed. The `mpsc` channel is closed when either all
    /// senders are dropped, or when [`mpsc::Receiver::close`] is called.
    ///
    /// This conduit owns at least one copy of [`mpsc::Sender`] (`requester_template`),
    /// so dropping all senders without also dropping this conduit is not possible.
    ///
    /// Then, this conduit does not call [`close`](mpsc::Receiver::close) and also
    /// does not expose ways to call it externally.
    ///
    /// Thus, this method takes a calculated risk of unwrapping the [`Option`] before
    /// returning.
    pub async fn requested(&self) -> oneshot::Sender<T> {
        let mut listener = self.listener.lock().await;

        listener.recv().await.expect(concat!(
            "it should not be possible for the mpsc channel of this conduit to be",
            " closed while this conduit still exists: this conduit owns both the",
            " receiver and at least one sender, and also precludes calling `close`",
            " on the receiver",
        ))
    }
}

impl<T> Retriever<T> {
    /// Requests and retrieves the resource `T` from the owner of the linked
    /// [`Conduit`]. Waits for the response potentially indefinitely. For example,
    /// if the linked conduit no longer exists, this method will never return.
    ///
    /// ## Use cases
    ///
    /// This method is useful when the requester of `T` is logically unable to proceed
    /// without it, and when it can be expected that the owner of the linked conduit
    /// has a good reason to not return a result.
    ///
    /// Examples may include the owner of the database connection struggling to
    /// establish a connection because the remote server has gone away. Another
    /// example would be the application entering the spindown phase before exiting,
    /// prompting the conduit’s owner to stop listening for requests.
    ///
    /// In such cases, this method exerts useful backpressure that prevents unwanted
    /// processing.
    pub async fn anticipate(&self) -> T {
        // Happy path: request and return
        if let Some(value) = self.request().await {
            return value;
        }

        // If we are here, there is no longer any hope of receiving the answer,
        // but the caller accepted the risk of waiting forever.

        // Wait forever (it is up to the caller to deal with this)
        pending::<()>().await;

        // We’ll never return in this case
        unreachable!()
    }

    /// Requests and retrieves the resource `T` from the owner of the linked
    /// [`Conduit`]. If any communication failure occurs (such as the linked
    /// conduit no longer exists, or the request is dropped without responding),
    /// this method returns [`None`].
    ///
    /// Note that if the linked conduit still hangs on to incoming requests
    /// without ever responding to them, this method may still wait indefinitely.
    /// [Request with a timeout](Retriever::request_with_timeout) if necessary.
    pub async fn request(&self) -> Option<T> {
        // Make a one-off channel for the owner task to send the resource `T` into
        let (oneshot_sender, oneshot_receiver) = oneshot::channel();

        // Send the request and silently give up if the linked conduit doesn’t exist anymore
        if self.requester.send(oneshot_sender).await.is_err() {
            return None;
        }

        // Return the result or nothing in case of error
        oneshot_receiver.await.ok()
    }

    /// Performs a [normal request](Retriever::request), but within the given
    /// timeout. If the request is not served in time, [`None`] is returned, and
    /// the request is dropped.
    pub async fn request_with_timeout(&self, timeout: Duration) -> Option<T> {
        // Create a notification mechanism for the timeout
        let notify_in = Arc::new(Notify::new());
        let notify_out = Arc::clone(&notify_in);

        // Start the spindown timeout
        tokio::spawn(async move {
            tokio::time::sleep(timeout).await;
            notify_in.notify_one();
        });

        // Send the request, and wait for the response or for the timeout
        select! {
            biased;
            response = self.request() => response,
            _ = notify_out.notified() => None,
        }
    }
}

impl<T> From<&Retriever<T>> for Retriever<T>
where
    T: Clone,
{
    fn from(value: &Retriever<T>) -> Self {
        value.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::FutureExt;
    use pretty_assertions::assert_eq;
    use std::panic::AssertUnwindSafe;
    use tokio::task;

    #[tokio::test]
    async fn simple_request_response() {
        // Given
        let conduit = Conduit::new();
        let retriever = conduit.retriever();

        // When
        let owner_task = task::spawn(async move {
            for i in 0..2 {
                let request = conduit.requested().await;
                request.send(format!("response_{}", i)).unwrap();
            }
        });

        // When
        let requested_value = retriever.request().await;
        let anticipated_value = retriever.anticipate().await;

        // Then
        assert_eq!(requested_value.unwrap(), "response_0");
        assert_eq!(anticipated_value, "response_1");
        assert!(owner_task.await.is_ok());
    }

    #[tokio::test]
    async fn simple_request_with_timeout() {
        // Given
        let conduit = Conduit::new();
        let retriever = conduit.retriever();

        // When
        let owner_task = task::spawn(async move {
            let request = conduit.requested().await;
            tokio::time::sleep(Duration::from_millis(20)).await;
            request.send("response").unwrap();
        });

        // When
        let requested_value = retriever
            .request_with_timeout(Duration::from_millis(10))
            .await;

        // Then
        assert_eq!(requested_value, None);
        assert!(owner_task.await.is_err()); // owner errors out because the request is dropped
    }

    #[tokio::test]
    async fn multiple_requests() {
        // Given
        let conduit = Conduit::new();
        let retriever = conduit.retriever();

        // When
        let owner_task = task::spawn(async move {
            for _ in 0..5 {
                let request = conduit.requested().await;
                request.send("response").unwrap();
            }
        });

        // Then
        let mut requester_tasks = vec![];
        for _ in 0..5 {
            let retriever = retriever.clone();
            let task = task::spawn(async move {
                let result = retriever.request().await;
                assert_eq!(result.unwrap(), "response");
            });
            requester_tasks.push(task);
        }

        // Then
        for requester_task in requester_tasks {
            assert!(requester_task.await.is_ok());
        }
        assert!(owner_task.await.is_ok());
    }

    #[tokio::test]
    async fn multiple_requests_in_order() {
        // Given
        let conduit = Conduit::new();
        let retriever = conduit.retriever();

        // When
        let owner_task = task::spawn(async move {
            for scheduled_payload in 0..5 {
                let request = conduit.requested().await;
                request.send(scheduled_payload).unwrap();
            }
        });

        // Then
        let requester_task = task::spawn(async move {
            for expected_payload in 0..5 {
                let result = retriever.request().await;
                assert_eq!(result.unwrap(), expected_payload);
            }
        });

        // Then
        assert!(requester_task.await.is_ok());
        assert!(owner_task.await.is_ok());
    }

    #[tokio::test]
    async fn retriever_request_send_error() {
        // Given
        let (requester_template, mut listener): (
            mpsc::Sender<oneshot::Sender<usize>>,
            mpsc::Receiver<oneshot::Sender<usize>>,
        ) = mpsc::channel(1);
        listener.close();

        // Given
        let retriever = Retriever {
            requester: requester_template,
        };

        // When
        let result = retriever.request().await;

        // Then
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn oneshot_sender_dropped() {
        // Given
        let conduit: Conduit<usize> = Conduit::new();
        let retriever = conduit.retriever();

        // When
        let owner_task = task::spawn(async move {
            // Intentionally drop the oneshot sender without sending a response
            let _request = conduit.requested().await;
        });
        let result = retriever.request().await;

        // Then
        assert!(result.is_none());
        assert!(owner_task.await.is_ok());
    }

    #[tokio::test]
    async fn conduit_requested_none() {
        // Given
        let (requester_template, mut listener): (
            mpsc::Sender<oneshot::Sender<usize>>,
            mpsc::Receiver<oneshot::Sender<usize>>,
        ) = mpsc::channel(1);
        listener.close();

        let conduit = Conduit {
            listener: AsyncMutex::new(listener),
            requester_template,
        };

        // When
        let result = AssertUnwindSafe(async {
            conduit.requested().await;
        });
        let result = result.catch_unwind().await;

        assert!(result.is_err());
    }
}
