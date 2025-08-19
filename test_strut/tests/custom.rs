#[cfg(test)]
mod tests {
    use test_util::Harness;

    #[test]
    fn case_full() {
        Harness::pass("cases/custom/full");
    }
}
