const MOVIE: [&str; 5] = ["G", "PG", "PG-13", "R", "NC-17"];
const TV: [&str; 6] = ["TV-Y", "TV-Y7", "TV-G", "TV-PG", "TV-14", "TV-MA"];
const UNRATED: [&str; 6] = ["NR", "UR", "UNRATED", "NOT RATED", "N/A", "NONE"];

#[derive(Clone, Debug, PartialEq)]
pub struct AgeGate {
    pub cap: Option<String>,
    pub allow_unrated: bool,
}

impl Default for AgeGate {
    fn default() -> Self {
        Self {
            cap: None,
            allow_unrated: true,
        }
    }
}

fn is_unrated(cert: &str) -> bool {
    let c = cert.trim().to_uppercase();
    c.is_empty() || UNRATED.contains(&c.as_str())
}

pub fn mal_rating(raw: &str) -> String {
    let head = raw.split(" - ").next().unwrap_or(raw).trim().to_uppercase();
    match head.as_str() {
        "G" => "G",
        "PG" => "PG",
        "PG-13" => "PG-13",
        "R" => "R",
        "R+" | "RX" => "NC-17",
        other => return other.to_string(),
    }
    .to_string()
}

fn movie_to_tv(cap: &str) -> &'static str {
    match cap {
        "G" => "TV-G",
        "PG" => "TV-PG",
        "PG-13" => "TV-14",
        _ => "TV-MA",
    }
}

impl AgeGate {
    pub fn movie_cap(&self) -> Option<&str> {
        let cap = self.cap.as_deref()?.trim();
        MOVIE
            .iter()
            .copied()
            .find(|rating| rating.eq_ignore_ascii_case(cap))
    }

    pub fn allows(&self, certification: &str, is_series: bool) -> bool {
        let Some(cap) = self.cap.as_deref().map(|c| c.trim().to_uppercase()) else {
            return true;
        };
        if cap.is_empty() || cap == "NONE" {
            return true;
        }
        if is_unrated(certification) {
            return self.allow_unrated;
        }
        let cert = certification.trim().to_uppercase();
        let (ladder, cap) = if is_series {
            let cap = if TV.contains(&cap.as_str()) {
                cap
            } else {
                movie_to_tv(&cap).to_string()
            };
            (&TV[..], cap)
        } else {
            (&MOVIE[..], cap)
        };
        match (
            ladder.iter().position(|c| *c == cert),
            ladder.iter().position(|c| *c == cap),
        ) {
            (Some(rating), Some(limit)) => rating <= limit,
            (None, _) => self.allow_unrated,
            (_, None) => true,
        }
    }

    pub fn allows_catalog_row(&self) -> bool {
        self.allow_unrated || self.cap.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gate(cap: &str) -> AgeGate {
        AgeGate {
            cap: Some(cap.into()),
            allow_unrated: true,
        }
    }

    #[test]
    fn movie_cap_blocks_higher_rating() {
        assert!(gate("PG-13").allows("PG", false));
        assert!(!gate("PG-13").allows("R", false));
    }

    #[test]
    fn movie_cap_converts_for_series() {
        assert!(gate("PG-13").allows("TV-14", true));
        assert!(!gate("PG-13").allows("TV-MA", true));
        assert!(!gate("G").allows("TV-PG", true));
    }

    #[test]
    fn mal_tokens_map_to_movie_ladder() {
        assert_eq!(mal_rating("R+ - Mild Nudity"), "NC-17");
        assert_eq!(mal_rating("PG-13 - Teens 13 or older"), "PG-13");
        assert!(!gate("R").allows(&mal_rating("Rx - Hentai"), false));
    }

    #[test]
    fn unrated_follows_the_flag() {
        let mut g = gate("PG");
        assert!(g.allows("Not Rated", false));
        g.allow_unrated = false;
        assert!(!g.allows("", false));
        assert!(!g.allows("N/A", true));
    }

    #[test]
    fn unknown_certification_is_treated_as_unrated() {
        let mut g = gate("PG");
        assert!(g.allows("12A", false));
        g.allow_unrated = false;
        assert!(!g.allows("12A", false));
    }
}
