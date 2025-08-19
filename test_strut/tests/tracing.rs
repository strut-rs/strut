#[cfg(test)]
mod tests {
    use assertables::assert_contains;
    use test_util::Harness;

    #[test]
    fn config_default_toml() {
        Harness::pass("cases/tracing/config_default_toml");
    }

    #[test]
    fn config_default_yaml() {
        Harness::pass("cases/tracing/config_default_yaml");
    }

    #[test]
    fn usage() {
        let output = Harness::pass("cases/tracing/usage");
        let stdout = str::from_utf8(output.stdout.as_slice()).unwrap();

        assert_contains!(stdout, "Running application logic");
    }

    #[test]
    fn sentry() {
        let output = Harness::pass("cases/tracing/sentry");
        let stdout = str::from_utf8(output.stdout.as_slice()).unwrap();

        assert_contains!(stdout, "This will be also sent to Sentry");
    }
}
