pub(crate) fn is_newer(remote: &str, current: &str) -> bool {
    let remote_parts = version_parts(remote);
    let current_parts = version_parts(current);
    let length = remote_parts.len().max(current_parts.len());
    (0..length)
        .map(|index| {
            (
                remote_parts.get(index).copied().unwrap_or(0),
                current_parts.get(index).copied().unwrap_or(0),
            )
        })
        .find(|(remote_part, current_part)| remote_part != current_part)
        .is_some_and(|(remote_part, current_part)| remote_part > current_part)
}

fn version_parts(value: &str) -> Vec<i64> {
    value
        .trim()
        .trim_start_matches(['v', 'V'])
        .split('.')
        .map(|part| part.parse::<i64>().unwrap_or(0))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::is_newer;

    #[test]
    fn compares_numeric_version_segments_and_missing_segments() {
        assert!(is_newer("1.0.1", "1.0.0"));
        assert!(is_newer("v2.0", "1.9.9"));
        assert!(!is_newer("1.0", "1.0.0"));
        assert!(!is_newer("1.0.0", "1.0.1"));
    }
}
