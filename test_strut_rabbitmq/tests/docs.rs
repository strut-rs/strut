#[cfg(test)]
mod tests {
    use test_util::Harness;

    #[test]
    fn usage_toml() {
        Harness::pass("cases/docs/usage_toml");
    }

    #[test]
    fn usage_yaml() {
        Harness::pass("cases/docs/usage_yaml");
    }

    #[test]
    fn config_toml() {
        Harness::pass("cases/docs/config_toml");
    }

    #[test]
    fn config_yaml() {
        Harness::pass("cases/docs/config_yaml");
    }

    #[test]
    fn egress_short_toml() {
        Harness::pass("cases/docs/egress_short_toml");
    }

    #[test]
    fn egress_short_yaml() {
        Harness::pass("cases/docs/egress_short_yaml");
    }

    #[test]
    fn egress_full_toml() {
        Harness::pass("cases/docs/egress_full_toml");
    }

    #[test]
    fn egress_full_yaml() {
        Harness::pass("cases/docs/egress_full_yaml");
    }

    #[test]
    fn ingress_short_toml() {
        Harness::pass("cases/docs/ingress_short_toml");
    }

    #[test]
    fn ingress_short_yaml() {
        Harness::pass("cases/docs/ingress_short_yaml");
    }

    #[test]
    fn ingress_full_toml() {
        Harness::pass("cases/docs/ingress_full_toml");
    }

    #[test]
    fn ingress_full_yaml() {
        Harness::pass("cases/docs/ingress_full_yaml");
    }
}
