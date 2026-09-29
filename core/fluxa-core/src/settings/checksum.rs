pub(crate) fn sha256_verification_status(expected: Option<&str>, actual: &str) -> &'static str {
    let Some(expected) = expected.map(str::trim).filter(|value| !value.is_empty()) else {
        return "missing";
    };
    if expected.len() != 64 || !expected.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return "invalid";
    }
    if expected.eq_ignore_ascii_case(actual.trim()) {
        "ok"
    } else {
        "mismatch"
    }
}

#[cfg(test)]
mod tests {
    use super::sha256_verification_status;

    const HASH: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    #[test]
    fn classifies_checksum_input_and_match() {
        assert_eq!(sha256_verification_status(None, HASH), "missing");
        assert_eq!(sha256_verification_status(Some("nope"), HASH), "invalid");
        assert_eq!(
            sha256_verification_status(Some(&HASH.to_uppercase()), HASH),
            "ok"
        );
        assert_eq!(sha256_verification_status(Some("f"), HASH), "invalid");
        assert_eq!(
            sha256_verification_status(Some(&"f".repeat(64)), HASH),
            "mismatch"
        );
    }
}
