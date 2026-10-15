const DEFAULT_DURATION_NS: i64 = 4_000_000_000;
const KEEP_BEHIND_NS: i64 = 30_000_000_000;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SubtitleFormat {
    Text,
    Ass,
    MovText,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Cue {
    pub start_ns: i64,
    pub end_ns: i64,
    pub text: String,
}

fn strip_markup(text: &str, open: char, close: char) -> String {
    let mut out = String::with_capacity(text.len());
    let mut depth = 0;
    for c in text.chars() {
        match c {
            c if c == open => depth += 1,
            c if c == close && depth > 0 => depth -= 1,
            c if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out
}

fn clean(text: &str) -> Option<String> {
    let lines: Vec<&str> = text.lines().map(str::trim).filter(|line| !line.is_empty()).collect();
    (!lines.is_empty()).then(|| lines.join("\n"))
}

pub fn extract(format: SubtitleFormat, data: &[u8]) -> Option<String> {
    match format {
        SubtitleFormat::Text => {
            let text = String::from_utf8_lossy(data);
            clean(&strip_markup(&strip_markup(&text, '<', '>'), '{', '}'))
        }
        SubtitleFormat::Ass => {
            let line = String::from_utf8_lossy(data);
            let text = line.splitn(9, ',').nth(8)?;
            let text = text.replace("\\N", "\n").replace("\\n", "\n").replace("\\h", " ");
            clean(&strip_markup(&text, '{', '}'))
        }
        SubtitleFormat::MovText => {
            let length = u16::from_be_bytes([*data.first()?, *data.get(1)?]) as usize;
            let body = data.get(2..2 + length.min(data.len().saturating_sub(2)))?;
            clean(&String::from_utf8_lossy(body))
        }
    }
}

pub fn cue(format: SubtitleFormat, data: &[u8], start_ns: i64, duration_ns: i64) -> Option<Cue> {
    let text = extract(format, data)?;
    let length = if duration_ns > 0 { duration_ns } else { DEFAULT_DURATION_NS };
    Some(Cue { start_ns, end_ns: start_ns + length, text })
}

#[derive(Default)]
pub struct Cues {
    cues: Vec<Cue>,
}

impl Cues {
    pub fn push(&mut self, cue: Cue) {
        if !self.cues.contains(&cue) {
            self.cues.push(cue);
        }
    }

    pub fn clear(&mut self) {
        self.cues.clear();
    }

    pub fn at(&mut self, position_ns: i64) -> Option<String> {
        self.cues.retain(|cue| cue.end_ns > position_ns - KEEP_BEHIND_NS);
        let active: Vec<&str> = self
            .cues
            .iter()
            .filter(|cue| cue.start_ns <= position_ns && position_ns < cue.end_ns)
            .map(|cue| cue.text.as_str())
            .collect();
        (!active.is_empty()).then(|| active.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ass_line_keeps_only_the_dialogue_text() {
        let line = b"12,0,Default,,0,0,0,,{\\an8}Hello,\\Nworld, again";
        assert_eq!(extract(SubtitleFormat::Ass, line).as_deref(), Some("Hello,\nworld, again"));
    }

    #[test]
    fn srt_markup_is_removed() {
        let text = b"<i>Hi</i> there\n\n{\\an8}you";
        assert_eq!(extract(SubtitleFormat::Text, text).as_deref(), Some("Hi there\nyou"));
    }

    #[test]
    fn mov_text_respects_its_length_prefix() {
        let mut data = vec![0, 5];
        data.extend_from_slice(b"Hello\x00\x00\x00junk");
        assert_eq!(extract(SubtitleFormat::MovText, &data).as_deref(), Some("Hello"));
    }

    #[test]
    fn cues_show_only_inside_their_window_and_dedupe() {
        let mut cues = Cues::default();
        let cue = cue(SubtitleFormat::Text, b"one", 1_000_000_000, 2_000_000_000).unwrap();
        cues.push(cue.clone());
        cues.push(cue);
        assert_eq!(cues.at(500_000_000), None);
        assert_eq!(cues.at(1_500_000_000).as_deref(), Some("one"));
        assert_eq!(cues.at(3_000_000_000), None);
    }

    #[test]
    fn zero_duration_falls_back_to_default() {
        let cue = cue(SubtitleFormat::Text, b"x", 0, 0).unwrap();
        assert_eq!(cue.end_ns, 4_000_000_000);
    }
}
