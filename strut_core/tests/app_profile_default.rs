#[cfg(test)]
mod tests {
    use strut_core::AppProfile;

    #[test]
    fn default() {
        // Then
        assert!(matches!(AppProfile::active(), AppProfile::Dev));

        // When
        unsafe { std::env::set_var("APP_PROFILE", "prod") }

        // Then
        assert!(matches!(AppProfile::active(), AppProfile::Dev));
    }
}
