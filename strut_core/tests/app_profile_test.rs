#[cfg(test)]
mod tests {
    use strut_core::AppProfile;

    #[test]
    fn test() {
        // When
        unsafe { std::env::set_var("APP_PROFILE", "test") }

        // Then
        assert!(matches!(AppProfile::active(), AppProfile::Test));
    }
}
