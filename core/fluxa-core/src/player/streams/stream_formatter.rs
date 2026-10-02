use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::LazyLock;

use super::stream_badges::compile;

pub const MAX_RULES: usize = 20;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Rule {
    pub id: String,
    pub pattern: String,
    pub replacement: String,
    #[serde(default = "enabled")]
    pub is_enabled: bool,
}

fn enabled() -> bool {
    true
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Formatter {
    pub rules: Vec<Rule>,
    pub presets: Vec<String>,
}

pub struct Preset {
    pub id: &'static str,
    pub rules: &'static [(&'static str, &'static str)],
}

const MEDIA_EXTENSION: &str = r"\.(mkv|mp4|avi|ts)\b";
const TECH_TAGS: &str = r"\b(x26[45]|h[ .]?26[45]|hevc|avc|av1|aac|ac-?3|e-?ac-?3|dts(-hd)?([ .-]?ma)?|true-?hd|atmos|10[ .-]?bit|hdr10\+?|hdr|dovi|dv)\b";
const QUALITY_TAGS: &str = r"\b(2160p|4k|uhd|1440p|1080[pi]|720p|480p|576p|web[ .-]?dl|web[ .-]?rip|blu-?ray|bdrip|brrip|remux|hdtv|dvdrip)\b";

pub const PRESETS: &[Preset] = &[
    Preset {
        id: "clean",
        rules: &[
            (r"(?m)^\s*\[[^\]\n]*\]\s*", ""),
            (r"\[[0-9a-f]{8}\]", ""),
            (MEDIA_EXTENSION, ""),
        ],
    },
    Preset {
        id: "emoji",
        rules: &[
            (r"\bmulti[ .-]?subs?\b", "🌐"),
            (r"\bdual[ .-]?audio\b", "🔊"),
            (r"\bbatch\b", "📦"),
            (r"\buncensored\b", "🔞"),
        ],
    },
    Preset {
        id: "flags",
        rules: &[
            (r"\b(english|eng)\b", "🇬🇧"),
            (r"\b(turkish|tur)\b", "🇹🇷"),
            (r"\b(german|ger)\b", "🇩🇪"),
            (r"\b(french|fre)\b", "🇫🇷"),
            (r"\b(spanish|spa)\b", "🇪🇸"),
            (r"\b(italian|ita)\b", "🇮🇹"),
            (r"\b(portuguese|por)\b", "🇵🇹"),
            (r"\b(russian|rus)\b", "🇷🇺"),
            (r"\b(japanese|jpn)\b", "🇯🇵"),
        ],
    },
    Preset {
        id: "tags",
        rules: &[(r"\b(multi[ .-]?subs?|dual[ .-]?audio|batch|uncensored|proper|repack)\b", "")],
    },
    Preset {
        id: "tech",
        rules: &[(TECH_TAGS, "")],
    },
    Preset {
        id: "quality",
        rules: &[(QUALITY_TAGS, "")],
    },
];

static EMPTY_BRACKETS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[\s*\]|\(\s*\)|\{\s*\}").unwrap());
static SPACES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[ \t]{2,}").unwrap());

fn usable(pattern: &str) -> Option<Regex> {
    compile(pattern).filter(|regex| !regex.is_match(""))
}

impl Formatter {
    pub fn from_value(value: &Value) -> Self {
        serde_json::from_value(value.clone()).unwrap_or_default()
    }

    pub fn upsert(&mut self, mut rule: Rule) -> bool {
        rule.pattern = rule.pattern.trim().to_owned();
        if usable(&rule.pattern).is_none() {
            return false;
        }
        if rule.id.is_empty() {
            if self.rules.len() >= MAX_RULES {
                return false;
            }
            let mut n = self.rules.len();
            while self.rules.iter().any(|existing| existing.id == format!("fmt-{n}")) {
                n += 1;
            }
            rule.id = format!("fmt-{n}");
        }
        match self.rules.iter_mut().find(|existing| existing.id == rule.id) {
            Some(existing) => *existing = rule,
            None => self.rules.push(rule),
        }
        true
    }

    pub fn toggle_preset(&mut self, id: &str) {
        if !PRESETS.iter().any(|preset| preset.id == id) {
            return;
        }
        match self.presets.iter().position(|current| current == id) {
            Some(index) => {
                self.presets.remove(index);
            }
            None => self.presets.push(id.to_owned()),
        }
    }

    pub fn remove(&mut self, id: &str) {
        self.rules.retain(|rule| rule.id != id);
    }

    pub fn toggle(&mut self, id: &str) {
        if let Some(rule) = self.rules.iter_mut().find(|rule| rule.id == id) {
            rule.is_enabled = !rule.is_enabled;
        }
    }

    pub fn apply(&self, text: &str) -> String {
        let mut text = text.to_owned();
        let mut changed = false;
        let presets = PRESETS
            .iter()
            .filter(|preset| self.presets.iter().any(|id| id == preset.id))
            .flat_map(|preset| preset.rules.iter().copied());
        let custom = self
            .rules
            .iter()
            .filter(|rule| rule.is_enabled)
            .map(|rule| (rule.pattern.as_str(), rule.replacement.as_str()));
        for (pattern, replacement) in presets.chain(custom) {
            let Some(regex) = usable(pattern) else {
                continue;
            };
            let replaced = regex.replace_all(&text, replacement);
            if replaced != text {
                text = replaced.into_owned();
                changed = true;
            }
        }
        if !changed {
            return text;
        }
        text.lines()
            .map(|line| {
                let line = EMPTY_BRACKETS.replace_all(line, "");
                SPACES.replace_all(&line, " ").trim().to_owned()
            })
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(pattern: &str, replacement: &str) -> Rule {
        Rule {
            pattern: pattern.to_owned(),
            replacement: replacement.to_owned(),
            is_enabled: true,
            ..Default::default()
        }
    }

    fn formatter(rules: &[(&str, &str)]) -> Formatter {
        let mut formatter = Formatter::default();
        for (pattern, replacement) in rules {
            assert!(formatter.upsert(rule(pattern, replacement)));
        }
        formatter
    }

    #[test]
    fn removed_keyword_leaves_no_empty_brackets() {
        let formatter = formatter(&[("multisub", ""), (r"\bbatch\b", "")]);
        assert_eq!(
            formatter.apply("[Erai-raws] Bleach - 001 [1080p][MultiSub] [BATCH]"),
            "[Erai-raws] Bleach - 001 [1080p]"
        );
    }

    #[test]
    fn keyword_becomes_emoji_and_captures_work() {
        let formatter = formatter(&[(r"multi ?subs?", "🌐"), (r"(\d+)p\b", "${1}P")]);
        assert_eq!(formatter.apply("Multi Subs 1080p"), "🌐 1080P");
    }

    #[test]
    fn disabled_rules_and_empty_matching_patterns_are_ignored() {
        let mut formatter = formatter(&[("batch", "")]);
        formatter.toggle("fmt-0");
        assert_eq!(formatter.apply("a [batch]"), "a [batch]");
        assert!(!formatter.upsert(rule("x*", "")));
        assert!(!formatter.upsert(rule("(", "")));
    }

    #[test]
    fn presets_clean_an_anime_release() {
        let mut formatter = Formatter::default();
        for id in ["clean", "emoji", "tech", "quality"] {
            formatter.toggle_preset(id);
        }
        let text = "[Erai-raws] Bleach - 001 [1080p DSNP WEB-DL AVC AAC][MultiSub][EF0AF7BA].mkv";
        assert_eq!(formatter.apply(text), "Bleach - 001 [ DSNP ][🌐]");
    }

    #[test]
    fn every_preset_pattern_compiles() {
        for preset in PRESETS {
            for (pattern, _) in preset.rules {
                assert!(usable(pattern).is_some(), "{pattern}");
            }
        }
    }

    #[test]
    fn untouched_text_keeps_its_layout() {
        let formatter = formatter(&[("zzz", "")]);
        assert_eq!(formatter.apply("a  b\n\nc"), "a  b\n\nc");
    }
}
