use sqlx::mysql::MySqlSslMode;
use strut_factory::Deserialize as StrutDeserialize;

/// Closely replicates the `sqlx` crate’s [`MySqlSslMode`] enum, providing the
/// [deserialization](serde::de::Deserialize) capability.
#[derive(Debug, StrutDeserialize)]
#[strut(eq_fn = strut_deserialize::Slug::eq_as_slugs)]
pub(crate) enum ProxyMySqlSslMode {
    /// The equivalent of [`MySqlSslMode::Disabled`].
    #[strut(alias = "disable", alias = "off", alias = "no", alias = "false")]
    Disabled,

    /// The equivalent of [`MySqlSslMode::Preferred`].
    #[strut(alias = "prefer")]
    Preferred,

    /// The equivalent of [`MySqlSslMode::Required`].
    #[strut(alias = "require", alias = "req")]
    Required,

    /// The equivalent of [`MySqlSslMode::VerifyCa`].
    #[strut(
        alias = "ca",
        alias = "authority",
        alias = "certificate_authority",
        alias = "verify_authority",
        alias = "verify_certificate_authority"
    )]
    VerifyCa,

    /// The equivalent of [`MySqlSslMode::VerifyIdentity`].
    #[strut(
        alias = "id",
        alias = "identity",
        alias = "full",
        alias = "host_identity",
        alias = "server_identity",
        alias = "verify_host_identity",
        alias = "verify_server_identity",
        alias = "verify_id",
        alias = "verify_full"
    )]
    VerifyIdentity,
}

impl From<ProxyMySqlSslMode> for MySqlSslMode {
    fn from(value: ProxyMySqlSslMode) -> Self {
        match value {
            ProxyMySqlSslMode::Disabled => MySqlSslMode::Disabled,
            ProxyMySqlSslMode::Preferred => MySqlSslMode::Preferred,
            ProxyMySqlSslMode::Required => MySqlSslMode::Required,
            ProxyMySqlSslMode::VerifyCa => MySqlSslMode::VerifyCa,
            ProxyMySqlSslMode::VerifyIdentity => MySqlSslMode::VerifyIdentity,
        }
    }
}
