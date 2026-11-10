#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ImeState {
    pub text: String,
    pub selection: [usize; 2],
    pub composing: Option<[usize; 2]>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImeOp {
    Commit { text: String, cursor: i32 },
    Compose { text: String, cursor: i32 },
    ComposeRegion { start: usize, end: usize },
    FinishComposing,
    DeleteSurrounding { before: usize, after: usize },
    DeleteSurroundingChars { before: usize, after: usize },
    Select { start: usize, end: usize },
    SelectAll,
    Backspace,
}

pub fn utf16_len(text: &str) -> usize {
    text.chars().map(char::len_utf16).sum()
}

pub fn utf16_offset(text: &str, chars: usize) -> usize {
    text.chars().take(chars).map(char::len_utf16).sum()
}

pub fn char_offset(text: &str, units: usize) -> usize {
    let mut used = 0;
    for (index, c) in text.chars().enumerate() {
        if used >= units {
            return index;
        }
        used += c.len_utf16();
    }
    text.chars().count()
}

fn clamp_range(range: [usize; 2], len: usize) -> [usize; 2] {
    let start = range[0].min(range[1]).min(len);
    let end = range[0].max(range[1]).min(len);
    [start, end]
}

fn walk_back(chars: &[char], from: usize, units: usize) -> usize {
    let mut used = 0;
    let mut count = 0;
    while count < from && used < units {
        used += chars[from - 1 - count].len_utf16();
        count += 1;
    }
    count
}

fn walk_forward(chars: &[char], from: usize, units: usize) -> usize {
    let mut used = 0;
    let mut count = 0;
    while from + count < chars.len() && used < units {
        used += chars[from + count].len_utf16();
        count += 1;
    }
    count
}

fn shift(position: usize, deleted: [usize; 2]) -> usize {
    if position <= deleted[0] {
        position
    } else if position >= deleted[1] {
        position - (deleted[1] - deleted[0])
    } else {
        deleted[0]
    }
}

impl ImeState {
    pub fn new(text: String, selection: [usize; 2]) -> Self {
        let len = text.chars().count();
        Self {
            text,
            selection: clamp_range(selection, len),
            composing: None,
        }
    }

    pub fn apply(&mut self, op: &ImeOp, multiline: bool) {
        let mut chars: Vec<char> = self.text.chars().collect();
        let len = chars.len();
        self.selection = clamp_range(self.selection, len);
        self.composing = self
            .composing
            .map(|range| clamp_range(range, len))
            .filter(|range| range[0] < range[1]);
        match op {
            ImeOp::Commit { text, cursor } => {
                self.replace(&mut chars, text, *cursor, false, multiline)
            }
            ImeOp::Compose { text, cursor } => {
                self.replace(&mut chars, text, *cursor, true, multiline)
            }
            ImeOp::ComposeRegion { start, end } => {
                let range = clamp_range(
                    [
                        walk_forward(&chars, 0, *start),
                        walk_forward(&chars, 0, *end),
                    ],
                    len,
                );
                self.composing = (range[0] < range[1]).then_some(range);
            }
            ImeOp::FinishComposing => self.composing = None,
            ImeOp::DeleteSurrounding { before, after } => {
                let [start, end] = self.selection;
                let back = walk_back(&chars, start, *before);
                let forward = walk_forward(&chars, end, *after);
                self.delete(&mut chars, [end, end + forward]);
                self.delete(&mut chars, [start - back, start]);
            }
            ImeOp::DeleteSurroundingChars { before, after } => {
                let [start, end] = self.selection;
                let back = (*before).min(start);
                let forward = (*after).min(len - end);
                self.delete(&mut chars, [end, end + forward]);
                self.delete(&mut chars, [start - back, start]);
            }
            ImeOp::Select { start, end } => {
                self.selection = clamp_range(
                    [
                        walk_forward(&chars, 0, *start),
                        walk_forward(&chars, 0, *end),
                    ],
                    len,
                );
            }
            ImeOp::SelectAll => self.selection = [0, len],
            ImeOp::Backspace => {
                let [start, end] = self.selection;
                if start != end {
                    self.delete(&mut chars, [start, end]);
                } else if start > 0 {
                    self.delete(&mut chars, [start - 1, start]);
                }
            }
        }
        self.text = chars.into_iter().collect();
    }

    fn replace(
        &mut self,
        chars: &mut Vec<char>,
        text: &str,
        cursor: i32,
        compose: bool,
        multiline: bool,
    ) {
        let [start, end] = self.composing.unwrap_or(self.selection);
        let inserted: Vec<char> = text
            .chars()
            .filter(|c| multiline || !matches!(c, '\n' | '\r'))
            .collect();
        let stop = start + inserted.len();
        chars.splice(start..end, inserted.iter().copied());
        self.composing = (compose && !inserted.is_empty()).then_some([start, stop]);
        let at = if cursor > 0 {
            stop + (cursor as usize - 1)
        } else {
            start.saturating_sub(cursor.unsigned_abs() as usize)
        };
        let at = at.min(chars.len());
        self.selection = [at, at];
    }

    fn delete(&mut self, chars: &mut Vec<char>, range: [usize; 2]) {
        if range[0] >= range[1] {
            return;
        }
        chars.drain(range[0]..range[1]);
        self.selection = [
            shift(self.selection[0], range),
            shift(self.selection[1], range),
        ];
        self.composing = self
            .composing
            .map(|composing| [shift(composing[0], range), shift(composing[1], range)])
            .filter(|composing| composing[0] < composing[1]);
    }

    pub fn before_cursor(&self, units: usize) -> String {
        let chars: Vec<char> = self.text.chars().collect();
        let start = self.selection[0].min(chars.len());
        let back = walk_back(&chars, start, units);
        chars[start - back..start].iter().collect()
    }

    pub fn after_cursor(&self, units: usize) -> String {
        let chars: Vec<char> = self.text.chars().collect();
        let end = self.selection[1].min(chars.len());
        let forward = walk_forward(&chars, end, units);
        chars[end..end + forward].iter().collect()
    }

    pub fn selected(&self) -> String {
        let [start, end] = self.selection;
        self.text.chars().skip(start).take(end - start).collect()
    }

    pub fn selection_units(&self) -> [usize; 2] {
        [
            utf16_offset(&self.text, self.selection[0]),
            utf16_offset(&self.text, self.selection[1]),
        ]
    }

    pub fn composing_units(&self) -> Option<[usize; 2]> {
        self.composing.map(|[start, end]| {
            [
                utf16_offset(&self.text, start),
                utf16_offset(&self.text, end),
            ]
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(text: &str, at: usize) -> ImeState {
        ImeState::new(text.to_owned(), [at, at])
    }

    fn compose(text: &str) -> ImeOp {
        ImeOp::Compose {
            text: text.to_owned(),
            cursor: 1,
        }
    }

    fn commit(text: &str) -> ImeOp {
        ImeOp::Commit {
            text: text.to_owned(),
            cursor: 1,
        }
    }

    #[test]
    fn composing_text_is_replaced_by_each_update() {
        let mut ime = state("hi ", 3);
        ime.apply(&compose("w"), false);
        ime.apply(&compose("wo"), false);
        assert_eq!(ime.text, "hi wo");
        assert_eq!(ime.composing, Some([3, 5]));
        assert_eq!(ime.selection, [5, 5]);
    }

    #[test]
    fn commit_replaces_the_composing_region() {
        let mut ime = state("hi ", 3);
        ime.apply(&compose("wor"), false);
        ime.apply(&commit("world"), false);
        assert_eq!(ime.text, "hi world");
        assert_eq!(ime.composing, None);
        assert_eq!(ime.selection, [8, 8]);
    }

    #[test]
    fn commit_replaces_the_selection_without_composing() {
        let mut ime = ImeState::new("hello world".to_owned(), [6, 11]);
        ime.apply(&commit("there"), false);
        assert_eq!(ime.text, "hello there");
        assert_eq!(ime.selection, [11, 11]);
    }

    #[test]
    fn negative_cursor_lands_before_the_inserted_text() {
        let mut ime = state("ab", 2);
        ime.apply(
            &ImeOp::Commit {
                text: "cd".to_owned(),
                cursor: 0,
            },
            false,
        );
        assert_eq!(ime.text, "abcd");
        assert_eq!(ime.selection, [2, 2]);
    }

    #[test]
    fn delete_surrounding_removes_around_the_cursor() {
        let mut ime = state("abcdef", 3);
        ime.apply(
            &ImeOp::DeleteSurrounding {
                before: 2,
                after: 1,
            },
            false,
        );
        assert_eq!(ime.text, "aef");
        assert_eq!(ime.selection, [1, 1]);
    }

    #[test]
    fn offsets_count_utf16_units_for_emoji() {
        let mut ime = state("a😀b", 3);
        assert_eq!(ime.selection_units(), [4, 4]);
        ime.apply(
            &ImeOp::DeleteSurrounding {
                before: 1,
                after: 0,
            },
            false,
        );
        assert_eq!(ime.text, "a😀");
        ime.apply(
            &ImeOp::DeleteSurrounding {
                before: 2,
                after: 0,
            },
            false,
        );
        assert_eq!(ime.text, "a");
    }

    #[test]
    fn compose_region_marks_existing_text() {
        let mut ime = state("hello world", 11);
        ime.apply(&ImeOp::ComposeRegion { start: 6, end: 11 }, false);
        assert_eq!(ime.composing, Some([6, 11]));
        ime.apply(&compose("there"), false);
        assert_eq!(ime.text, "hello there");
    }

    #[test]
    fn backspace_inside_composing_shrinks_it() {
        let mut ime = state("ab", 2);
        ime.apply(&compose("cd"), false);
        ime.apply(&ImeOp::Backspace, false);
        assert_eq!(ime.text, "abc");
        assert_eq!(ime.composing, Some([2, 3]));
    }

    #[test]
    fn single_line_fields_drop_newlines() {
        let mut ime = state("", 0);
        ime.apply(&commit("a\nb"), false);
        assert_eq!(ime.text, "ab");
        ime.apply(&commit("\n"), true);
        assert_eq!(ime.text, "ab\n");
    }

    #[test]
    fn text_around_the_cursor_is_served_in_units() {
        let ime = state("a😀bc", 3);
        assert_eq!(ime.before_cursor(1), "b");
        assert_eq!(ime.before_cursor(3), "😀b");
        assert_eq!(ime.after_cursor(5), "c");
    }
}
