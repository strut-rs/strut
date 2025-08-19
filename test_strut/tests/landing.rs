#[cfg(test)]
mod tests {
    use test_util::Harness;

    #[test]
    fn passes() {
        Harness::pass("cases/landing");
    }
}
