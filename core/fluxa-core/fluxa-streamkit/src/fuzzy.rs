use unicode_normalization::UnicodeNormalization;

pub fn normalise_title(text: &str) -> String {
    let mut folded = String::with_capacity(text.len());
    for c in text.nfd().filter(|c| !is_combining(*c)) {
        match c {
            'ß' => folded.push_str("ss"),
            'ø' | 'Ø' => folded.push('o'),
            'æ' | 'Æ' => folded.push_str("ae"),
            'œ' | 'Œ' => folded.push_str("oe"),
            'đ' | 'Đ' => folded.push('d'),
            'ł' | 'Ł' => folded.push('l'),
            'þ' | 'Þ' => folded.push_str("th"),
            '&' => folded.push_str("and"),
            c if c.is_alphanumeric() || c == '+' => folded.extend(c.to_lowercase()),
            _ => {}
        }
    }
    let mut out = String::with_capacity(folded.len());
    let chars: Vec<char> = folded.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let pair = chars.get(i..i + 2);
        let collapsed = match pair {
            Some(['a', 'e']) | Some(['a', 'a']) => Some('a'),
            Some(['o', 'e']) => Some('o'),
            Some(['u', 'e']) => Some('u'),
            _ => None,
        };
        match collapsed {
            Some(c) => {
                out.push(c);
                i += 2;
            }
            None => {
                out.extend(chars.get(i));
                i += 1;
            }
        }
    }
    out
}

fn is_combining(c: char) -> bool {
    ('\u{0300}'..='\u{036F}').contains(&c)
}

fn lcs_len(a: &[char], b: &[char]) -> usize {
    if a.is_empty() || b.is_empty() {
        return 0;
    }
    let mut prev = vec![0usize; b.len() + 1];
    let mut cur = vec![0usize; b.len() + 1];
    for &ca in a {
        for (j, &cb) in b.iter().enumerate() {
            cur[j + 1] = if ca == cb {
                prev[j] + 1
            } else {
                prev[j + 1].max(cur[j])
            };
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

pub fn ratio(a: &str, b: &str) -> f64 {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let total = a.len() + b.len();
    if total == 0 {
        return 100.0;
    }
    200.0 * lcs_len(&a, &b) as f64 / total as f64
}

pub fn partial_ratio(a: &str, b: &str) -> f64 {
    let (short, long) = if a.chars().count() <= b.chars().count() {
        (a, b)
    } else {
        (b, a)
    };
    let s: Vec<char> = short.chars().collect();
    let l: Vec<char> = long.chars().collect();
    if s.is_empty() {
        return if l.is_empty() { 100.0 } else { 0.0 };
    }
    if s.len() == l.len() {
        return ratio(short, long);
    }
    let mut best = 0.0f64;
    for start in 0..=(l.len() - s.len()) {
        let window: String = l[start..start + s.len()].iter().collect();
        best = best.max(ratio(short, &window));
        if best >= 100.0 {
            break;
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diacritics_and_ampersand_fold() {
        assert_eq!(
            normalise_title("Amélie & Søren"),
            normalise_title("Amelie and Soren")
        );
    }

    #[test]
    fn identical_strings_score_full() {
        assert_eq!(ratio("dune", "dune"), 100.0);
    }

    #[test]
    fn partial_finds_substring() {
        assert_eq!(partial_ratio("dune", "dunepart2"), 100.0);
    }
}
