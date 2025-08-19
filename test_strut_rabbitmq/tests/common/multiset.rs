use parking_lot::Mutex;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;
use strut_rabbitmq::{Envelope, EnvelopeStack, NonEmpty};

/// Transforms a slice of `&str` into a vector of `String` for easy assertions.
pub fn multiset<S>(v: &[S]) -> Multiset<String>
where
    S: AsRef<str>,
{
    v.iter().map(|s| s.as_ref().to_string()).collect()
}

/// Collects items disregarding of ordering and keeps count of duplicates.
#[derive(Debug, PartialEq, Eq)]
pub struct Multiset<T>
where
    T: Eq + Hash,
{
    counts: HashMap<T, usize>,
}

impl<T> Multiset<T>
where
    T: Eq + Hash,
{
    pub fn count(&self) -> usize {
        let mut result = 0usize;

        for (_key, count) in self.counts.iter() {
            result += count;
        }

        result
    }

    pub fn is_disjoint_from(&self, other: &Multiset<T>) -> bool {
        self.counts
            .keys()
            .all(|key| !other.counts.contains_key(key))
    }
}

impl<T> FromIterator<T> for Multiset<T>
where
    T: Eq + Hash,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut counts = HashMap::new();
        for item in iter {
            *counts.entry(item).or_insert(0) += 1;
        }
        Multiset { counts }
    }
}

impl<X> From<Dropbox<X>> for Multiset<X::Extracted>
where
    X: Extractor,
{
    fn from(value: Dropbox<X>) -> Self {
        let Dropbox { storage, .. } = value;
        let values = std::mem::take(&mut *storage.lock());

        values.into_iter().collect()
    }
}

#[derive(Debug)]
pub struct Dropbox<X>
where
    X: Extractor,
{
    extractor: X,
    storage: Mutex<Vec<X::Extracted>>,
}

pub type PayloadDropbox<P> = Dropbox<PayloadExtractor<P>>;
pub type StringDropbox = PayloadDropbox<String>;

impl<X> Dropbox<X>
where
    X: Extractor,
{
    pub fn new(extractor: X) -> Self {
        Self {
            extractor,
            storage: Mutex::new(Vec::new()),
        }
    }

    pub fn to_multiset(self) -> Multiset<X::Extracted> {
        Multiset::from(self)
    }
}

impl<X, P> Dropbox<X>
where
    X: Extractor<Payload = P>,
{
    pub async fn add(&self, envelope: Envelope<P>) {
        self.storage.lock().push(self.extractor.extract(&envelope));
        envelope.complete().await;
    }

    pub async fn add_many(&self, envelopes: NonEmpty<Envelope<P>>)
    where
        P: Send,
    {
        for envelope in envelopes.iter() {
            self.storage.lock().push(self.extractor.extract(envelope));
        }
        envelopes.complete_all().await;
    }
}

impl<P> Dropbox<PayloadExtractor<P>>
where
    P: Debug + Default + Clone + Eq + Hash,
{
    pub fn new_payload() -> Self {
        Self::new(PayloadExtractor::default())
    }
}

impl<P> Dropbox<MessageIdExtractor<P>>
where
    P: Debug + Default,
{
    pub fn new_message_id() -> Self {
        Self::new(MessageIdExtractor::default())
    }
}

impl<P> Dropbox<RoutingKeyExtractor<P>>
where
    P: Debug + Default,
{
    pub fn new_routing_key() -> Self {
        Self::new(RoutingKeyExtractor::default())
    }
}

pub trait Extractor: Default + Debug {
    type Payload;
    type Extracted: Eq + Hash;

    fn extract(&self, envelope: &Envelope<Self::Payload>) -> Self::Extracted;
}

#[derive(Debug, Default)]
pub struct PayloadExtractor<P>
where
    P: Debug + Default,
{
    _phantom: PhantomData<P>,
}

impl<P> Extractor for PayloadExtractor<P>
where
    P: Debug + Default + Eq + Hash + Clone,
{
    type Payload = P;
    type Extracted = P;

    fn extract(&self, envelope: &Envelope<P>) -> P {
        envelope.payload().clone()
    }
}

#[derive(Debug, Default)]
pub struct MessageIdExtractor<P>
where
    P: Debug + Default,
{
    _phantom: PhantomData<P>,
}

impl<P> Extractor for MessageIdExtractor<P>
where
    P: Debug + Default,
{
    type Payload = P;
    type Extracted = u64;

    fn extract(&self, envelope: &Envelope<Self::Payload>) -> Self::Extracted {
        envelope.message_id().unwrap()
    }
}

#[derive(Debug, Default)]
pub struct RoutingKeyExtractor<P>
where
    P: Debug + Default,
{
    _phantom: PhantomData<P>,
}

impl<P> Extractor for RoutingKeyExtractor<P>
where
    P: Debug + Default,
{
    type Payload = P;
    type Extracted = String;

    fn extract(&self, envelope: &Envelope<Self::Payload>) -> Self::Extracted {
        envelope.routing_key().to_string()
    }
}
