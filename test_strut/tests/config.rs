#[cfg(test)]
mod tests {
    use assertables::assert_contains;
    use test_util::Harness;

    #[test]
    fn case_01_toml() {
        Harness::pass("cases/config/01_toml");
    }

    #[test]
    fn case_01_yml() {
        Harness::pass("cases/config/01_yml");
    }

    #[test]
    fn case_02_toml() {
        let output = Harness::pass_with_env(
            "cases/config/02_toml",
            &[("APP_DATABASE_MYSQL_MYDATABASE_PASSWORD", "password_a")],
        );
        let stdout = str::from_utf8(output.stdout.as_slice()).unwrap();

        assert_contains!(stdout, "password_a");
        assert_contains!(stdout, "password_b");
    }

    #[test]
    fn case_02_yml() {
        let output = Harness::pass_with_env(
            "cases/config/02_yml",
            &[("APP_DATABASE_MYSQL_MYDATABASE_PASSWORD", "password_a")],
        );
        let stdout = str::from_utf8(output.stdout.as_slice()).unwrap();

        assert_contains!(stdout, "password_a");
        assert_contains!(stdout, "password_b");
    }
}
