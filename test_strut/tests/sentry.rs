#[cfg(test)]
mod tests {
    use test_util::Harness;

    #[test]
    fn config_short_toml() {
        Harness::pass("cases/sentry/config_short_toml");
    }

    #[test]
    fn config_short_yaml() {
        Harness::pass("cases/sentry/config_short_yaml");
    }

    #[test]
    fn config_full_toml() {
        Harness::dump_output("cases/sentry/config_full_toml");
    }

    #[test]
    fn config_full_yaml() {
        Harness::dump_output("cases/sentry/config_full_yaml");
    }
}
