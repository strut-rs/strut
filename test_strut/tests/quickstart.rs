#[cfg(test)]
mod tests {
    use assertables::{assert_contains, assert_is_match};
    use regex::Regex;
    use std::process::Output;
    use test_util::Harness;

    #[test]
    fn case_01() {
        Harness::pass("cases/quickstart/01");
    }

    #[test]
    fn case_02_toml() {
        Harness::pass("cases/quickstart/02_toml");
    }

    #[test]
    fn case_02_yaml() {
        Harness::pass("cases/quickstart/02_yaml");
    }

    #[test]
    fn case_03() {
        Harness::pass_with_env("cases/quickstart/03", &[("APP_NAME", "app-custom")]);
    }

    #[test]
    fn case_04() {
        let output = Harness::pass("cases/quickstart/04");

        assert_human_output(output);
    }

    #[test]
    fn case_05_toml() {
        let output = Harness::pass("cases/quickstart/05_toml");

        assert_json_output(output);
    }

    #[test]
    fn case_05_yaml() {
        let output = Harness::pass("cases/quickstart/05_yaml");

        assert_json_output(output);
    }

    const TIMESTAMP_REGEX: &str = r#"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{6}Z"#;
    const LINE_1_REGEX: &str = r#"Starting app-backend with profile 'dev' \(default replica, lifetime ID '[a-z]{3}-[a-z]{4}-[a-z]{3}'\)"#;

    fn assert_human_output(output: Output) {
        // Given
        let output = sanitize(&output.stdout);
        let mut lines = output.lines();

        // Then
        let timestamp_regex_raw = format!(r#"^{}.*"#, TIMESTAMP_REGEX);
        let timestamp_regex = Regex::new(&timestamp_regex_raw).unwrap();

        let line_1 = lines.next().unwrap();
        let raw_regex = format!(r#".*{}.*"#, LINE_1_REGEX);
        let regex = Regex::new(&raw_regex).unwrap();
        assert_is_match!(timestamp_regex, line_1);
        assert_contains!(line_1, "INFO");
        assert_contains!(line_1, "strut::launchpad::wiring::preflight:");
        assert_is_match!(regex, line_1);

        let line_2 = lines.next().unwrap();
        assert_is_match!(timestamp_regex, line_2);
        assert_contains!(line_2, "INFO");
        assert_contains!(line_2, "demo_app: Running app-backend...");

        let line_3 = lines.next().unwrap();
        assert_is_match!(timestamp_regex, line_3);
        assert_contains!(line_3, "INFO");
        assert_contains!(
            line_3,
            "strut_core::context: Terminating application context",
        );

        let line_4 = lines.next().unwrap();
        assert_is_match!(timestamp_regex, line_4);
        assert_contains!(line_4, "INFO");
        assert_contains!(line_4, "strut_core::spindown::registry: Spindown initiated");

        let line_5 = lines.next().unwrap();
        assert_is_match!(timestamp_regex, line_5);
        assert_contains!(line_5, "INFO");
        assert_contains!(line_5, "strut_core::spindown::registry: Spindown completed");
    }

    fn assert_json_output(output: Output) {
        // Given
        let output = sanitize(&output.stdout);
        let mut lines = output.lines();

        // Then
        let timestamp_regex_raw = format!(r#".*"timestamp":"{}".*"#, TIMESTAMP_REGEX);
        let timestamp_regex = Regex::new(&timestamp_regex_raw).unwrap();

        let line_1 = lines.next().unwrap();
        let raw_regex = format!(r#".*"fields":\{{"message":"{}"\}}.*"#, LINE_1_REGEX);
        let regex = Regex::new(&raw_regex).unwrap();
        assert_is_match!(timestamp_regex, line_1);
        assert_contains!(line_1, r#""level":"INFO""#);
        assert_contains!(line_1, r#""target":"strut::launchpad::wiring::preflight""#);
        assert_is_match!(regex, line_1);

        let line_2 = lines.next().unwrap();
        assert_is_match!(timestamp_regex, line_2);
        assert_contains!(line_2, r#""level":"INFO""#);
        assert_contains!(line_2, r#""target":"demo_app""#);
        assert_contains!(line_2, r#""fields":{"message":"Running app-backend..."}"#);

        let line_3 = lines.next().unwrap();
        assert_is_match!(timestamp_regex, line_3);
        assert_contains!(line_3, r#""level":"INFO""#);
        assert_contains!(line_3, r#""target":"strut_core::context""#);
        assert_contains!(
            line_3,
            r#""fields":{"message":"Terminating application context"}"#,
        );

        let line_4 = lines.next().unwrap();
        assert_is_match!(timestamp_regex, line_4);
        assert_contains!(line_4, r#""level":"INFO""#);
        assert_contains!(line_4, r#""target":"strut_core::spindown::registry""#);
        assert_contains!(line_4, r#""fields":{"message":"Spindown initiated"}"#);

        let line_5 = lines.next().unwrap();
        assert_is_match!(timestamp_regex, line_5);
        assert_contains!(line_5, r#""level":"INFO""#);
        assert_contains!(line_5, r#""target":"strut_core::spindown::registry""#);
        assert_contains!(line_5, r#""fields":{"message":"Spindown completed"}"#);
    }

    fn sanitize(output: &[u8]) -> String {
        str::from_utf8(output)
            .unwrap()
            .chars()
            .filter(|c| {
                c.is_ascii_alphanumeric() || c.is_ascii_punctuation() || c.is_ascii_whitespace()
            })
            .collect()
    }
}
