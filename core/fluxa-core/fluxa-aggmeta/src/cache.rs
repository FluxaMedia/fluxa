use std::time::Duration;

const MINUTE: u64 = 60;
const HOUR: u64 = 3600;
const DAY: u64 = 86400;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TtlClass {
    Meta,
    Catalog,
    JikanDetail,
    StaticCatalog,
    TvdbApi,
    TvmazeApi,
    AnilistCatalog,
    StremthruGenres,
}

impl TtlClass {
    pub fn ttl(self) -> Duration {
        let secs = match self {
            TtlClass::Meta => 7 * DAY,
            TtlClass::Catalog => DAY,
            TtlClass::JikanDetail => 30 * DAY,
            TtlClass::StaticCatalog => 30 * DAY,
            TtlClass::TvdbApi | TtlClass::TvmazeApi => 12 * HOUR,
            TtlClass::AnilistCatalog => DAY,
            TtlClass::StremthruGenres => 7 * DAY,
        };
        Duration::from_secs(secs)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NegativeKind {
    NotFound,
    RateLimited,
    TemporaryError,
    PermanentError,
    EmptyResult,
}

impl NegativeKind {
    pub fn ttl(self) -> Duration {
        let secs = match self {
            NegativeKind::RateLimited => 2 * MINUTE,
            NegativeKind::TemporaryError => 5 * MINUTE,
            NegativeKind::EmptyResult => 30 * MINUTE,
            NegativeKind::NotFound => 6 * HOUR,
            NegativeKind::PermanentError => 12 * HOUR,
        };
        Duration::from_secs(secs)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Stored<T> {
    Value(T),
    Negative(NegativeKind),
}

pub fn store_ttl(class: TtlClass, degraded: bool) -> Option<Duration> {
    (!degraded).then(|| class.ttl())
}

#[derive(Clone, Debug, Default)]
pub struct KeyBuilder {
    parts: Vec<String>,
}

impl KeyBuilder {
    pub fn new(namespace: &str) -> Self {
        Self {
            parts: vec![namespace.to_string()],
        }
    }

    pub fn part(mut self, label: &str, value: &str) -> Self {
        self.parts.push(format!("{label}={value}"));
        self
    }

    pub fn opt(mut self, label: &str, value: Option<&str>) -> Self {
        if let Some(v) = value.filter(|v| !v.is_empty()) {
            self.parts.push(format!("{label}={v}"));
        }
        self
    }

    pub fn build(self) -> String {
        self.parts.join("|")
    }
}

pub struct MetaKey<'a> {
    pub kind: &'a str,
    pub id: &'a str,
    pub language: &'a str,
    pub provider: &'a str,
    pub episode_order: &'a str,
    pub season_numbering: &'a str,
    pub signature: &'a str,
}

impl MetaKey<'_> {
    pub fn build(&self) -> String {
        let years = (self.season_numbering != "sequential" && !self.season_numbering.is_empty())
            .then_some(self.season_numbering);
        KeyBuilder::new("meta")
            .part("type", self.kind)
            .part("id", self.id)
            .part("lang", self.language)
            .part("provider", self.provider)
            .opt(
                "order",
                (self.episode_order != "official").then_some(self.episode_order),
            )
            .opt("numbering", years)
            .opt("sig", Some(self.signature))
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(numbering: &str, order: &str) -> String {
        MetaKey {
            kind: "series",
            id: "tt1",
            language: "en-US",
            provider: "tvdb",
            episode_order: order,
            season_numbering: numbering,
            signature: "",
        }
        .build()
    }

    #[test]
    fn default_users_keep_a_stable_key() {
        assert_eq!(key("sequential", "official"), key("", "official"));
        assert!(!key("sequential", "official").contains("numbering"));
    }

    #[test]
    fn non_default_profiles_change_the_key() {
        assert_ne!(key("years", "official"), key("sequential", "official"));
        assert_ne!(key("sequential", "dvd"), key("sequential", "official"));
    }

    #[test]
    fn degraded_results_are_not_stored() {
        assert!(store_ttl(TtlClass::Meta, true).is_none());
        assert_eq!(store_ttl(TtlClass::Meta, false), Some(TtlClass::Meta.ttl()));
    }

    #[test]
    fn rate_limits_have_the_shortest_negative_ttl() {
        let kinds = [
            NegativeKind::NotFound,
            NegativeKind::TemporaryError,
            NegativeKind::PermanentError,
            NegativeKind::EmptyResult,
        ];
        assert!(
            kinds
                .iter()
                .all(|k| NegativeKind::RateLimited.ttl() < k.ttl())
        );
    }
}
