#[cfg(test)]
mod tests {
    use test_util::Harness;

    #[test]
    fn case_01() {
        Harness::pass("cases/intro/01");
    }

    #[test]
    fn case_02() {
        Harness::pass("cases/intro/02");
    }

    #[test]
    fn case_03() {
        Harness::pass("cases/intro/03");
    }
}
