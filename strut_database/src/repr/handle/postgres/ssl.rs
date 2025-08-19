use sqlx::postgres::PgSslMode;
use strut_factory::Deserialize as StrutDeserialize;

/// Closely replicates the `sqlx` crate’s [`PgSslMode`] enum, providing the
/// [deserialization](serde::de::Deserialize) capability.
#[derive(Debug, StrutDeserialize)]
#[strut(eq_fn = strut_deserialize::Slug::eq_as_slugs)]
pub(crate) enum ProxyPgSslMode {
    /// The equivalent of [`PgSslMode::Disable`].
    #[strut(alias = "disabled", alias = "off", alias = "no", alias = "false")]
    Disable,

    /// The equivalent of [`PgSslMode::Allow`].
    #[strut(alias = "allowed")]
    Allow,

    /// The equivalent of [`PgSslMode::Prefer`].
    #[strut(alias = "preferred")]
    Prefer,

    /// The equivalent of [`PgSslMode::Require`].
    #[strut(alias = "required", alias = "req")]
    Require,

    /// The equivalent of [`PgSslMode::VerifyCa`].
    #[strut(
        alias = "ca",
        alias = "authority",
        alias = "certificate_authority",
        alias = "verify_authority",
        alias = "verify_certificate_authority"
    )]
    VerifyCa,

    /// The equivalent of [`PgSslMode::VerifyFull`].
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
    VerifyFull,
}

impl From<ProxyPgSslMode> for PgSslMode {
    fn from(value: ProxyPgSslMode) -> Self {
        match value {
            ProxyPgSslMode::Disable => PgSslMode::Disable,
            ProxyPgSslMode::Allow => PgSslMode::Allow,
            ProxyPgSslMode::Prefer => PgSslMode::Prefer,
            ProxyPgSslMode::Require => PgSslMode::Require,
            ProxyPgSslMode::VerifyCa => PgSslMode::VerifyCa,
            ProxyPgSslMode::VerifyFull => PgSslMode::VerifyFull,
        }
    }
}
