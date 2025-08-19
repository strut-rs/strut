#[cfg(test)]
mod tests {
    use assertables::assert_contains;
    use test_util::Harness;

    #[test]
    fn args() {
        let output = Harness::fail("cases/attribute/args");
        let stderr = str::from_utf8(output.stderr.as_slice()).unwrap();

        assert_contains!(
            stderr,
            "this attribute is only allowed on the `main` function without arguments",
        );
    }

    #[test]
    fn non_async() {
        let output = Harness::fail("cases/attribute/non_async");
        let stderr = str::from_utf8(output.stderr.as_slice()).unwrap();

        assert_contains!(
            stderr,
            "this attribute is only allowed on the `async main` function",
        );
    }

    #[test]
    fn non_main() {
        let output = Harness::fail("cases/attribute/non_main");
        let stderr = str::from_utf8(output.stderr.as_slice()).unwrap();

        assert_contains!(
            stderr,
            "this attribute is only allowed on the `main` function",
        );
    }

    #[test]
    fn return_type() {
        let output = Harness::fail("cases/attribute/return_type");
        let stderr = str::from_utf8(output.stderr.as_slice()).unwrap();

        assert_contains!(
            stderr,
            "this attribute is only allowed on the `main` function that doesn’t declare a return type",
        );
    }
}
