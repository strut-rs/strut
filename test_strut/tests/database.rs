#[cfg(test)]
mod tests {
    use test_util::Harness;

    #[test]
    fn usage_toml() {
        Harness::pass("cases/database/usage_toml");
    }

    #[test]
    fn usage_yaml() {
        Harness::pass("cases/database/usage_yaml");
    }

    #[test]
    fn config_structure_toml() {
        Harness::pass("cases/database/config_structure_toml");
    }

    #[test]
    fn config_structure_yaml() {
        Harness::pass("cases/database/config_structure_yaml");
    }

    #[test]
    fn mysql_url_toml() {
        Harness::pass("cases/database/mysql_url_toml");
    }

    #[test]
    fn mysql_url_yaml() {
        Harness::pass("cases/database/mysql_url_yaml");
    }

    #[test]
    fn mysql_exploded_toml() {
        Harness::pass("cases/database/mysql_exploded_toml");
    }

    #[test]
    fn mysql_exploded_yaml() {
        Harness::pass("cases/database/mysql_exploded_yaml");
    }

    #[test]
    fn mysql_full_toml() {
        Harness::pass("cases/database/mysql_full_toml");
    }

    #[test]
    fn mysql_full_yaml() {
        Harness::pass("cases/database/mysql_full_yaml");
    }

    #[test]
    fn postgres_url_toml() {
        Harness::pass("cases/database/postgres_url_toml");
    }

    #[test]
    fn postgres_url_yaml() {
        Harness::pass("cases/database/postgres_url_yaml");
    }

    #[test]
    fn postgres_exploded_toml() {
        Harness::pass("cases/database/postgres_exploded_toml");
    }

    #[test]
    fn postgres_exploded_yaml() {
        Harness::pass("cases/database/postgres_exploded_yaml");
    }

    #[test]
    fn postgres_full_toml() {
        Harness::pass("cases/database/postgres_full_toml");
    }

    #[test]
    fn postgres_full_yaml() {
        Harness::pass("cases/database/postgres_full_yaml");
    }

    #[test]
    fn sqlite_url_toml() {
        Harness::pass("cases/database/sqlite_url_toml");
    }

    #[test]
    fn sqlite_url_yaml() {
        Harness::pass("cases/database/sqlite_url_yaml");
    }

    #[test]
    fn sqlite_exploded_toml() {
        Harness::pass("cases/database/sqlite_exploded_toml");
    }

    #[test]
    fn sqlite_exploded_yaml() {
        Harness::pass("cases/database/sqlite_exploded_yaml");
    }

    #[test]
    fn sqlite_full_toml() {
        Harness::pass("cases/database/sqlite_full_toml");
    }

    #[test]
    fn sqlite_full_yaml() {
        Harness::pass("cases/database/sqlite_full_yaml");
    }

    #[test]
    fn facade() {
        Harness::pass("cases/database/facade");
    }
}
