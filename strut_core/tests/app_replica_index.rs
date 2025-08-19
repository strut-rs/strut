#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use strut_core::AppReplica;

    /// Relies on `APP_REPLICA_INDEX=42` being set in the crate’s `build.rs`
    /// file for tests, but then overwrites it with an unsafe call to `set_var`.
    #[test]
    fn index() {
        // When
        unsafe {
            std::env::set_var("APP_REPLICA_INDEX", "777");
        }

        // Then
        assert_eq!(AppReplica::index(), Some(777));
    }
}
