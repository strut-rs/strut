use crate::{QueueKind, QueueRenamingBehavior};
use serde::de::{Error, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::borrow::Cow;
use std::fmt::Formatter;
use std::sync::Arc;
use strut_core::AppReplica;
use strut_factory::impl_deserialize_field;

/// Defines of a RabbitMQ queue to be declared from the consuming side.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Queue {
    name: Arc<str>,
    kind: QueueKind,
    rename: QueueRenamingBehavior,
}

impl Default for Queue {
    fn default() -> Self {
        Self::named(Self::default_name())
    }
}

impl Queue {
    /// Creates a queue definition with the given name, falling on defaults for
    /// all other configuration.
    pub fn named(name: impl AsRef<str>) -> Self {
        Self {
            name: Arc::from(name.as_ref()),
            kind: Self::default_kind(),
            rename: Self::default_rename(),
        }
    }

    /// Creates a queue definition without a given name (which will cause RabbitMQ
    /// to generate one), falling on defaults for all other configuration.
    pub fn empty() -> Self {
        Self {
            name: "".into(),
            kind: Self::default_kind(),
            rename: Self::default_rename(),
        }
    }

    /// Re-creates this queue definition with the given kind.
    pub fn with_kind(self, kind: QueueKind) -> Self {
        Self { kind, ..self }
    }

    /// Re-creates this queue definition with the given renaming behavior.
    pub fn with_rename(self, rename: QueueRenamingBehavior) -> Self {
        Self { rename, ..self }
    }
}

impl Queue {
    /// Reports the queue name for this definition, taking into account the
    /// [renaming behavior](QueueRenamingBehavior).
    pub fn name(&self) -> Cow<'_, str> {
        // If the queue name is empty, allow the broker to generate the name
        if self.name.is_empty() {
            return Cow::Borrowed(&self.name);
        }

        match self.rename {
            QueueRenamingBehavior::Verbatim => Cow::Borrowed(&self.name),
            QueueRenamingBehavior::ReplicaIndex => {
                if let Some(index) = AppReplica::index() {
                    Cow::Owned(format!("{}.{}", &self.name, index))
                } else {
                    Cow::Borrowed(&self.name)
                }
            }
            QueueRenamingBehavior::ReplicaLifetimeId => Cow::Owned(format!(
                "{}.{}",
                &self.name,
                AppReplica::lifetime_id().dotted(),
            )),
        }
    }

    /// Reports whether the queue name for this definition is empty.
    ///
    /// An empty name is a signal to RabbitMQ to generate a random queue name,
    /// which may or may not be acceptable. For example, it is not possible to
    /// define a queue with an empty name for the built-in default exchange.
    pub fn is_empty(&self) -> bool {
        self.name.is_empty()
    }

    /// Reports the queue kind for this definition.
    pub fn kind(&self) -> QueueKind {
        self.kind
    }

    /// Reports the queue renaming behavior for this definition.
    pub fn rename(&self) -> QueueRenamingBehavior {
        self.rename
    }
}

impl Queue {
    fn default_name() -> &'static str {
        ""
    }

    fn default_kind() -> QueueKind {
        QueueKind::Classic
    }

    fn default_rename() -> QueueRenamingBehavior {
        QueueRenamingBehavior::Verbatim
    }
}

const _: () = {
    impl<'de> Deserialize<'de> for Queue {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_any(QueueVisitor)
        }
    }

    struct QueueVisitor;

    impl<'de> Visitor<'de> for QueueVisitor {
        type Value = Queue;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a map of RabbitMQ queue or a string queue name")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(Queue::named(value))
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut name: Option<String> = None;
            let mut kind = None;
            let mut rename = None;

            while let Some(key) = map.next_key()? {
                match key {
                    QueueField::name => key.poll(&mut map, &mut name)?,
                    QueueField::kind => key.poll(&mut map, &mut kind)?,
                    QueueField::rename => key.poll(&mut map, &mut rename)?,
                    QueueField::__ignore => map.next_value()?,
                };
            }

            let mut queue = Queue::named(name.as_deref().unwrap_or_else(|| Queue::default_name()));

            if let Some(kind) = kind {
                queue = queue.with_kind(kind);
            }

            if let Some(rename) = rename {
                queue = queue.with_rename(rename);
            }

            Ok(queue)
        }
    }

    impl_deserialize_field!(
        QueueField,
        strut_deserialize::Slug::eq_as_slugs,
        name,
        kind,
        rename | renaming | renaming_behavior,
    );
};

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn from_empty() {
        // Given
        let input = "{}";
        let expected_output = Queue::default();

        // When
        let actual_output = serde_yml::from_str::<Queue>(input).unwrap();

        // Then
        assert!(actual_output.is_empty());
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn from_string() {
        // Given
        let input = "test_queue";
        let expected_output = Queue {
            name: "test_queue".into(),
            ..Default::default()
        };

        // When
        let actual_output = serde_yml::from_str::<Queue>(input).unwrap();

        // Then
        assert!(!actual_output.is_empty());
        assert_eq!(expected_output, actual_output);
    }

    #[test]
    fn deserialize_from_full() {
        // Given
        let input = r#"
extra_field: ignored
name: test_queue
kind: quorum
"#;
        let expected_queue = Queue {
            name: "test_queue".into(),
            kind: QueueKind::Quorum,
            ..Default::default()
        };

        // When
        let actual_queue = serde_yml::from_str::<Queue>(input).unwrap();

        // Then
        assert!(!actual_queue.is_empty());
        assert_eq!(expected_queue, actual_queue);
    }
}
