mod tests {
    use pretty_assertions::assert_eq;
    use strut_core::AppReplica;

    #[test]
    fn lifetime_id() {
        // When
        let lifetime_id_a = AppReplica::lifetime_id();
        let lifetime_id_b = AppReplica::lifetime_id();

        // Then
        assert_eq!(lifetime_id_a, lifetime_id_b);
    }
}
