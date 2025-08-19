use crate::profile::name::Name;
use std::env;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::sync::OnceLock;

/// Implements the stack-allocated string for storing custom profile names.
mod name;

/// The string that is recognized as the [**production**](AppProfile::Prod)
/// profile.
pub const APP_PROFILE_PROD: &str = "prod";

/// The string that is recognized as the [**development**](AppProfile::Dev)
/// profile.
pub const APP_PROFILE_DEV: &str = "dev";

/// The string that is recognized as the [**test**](AppProfile::Test) profile.
pub const APP_PROFILE_TEST: &str = "test";

/// Represents the runtime profile of the application. The profile affects
/// primarily which set of configuration files is applied, and the application
/// is free to implement any profile-specific logic.
///
/// There are three **well-known profiles**:
///
/// - [**Production**](AppProfile::Prod) profile.
/// - [**Development**](AppProfile::Dev) profile.
/// - [**Test**](AppProfile::Test) profile.
///
/// Then, there are [**custom profiles**](AppProfile::Custom), which can take
/// any lowercase ASCII-only [name](Name) within the limit of
/// [`NAME_MAX_LEN`](name::NAME_MAX_LEN) characters. The custom profile names
/// are always forced to lowercase.
///
/// This enumeration defines the
/// [**active runtime profile**](AppProfile::active), which is lazily discerned
/// from the environment on the first access, and is then statically stored for
/// the whole runtime of the application. See the
/// [`discern`](AppProfile::discern) method for details on how the active
/// profile is chosen.
///
/// ## Usage
///
/// The intended way to match against the active profile is:
///
/// ```
/// use strut_core::AppProfile;
///
/// match AppProfile::active() {
///     AppProfile::Prod => println!("We are in prod"),
///     AppProfile::Dev => println!("We are in dev"),
///     AppProfile::Test => println!("We are in test"),
///     AppProfile::Custom(name) => {
///         match name.as_str() {
///             "preprod" => println!("We are in preprod"),
///             other => println!("We are in {}", other)
///         };
///     }
/// };
/// ```
///
/// ## Implicit detection
///
/// On the surface it looks like the three [`AppProfile`]s:
/// [`prod`](AppProfile::Prod), [`dev`](AppProfile::Dev), and
/// [`test`](AppProfile::Test) match quite nicely with the three out of four
/// built-in
/// [Cargo compilation profiles](https://doc.rust-lang.org/cargo/reference/profiles.html):
/// `release`, `dev`, and `test` (leaving the fourth, `bench`, unmatched).
///
/// Having noted this similarity, a logical next step would be to use the Cargo
/// compilation profile to automatically and implicitly infer the
/// [active](AppProfile::active) [`AppProfile`] without requiring any custom
/// environment variables to be set.
///
/// Unfortunately, in the current implementation of Cargo this is not feasible.
///
/// Firstly, [`AppProfile`] is a **runtime** construct, and the Cargo profiles
/// exist only during **compilation**. Current implementation of Cargo provides
/// no relevant runtime indicators: Are we running a unit test binary? A
/// benchmark test? A compiled binary crate? We don’t know. That by itself could
/// be a disqualifier, but we must also give a chance to the build scripts.
///
/// A build script is able to capture some of the compilation environment and
/// pass it on to this crate’s compilation environment. We could then
/// theoretically compile a different [`AppProfile`] depending on the
/// compilation environment.
///
/// But that is not feasible either.
///
/// For one, there is no way to distinguish whether a test is being compiled:
/// all test dependencies (including this crate) are compiled with the `dev` or
/// `release` Cargo profile, not `test`.
///
/// We could technically infer (and pass on) whether the `release` profile is
/// being used, but that alone tells us almost nothing. For example, there is
/// nothing preventing the tests from compiling their dependencies with the
/// `release` Cargo profile.
///
/// That all being said, we choose to not implement any “auto-detection” logic
/// for the active profile, and instead rely on the special `APP_PROFILE`
/// environment variable, falling back on the [`dev`](AppProfile::Dev) profile
/// as the default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AppProfile {
    /// The **production** profile, used for live, customer-facing deployments.
    /// Typically configured with optimized performance settings, reduced
    /// logging verbosity, and real external services.
    ///
    /// If your application needs any prod-specific runtime configuration, it is
    /// advised to run the compiled binary with the special environment variable:
    ///
    /// `APP_PROFILE=prod ./compiled_binary ...`
    Prod,

    /// The **development** profile, used during local or in-team development.
    /// Commonly features hot-reloading, detailed logging, and integration with
    /// mock or local services.
    ///
    /// This profile is the default if the special `APP_PROFILE` environment
    /// variable is not set.
    Dev,

    /// The **test** profile, used in automated testing environments. Often
    /// configured with in-memory databases, deterministic behavior, and fast
    /// execution settings.
    ///
    /// If your application needs any test-specific runtime configuration, it is
    /// advised to run Cargo with the special environment variable:
    ///
    /// `APP_PROFILE=test cargo test ...`
    Test,

    /// Any custom profile that a given application may choose to have.
    ///
    /// Examples from across the industry include `"preprod"`, `"qa"`, `"uat"`,
    /// `"staging"`, `"sandbox"`, `"demo"`, `"canary"`, `"perf"`, `"local"`,
    /// `"ci"`, `"nightly"`, `"hotfix"`, etc.
    ///
    /// The name of a custom profile is limited to
    /// [`NAME_MAX_LEN`](name::NAME_MAX_LEN) ASCII lowercase characters. All
    /// examples above fit into that limit.
    ///
    /// If your application needs any env-specific runtime configuration in a
    /// custom environment, it is advised to run the compiled binary with the
    /// special environment variable:
    ///
    /// `APP_PROFILE=preprod ./compiled_binary ...`
    Custom(Name),
}

impl AppProfile {
    /// Returns the active runtime [`AppProfile`], lazily
    /// [discerned](AppProfile::discern).
    pub fn active() -> &'static AppProfile {
        static APP_PROFILE: OnceLock<AppProfile> = OnceLock::new();

        APP_PROFILE.get_or_init(Self::discern)
    }

    /// Constructs a new [`AppProfile`] with the given name.
    pub fn new(name: impl AsRef<str>) -> Self {
        let name = Name::new(name);

        match name.as_str() {
            "prod" => Self::Prod,
            "dev" => Self::Dev,
            "test" => Self::Test,
            _ => Self::Custom(name),
        }
    }

    /// Reads the active runtime [`AppProfile`] from the `APP_PROFILE`
    /// environment variable. If it is not set, delegates to
    /// [`AppProfile::default`].
    fn discern() -> Self {
        // Detect if profile is set explicitly
        if let Ok(profile) = env::var("APP_PROFILE") {
            return Self::new(profile);
        }

        // Otherwise, return the default
        Self::default()
    }
}

impl AppProfile {
    /// Reports whether the [`active`](AppProfile::active) profile is the
    /// [**production**](AppProfile::Prod) profile.
    pub fn active_is_prod() -> bool {
        Self::active().is_prod()
    }

    /// Reports whether the [`active`](AppProfile::active) profile is the
    /// [**development**](AppProfile::Dev) profile.
    pub fn active_is_dev() -> bool {
        Self::active().is_dev()
    }

    /// Reports whether the [`active`](AppProfile::active) profile is the
    /// [**test**](AppProfile::Test) profile.
    pub fn active_is_test() -> bool {
        Self::active().is_test()
    }

    /// Reports whether the [`active`](AppProfile::active) profile is the
    /// given profile name.
    pub fn active_is(given: impl AsRef<str>) -> bool {
        Self::active().is(given)
    }
}

impl AppProfile {
    /// Reports whether this [`AppProfile`] is the
    /// [**production**](AppProfile::Prod) profile.
    pub fn is_prod(&self) -> bool {
        matches!(self, Self::Prod)
    }

    /// Reports whether this [`AppProfile`] is the
    /// [**development**](AppProfile::Dev) profile.
    pub fn is_dev(&self) -> bool {
        matches!(self, Self::Dev)
    }

    /// Reports whether this [`AppProfile`] is the [**test**](AppProfile::Test)
    /// profile.
    pub fn is_test(&self) -> bool {
        matches!(self, Self::Test)
    }

    /// Reports whether this [`AppProfile`] matches the given profile name.
    ///
    /// Before comparing, forces the given name into the same restrictions that
    /// are [applied](Name::new) to a profile name normally.
    pub fn is(&self, given: impl AsRef<str>) -> bool {
        self.as_str() == Name::new(given).as_str()
    }

    /// Exposes a view on this [`AppProfile`] as a string slice.
    pub fn as_str(&self) -> &str {
        match self {
            AppProfile::Prod => APP_PROFILE_PROD,
            AppProfile::Dev => APP_PROFILE_DEV,
            AppProfile::Test => APP_PROFILE_TEST,
            AppProfile::Custom(name) => name.as_str(),
        }
    }
}

impl Display for AppProfile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl AsRef<AppProfile> for AppProfile {
    fn as_ref(&self) -> &AppProfile {
        self
    }
}

impl AsRef<str> for AppProfile {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Deref for AppProfile {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl From<&str> for AppProfile {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl Default for AppProfile {
    /// Defines the default profile if it cannot be inferred from the special
    /// `APP_PROFILE` environment variable.
    fn default() -> Self {
        Self::Dev
    }
}
