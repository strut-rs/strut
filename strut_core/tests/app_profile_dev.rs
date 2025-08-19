#[cfg(test)]
mod tests {
    use strut_core::AppProfile;

    #[test]
    fn dev() {
        // When
        unsafe { std::env::set_var("APP_PROFILE", "dev") }

        // Then
        assert!(matches!(AppProfile::active(), AppProfile::Dev));
    }
}
