#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use strut_core::AppProfile;

    #[test]
    fn custom() {
        // When
        unsafe { std::env::set_var("APP_PROFILE", "CUSTOM_PROFILE") }

        // Then
        assert_eq!(AppProfile::active().as_str(), "customp");
    }
}
