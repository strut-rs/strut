use tokio_util::sync::CancellationToken;

/// A token issued for every workload registered with
/// [`AppSpindownRegistry`](crate::spindown::registry::SpindownRegistry).
///
/// This token allows the workload to [indicate](AppSpindownToken::punch_out)
/// that it has gracefully completed and cleaned up its resources.
pub struct AppSpindownToken {
    token: CancellationToken,
}

impl AppSpindownToken {
    /// Internal constructor.
    pub(crate) fn new(token: CancellationToken) -> Self {
        Self { token }
    }

    /// Indicate that the workload associated with this [`AppSpindownToken`] has completed its
    /// spindown procedure, whatever it might be.
    ///
    /// This method is automatically called from the token’s [`Drop`] implementation, so allowing
    /// the token to go out of scope is an alternative way to conveniently punch out.
    pub fn punch_out(&self) {
        self.token.cancel();
    }
}

impl Drop for AppSpindownToken {
    /// Convenience implementation of [`Drop`] that allows the owners of the [`AppSpindownToken`]
    /// to punch out by simply allowing the token to go out of scope.
    fn drop(&mut self) {
        self.punch_out();
    }
}
