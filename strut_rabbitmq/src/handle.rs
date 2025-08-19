use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use secure_string::SecureString;
use serde::de::{DeserializeSeed, Error, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::any::type_name;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;
use strut_deserialize::{Slug, SlugMap};
use strut_factory::impl_deserialize_field;
use strut_util::BackoffConfig;

const VHOST_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b'/') // Encode '/' as %2F
    .add(b'?') // Encode '?' as %3F
    .add(b'#') // Encode '#' as %23
    .add(b'%'); // Encode '%' as %25 (to avoid ambiguity)

/// Represents a collection of uniquely named [`Handle`]s.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct HandleCollection {
    handles: SlugMap<Handle>,
}

/// Defines a connection handle for a RabbitMQ cluster, consisting primarily of
/// a set of credentials, along with a bit of metadata for logging/debugging
/// purposes.
///
/// This handle by itself does not implement any connection logic.
#[derive(Clone, PartialEq)]
pub struct Handle {
    name: Arc<str>,
    identifier: Arc<str>,
    dsn: SecureString,
    backoff: BackoffConfig,
}

/// Groups the pieces of a RabbitMQ DSN for convenient passing into
/// [`Handle::new`].
pub struct DsnChunks<H, U, P, VH>
where
    H: AsRef<str>,
    U: AsRef<str>,
    P: Into<SecureString>,
    VH: AsRef<str>,
{
    /// The `localhost` part of `amqp://user:pass@localhost:5672/%2F`.
    pub host: H,
    /// The `5672` part of `amqp://user:pass@localhost:5672/%2F`.
    pub port: u16,
    /// The `user` part of `amqp://user:pass@localhost:5672/%2F`.
    pub user: U,
    /// The `pass` part of `amqp://user:pass@localhost:5672/%2F`.
    ///
    /// This has to be represented with anything that implements
    /// [`Into<SecureString>`], which includes `&str`.
    pub password: P,
    /// The `%2F` part of `amqp://user:pass@localhost:5672/%2F`.
    ///
    /// This does **not** need to be percent-encoded. [`Handle`] takes
    /// care of percent-encoding. In the example above, the equivalent
    /// human-readable string `"/"` will work just fine.
    pub vhost: VH,
}

impl Handle {
    /// Creates a new handle with the given name and composes the DSN from the
    /// given [`chunks`](DsnChunks).
    ///
    /// Takes care of securing the password against _accidental_ debug-printing.
    /// Ensures proper percent-encoding of the `vhost`; there is no need to
    /// pre-encode it.
    pub fn new<H, U, P, VH>(name: impl AsRef<str>, chunks: DsnChunks<H, U, P, VH>) -> Self
    where
        H: AsRef<str>,
        U: AsRef<str>,
        P: Into<SecureString>,
        VH: AsRef<str>,
    {
        let name = Arc::from(name.as_ref());

        let vhost = Self::ensure_encoded_vhost(chunks.vhost.as_ref());
        let identifier = Self::compose_identifier(
            chunks.host.as_ref(),
            chunks.port,
            chunks.user.as_ref(),
            vhost.as_ref(),
        );

        let password = chunks.password.into();
        let dsn = Self::compose_dsn(
            chunks.host.as_ref(),
            chunks.port,
            chunks.user.as_ref(),
            &password,
            vhost.as_ref(),
        );

        let backoff = BackoffConfig::default();

        Self {
            name,
            identifier,
            dsn,
            backoff,
        }
    }

    /// Re-create this [`Handle`] with the given [`BackoffConfig`].
    pub fn with_backoff(self, backoff: BackoffConfig) -> Self {
        Self { backoff, ..self }
    }

    /// Ensures that the given `vhost` value is correctly percent-encoded to be
    /// included in a DSN.
    fn ensure_encoded_vhost(vhost: &str) -> Cow<'_, str> {
        utf8_percent_encode(vhost, VHOST_ENCODE_SET).into()
    }

    /// Composes a non-sensitive identifier useful for debug-printing a handle.
    fn compose_identifier(host: &str, port: u16, user: &str, vhost: &str) -> Arc<str> {
        Arc::from(format!("{}@{}:{}/{}", user, host, port, vhost))
    }

    /// Composes a sensitive DSN to be used for connecting to the RabbitMQ cluster.
    fn compose_dsn(
        host: &str,
        port: u16,
        user: &str,
        password: &SecureString,
        vhost: &str,
    ) -> SecureString {
        SecureString::from(format!(
            "amqp://{}:{}@{}:{}/{}",
            user,
            password.unsecure(),
            host,
            port,
            vhost,
        ))
    }
}

impl HandleCollection {
    /// Reports whether this collection contains a [`Handle`] with the
    /// given unique name.
    pub fn contains(&self, name: &str) -> bool {
        self.handles.contains_key(name)
    }

    /// Retrieves `Some` reference to a [`Handle`] from this collection
    /// under the given name, or `None`, if the name is not present in the
    /// collection.
    pub fn get(&self, name: &str) -> Option<&Handle> {
        self.handles.get(name)
    }

    /// Retrieves a reference to a [`Handle`] from this collection under
    /// the given name. Panics if the name is not present in the collection.
    pub fn expect(&self, name: &str) -> &Handle {
        self.get(name)
            .unwrap_or_else(|| panic!("requested an undefined RabbitMQ handle '{}'", name))
    }
}

impl Handle {
    /// Reports the handle name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Reports the handle identifier, which is the normal connection DSN, but
    /// with the password obscured. This identifier is generally safe for debug
    /// logging.
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    /// Reports the handle DSN.
    pub fn dsn(&self) -> &SecureString {
        &self.dsn
    }

    /// Exposes the exponential [`Backoff`](strut_util::Backoff) configuration
    /// for this handle.
    pub fn backoff(&self) -> &BackoffConfig {
        &self.backoff
    }
}

/// Convenience implementation for providing partially hard-coding chunks.
impl Default for DsnChunks<&str, &str, &str, &str> {
    fn default() -> Self {
        Self {
            host: Handle::default_host(),
            port: Handle::default_port(),
            user: Handle::default_user(),
            password: Handle::default_password(),
            vhost: Handle::default_vhost(),
        }
    }
}

impl Handle {
    fn default_name() -> &'static str {
        "default"
    }

    fn default_host() -> &'static str {
        "localhost"
    }

    fn default_port() -> u16 {
        5672
    }

    fn default_user() -> &'static str {
        "guest"
    }

    fn default_password() -> &'static str {
        "guest"
    }

    fn default_vhost() -> &'static str {
        "/"
    }
}

impl Default for Handle {
    fn default() -> Self {
        Self::new(Self::default_name(), DsnChunks::default())
    }
}

/// Omits `dsn` from debug representation. DSN is largely safe (it’s a [`SecureString`]),
/// but its inclusion adds no valuable debug information.
impl Debug for Handle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(type_name::<Self>())
            .field("name", &self.name)
            .field("identifier", &self.identifier)
            .field("backoff", &self.backoff)
            .finish()
    }
}

impl Display for Handle {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        f.write_str(&self.identifier)
    }
}

impl AsRef<Handle> for Handle {
    fn as_ref(&self) -> &Handle {
        self
    }
}

impl AsRef<HandleCollection> for HandleCollection {
    fn as_ref(&self) -> &HandleCollection {
        self
    }
}

const _: () = {
    impl<S> FromIterator<(S, Handle)> for HandleCollection
    where
        S: Into<Slug>,
    {
        fn from_iter<T: IntoIterator<Item = (S, Handle)>>(iter: T) -> Self {
            let handles = iter.into_iter().collect();

            Self { handles }
        }
    }

    impl<const N: usize, S> From<[(S, Handle); N]> for HandleCollection
    where
        S: Into<Slug>,
    {
        fn from(value: [(S, Handle); N]) -> Self {
            value.into_iter().collect()
        }
    }
};

const _: () = {
    impl<'de> Deserialize<'de> for HandleCollection {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_map(HandleCollectionVisitor)
        }
    }

    struct HandleCollectionVisitor;

    impl<'de> Visitor<'de> for HandleCollectionVisitor {
        type Value = HandleCollection;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a map of RabbitMQ handles")
        }

        fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let grouped = Slug::group_map(map)?;
            let mut handles = HashMap::with_capacity(grouped.len());

            for (key, value) in grouped {
                let seed = HandleSeed {
                    name: key.original(),
                };
                let handle = seed.deserialize(value).map_err(Error::custom)?;
                handles.insert(key, handle);
            }

            Ok(HandleCollection {
                handles: SlugMap::new(handles),
            })
        }
    }

    struct HandleSeed<'a> {
        name: &'a str,
    }

    impl<'de> DeserializeSeed<'de> for HandleSeed<'_> {
        type Value = Handle;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_map(HandleSeedVisitor { name: self.name })
        }
    }

    struct HandleSeedVisitor<'a> {
        name: &'a str,
    }

    impl<'de> Visitor<'de> for HandleSeedVisitor<'_> {
        type Value = Handle;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a map of RabbitMQ handle")
        }

        fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            visit_handle(map, Some(self.name))
        }
    }

    impl<'de> Deserialize<'de> for Handle {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_map(HandleVisitor)
        }
    }

    struct HandleVisitor;

    impl<'de> Visitor<'de> for HandleVisitor {
        type Value = Handle;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a map of RabbitMQ handle")
        }

        fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            visit_handle(map, None)
        }
    }

    fn visit_handle<'de, A>(mut map: A, known_name: Option<&str>) -> Result<Handle, A::Error>
    where
        A: MapAccess<'de>,
    {
        // Type hints are needed on `String`s to avoid deserializer expecting a
        // borrowed string, which not all deserializers support.
        let mut name: Option<String> = None;
        let mut host: Option<String> = None;
        let mut port = None;
        let mut user: Option<String> = None;
        let mut password: Option<SecureString> = None;
        let mut vhost: Option<String> = None;

        while let Some(key) = map.next_key()? {
            match key {
                HandleField::name => key.poll(&mut map, &mut name)?,
                HandleField::host => key.poll(&mut map, &mut host)?,
                HandleField::port => key.poll(&mut map, &mut port)?,
                HandleField::user => key.poll(&mut map, &mut user)?,
                HandleField::password => key.poll(&mut map, &mut password)?,
                HandleField::vhost => key.poll(&mut map, &mut vhost)?,
                HandleField::__ignore => map.next_value()?,
            };
        }

        let name = match known_name {
            Some(known_name) => known_name,
            None => name.as_deref().unwrap_or_else(|| Handle::default_name()),
        };

        // “Useless” closures are needed to avoid lifetime issues
        let chunks = DsnChunks {
            host: host.as_deref().unwrap_or_else(|| Handle::default_host()),
            port: port.unwrap_or_else(Handle::default_port),
            user: user.as_deref().unwrap_or_else(|| Handle::default_user()),
            password: password.unwrap_or_else(|| Handle::default_password().into()),
            vhost: vhost.as_deref().unwrap_or_else(|| Handle::default_vhost()),
        };

        Ok(Handle::new(name, chunks))
    }

    impl_deserialize_field!(
        HandleField,
        strut_deserialize::Slug::eq_as_slugs,
        name,
        host | hostname,
        port,
        user | username,
        password,
        vhost,
    );
};

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn deserialize_from_empty() {
        // Given
        let input = "";
        let expected_output = Handle::default();

        // When
        let actual_output = serde_yml::from_str::<Handle>(input).unwrap();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn deserialize_from_full() {
        // Given
        let input = r#"
name: test_handle
host: test_host
port: 8080
user: test_user
password: test_password
vhost: test_vhost
"#;
        let expected_output = Handle::new(
            "test_handle",
            DsnChunks {
                host: "test_host",
                port: 8080,
                user: "test_user",
                password: "test_password",
                vhost: "test_vhost",
            },
        );

        // When
        let actual_output = serde_yml::from_str::<Handle>(input).unwrap();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn deserialize_collection_from_empty() {
        // Given
        let input = "";
        let expected_output = HandleCollection::default();

        // When
        let actual_output = serde_yml::from_str::<HandleCollection>(input).unwrap();

        // Then
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn deserialize_collection_from_full() {
        // Given
        let input = r#"
test_handle_a: {}
test_handle_b:
  host: test_host
  port: 8080
"#;
        let expected_output = HandleCollection::from([
            (
                "test_handle_a",
                Handle::new("test_handle_a", DsnChunks::default()),
            ),
            (
                "test_handle_b",
                Handle::new(
                    "test_handle_b",
                    DsnChunks {
                        host: "test_host",
                        port: 8080,
                        ..Default::default()
                    },
                ),
            ),
        ]);

        // When
        let actual_output = serde_yml::from_str::<HandleCollection>(input).unwrap();

        // Then
        assert_eq!(expected_output, actual_output);
    }
}
