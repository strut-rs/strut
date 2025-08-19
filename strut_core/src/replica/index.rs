/// Reads the environment to discern whether the current application runtime has
/// the replica index defined for it. If the environment does not set the replica
/// index to a valid unsigned integer, defaults to `None`.
pub fn discern() -> Option<usize> {
    std::env::var("APP_REPLICA_INDEX")
        .ok()
        .and_then(|index| index.parse().ok())
}
