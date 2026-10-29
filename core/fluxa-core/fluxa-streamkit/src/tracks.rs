use crate::types::Track;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnknownField(pub String);

enum Test {
    Set,
    Equals(Vec<String>),
    Contains(Vec<String>),
}

struct Condition {
    negate: bool,
    field: String,
    test: Test,
}

fn parse(text: &str) -> Condition {
    let t = text.trim();
    let (negate, t) = t.strip_prefix('!').map_or((false, t), |r| (true, r));
    let split = t.find(['=', '~']);
    match split {
        Some(i) => {
            let values = t[i + 1..]
                .split('|')
                .map(|v| v.trim().to_lowercase())
                .collect();
            let test = if t.as_bytes()[i] == b'=' {
                Test::Equals(values)
            } else {
                Test::Contains(values)
            };
            Condition {
                negate,
                field: t[..i].trim().to_lowercase(),
                test,
            }
        }
        None => Condition {
            negate,
            field: t.to_lowercase(),
            test: Test::Set,
        },
    }
}

fn flag(on: bool) -> (bool, Vec<String>) {
    (on, vec![on.to_string()])
}

fn text(v: &Option<String>) -> (bool, Vec<String>) {
    match v {
        Some(s) if !s.is_empty() => (true, vec![s.to_lowercase()]),
        _ => (false, Vec::new()),
    }
}

fn field(track: &Track, name: &str) -> Result<(bool, Vec<String>), UnknownField> {
    Ok(match name {
        "lang" => text(&track.lang),
        "title" => text(&track.title),
        "codec" => text(&track.codec),
        "channels" => text(&track.channels),
        "tags" | "tag" => (
            !track.tags.is_empty(),
            track.tags.iter().map(|t| t.to_lowercase()).collect(),
        ),
        "default" => flag(track.default),
        "forced" => flag(track.forced),
        "hearingimpaired" => flag(track.hearing_impaired),
        "commentary" => flag(track.commentary),
        "dub" => flag(track.dub),
        "original" => flag(track.original),
        "visualimpaired" => flag(track.visual_impaired),
        other => return Err(UnknownField(other.to_string())),
    })
}

pub fn matches(track: &Track, conditions: &[String]) -> Result<bool, UnknownField> {
    evaluate(conditions, &|name| field(track, name))
}

pub fn evaluate(
    conditions: &[String],
    get: &dyn Fn(&str) -> Result<(bool, Vec<String>), UnknownField>,
) -> Result<bool, UnknownField> {
    for c in conditions.iter().map(|c| parse(c)) {
        let (set, values) = get(&c.field)?;
        let hit = match &c.test {
            Test::Set => set,
            Test::Equals(wanted) => values.iter().any(|v| wanted.contains(v)),
            Test::Contains(wanted) => values
                .iter()
                .any(|v| wanted.iter().any(|w| v.contains(w.as_str()))),
        };
        if hit == c.negate {
            return Ok(false);
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn track(lang: &str, codec: &str, tags: &[&str]) -> Track {
        Track {
            lang: Some(lang.into()),
            codec: Some(codec.into()),
            tags: tags.iter().map(|t| t.to_string()).collect(),
            ..Default::default()
        }
    }

    fn conds(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn all_conditions_must_hold_on_the_same_track() {
        let t = track("English", "eac3", &["atmos"]);
        assert!(matches(&t, &conds(&["lang=english|french", "codec~ac3"])).unwrap());
        assert!(!matches(&t, &conds(&["lang=french", "codec~ac3"])).unwrap());
    }

    #[test]
    fn bang_negates_and_bare_field_means_set() {
        let t = track("English", "aac", &[]);
        assert!(matches(&t, &conds(&["!tags", "codec"])).unwrap());
        assert!(!matches(&t, &conds(&["!codec"])).unwrap());
    }

    #[test]
    fn list_fields_match_on_any_entry() {
        let t = track("English", "truehd", &["atmos", "lossless"]);
        assert!(matches(&t, &conds(&["tags=lossless"])).unwrap());
    }

    #[test]
    fn boolean_fields_are_set_only_when_true() {
        let t = Track {
            forced: true,
            ..Default::default()
        };
        assert!(matches(&t, &conds(&["forced", "!default"])).unwrap());
    }

    #[test]
    fn unknown_fields_are_reported() {
        assert_eq!(
            matches(&Track::default(), &conds(&["nope"])),
            Err(UnknownField("nope".into()))
        );
    }
}
