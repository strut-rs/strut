#[cfg(test)]
mod tests {
    use test_util::Harness;

    #[test]
    fn dev_default() {
        Harness::pass("cases/profile/dev_default");
    }

    #[test]
    fn dev() {
        Harness::pass_with_env("cases/profile/dev", &[("APP_PROFILE", "dev")]);
    }

    #[test]
    fn prod() {
        Harness::pass_with_env("cases/profile/prod", &[("APP_PROFILE", "PROD")]);
    }

    #[test]
    fn test() {
        Harness::pass_with_env("cases/profile/test", &[("APP_PROFILE", "Test_")]);
    }

    #[test]
    fn other13() {
        Harness::pass_with_env(
            "cases/profile/other13",
            &[("APP_PROFILE", "++++++++++++++other_135")],
        );
    }

    #[test]
    fn some_env() {
        Harness::pass("cases/profile/some_env");
    }
}
