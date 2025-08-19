use crate::repr::log::ProxyLevelFilter;
use humantime::parse_duration;
use log::LevelFilter;
use serde::de::{Error, IgnoredAny, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use sqlx_core::database::Database;
use sqlx_core::pool::PoolOptions;
use std::fmt::Formatter;
use std::marker::PhantomData;
use std::time::Duration;
use strut_factory::impl_deserialize_field;

pub struct ProxyPoolOptions<DB>
where
    DB: Database,
{
    inner: PoolOptions<DB>,
}

const _: () = {
    impl<DB> Default for ProxyPoolOptions<DB>
    where
        DB: Database,
    {
        fn default() -> Self {
            Self {
                inner: PoolOptions::default(),
            }
        }
    }

    impl<DB> From<ProxyPoolOptions<DB>> for PoolOptions<DB>
    where
        DB: Database,
    {
        fn from(value: ProxyPoolOptions<DB>) -> Self {
            value.inner
        }
    }
};

const _: () = {
    impl<'de, DB> Deserialize<'de> for ProxyPoolOptions<DB>
    where
        DB: Database,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_map(ProxyPoolOptionsVisitor::default())
        }
    }

    struct ProxyPoolOptionsVisitor<DB>
    where
        DB: Database,
    {
        phantom_data: PhantomData<DB>,
    }

    impl<DB> Default for ProxyPoolOptionsVisitor<DB>
    where
        DB: Database,
    {
        fn default() -> Self {
            Self {
                phantom_data: PhantomData,
            }
        }
    }

    impl<'de, DB> Visitor<'de> for ProxyPoolOptionsVisitor<DB>
    where
        DB: Database,
    {
        type Value = ProxyPoolOptions<DB>;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a map of connection pool options")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut min_connections = None;
            let mut max_connections = None;
            let mut test_before_acquire = None;
            let mut acquire_time_level: Option<ProxyLevelFilter> = None;
            let mut acquire_slow_level: Option<ProxyLevelFilter> = None;
            let mut acquire_slow_threshold = None;
            let mut acquire_timeout = None;
            let mut max_lifetime: Option<Option<Duration>> = None;
            let mut idle_timeout: Option<Option<Duration>> = None;

            while let Some(key) = map.next_key()? {
                match key {
                    ProxyPoolOptionsField::min_connections => {
                        key.poll(&mut map, &mut min_connections)?
                    }
                    ProxyPoolOptionsField::max_connections => {
                        key.poll(&mut map, &mut max_connections)?
                    }
                    ProxyPoolOptionsField::test_before_acquire => {
                        key.poll(&mut map, &mut test_before_acquire)?
                    }
                    ProxyPoolOptionsField::acquire_time_level => {
                        key.poll(&mut map, &mut acquire_time_level)?
                    }
                    ProxyPoolOptionsField::acquire_slow_level => {
                        key.poll(&mut map, &mut acquire_slow_level)?
                    }
                    ProxyPoolOptionsField::acquire_slow_threshold => {
                        let duration_string = map.next_value::<String>()?;
                        let duration = parse_duration(&duration_string).map_err(Error::custom)?;
                        acquire_slow_threshold = Some(duration);
                        IgnoredAny
                    }
                    ProxyPoolOptionsField::acquire_timeout => {
                        let duration_string = map.next_value::<String>()?;
                        let duration = parse_duration(&duration_string).map_err(Error::custom)?;
                        acquire_timeout = Some(duration);
                        IgnoredAny
                    }
                    ProxyPoolOptionsField::max_lifetime => {
                        let duration_string = map.next_value::<Option<String>>()?;
                        if let Some(duration_string) = duration_string {
                            let duration =
                                parse_duration(&duration_string).map_err(Error::custom)?;
                            max_lifetime = Some(Some(duration));
                        }
                        IgnoredAny
                    }
                    ProxyPoolOptionsField::idle_timeout => {
                        let duration_string = map.next_value::<Option<String>>()?;
                        if let Some(duration_string) = duration_string {
                            let duration =
                                parse_duration(&duration_string).map_err(Error::custom)?;
                            idle_timeout = Some(Some(duration));
                        }
                        IgnoredAny
                    }
                    ProxyPoolOptionsField::__ignore => map.next_value()?,
                };
            }

            let mut inner = PoolOptions::<DB>::default();

            if let Some(min_connections) = min_connections {
                inner = inner.min_connections(min_connections);
            }

            if let Some(max_connections) = max_connections {
                inner = inner.max_connections(max_connections);
            }

            if let Some(test_before_acquire) = test_before_acquire {
                inner = inner.test_before_acquire(test_before_acquire);
            }

            if let Some(acquire_time_level) = acquire_time_level {
                inner = inner.acquire_time_level(LevelFilter::from(acquire_time_level));
            }

            if let Some(acquire_slow_level) = acquire_slow_level {
                inner = inner.acquire_slow_level(LevelFilter::from(acquire_slow_level));
            }

            if let Some(acquire_slow_threshold) = acquire_slow_threshold {
                inner = inner.acquire_slow_threshold(acquire_slow_threshold);
            }

            if let Some(acquire_timeout) = acquire_timeout {
                inner = inner.acquire_timeout(acquire_timeout);
            }

            if let Some(max_lifetime) = max_lifetime {
                inner = inner.max_lifetime(max_lifetime);
            }

            if let Some(idle_timeout) = idle_timeout {
                inner = inner.idle_timeout(idle_timeout);
            }

            Ok(ProxyPoolOptions { inner })
        }
    }

    impl_deserialize_field!(
        ProxyPoolOptionsField,
        strut_deserialize::Slug::eq_as_slugs,
        min_connections,
        max_connections,
        test_before_acquire,
        acquire_time_level,
        acquire_slow_level,
        acquire_slow_threshold,
        acquire_timeout,
        max_lifetime,
        idle_timeout,
    );
};
