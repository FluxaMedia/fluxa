use super::*;

const KEY: &str = "tRGFlzhNNm970Bxwuyg/+tsei6QDac+TaayTsVrAm5Q=";
const SIG: &str =
    "6hkuc/QV3YCPuAOt7l66FIecTmovoWOBuqu84PuSZ54xtqls9e08E5gyAIFrpCHF4bqSKB4gcez+O0LPcwl+BQ==";
const BODY: &str = r#"{"version":"1.4.0","notes":"n","assets":{"linux-x86_64":{"url":"https://example.com/a.tar.gz","sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","size":1}}}"#;

#[test]
fn newer_signed_release_is_offered() {
    let release = check(BODY.as_bytes(), SIG, KEY, "1.3.9", "linux-x86_64")
        .unwrap()
        .unwrap();
    assert_eq!(release.version, "1.4.0");
    assert_eq!(release.asset.url, "https://example.com/a.tar.gz");
}

#[test]
fn same_or_older_release_is_ignored() {
    assert_eq!(
        check(BODY.as_bytes(), SIG, KEY, "1.4.0", "linux-x86_64"),
        Ok(None)
    );
    assert_eq!(
        check(BODY.as_bytes(), SIG, KEY, "2.0.0", "linux-x86_64"),
        Ok(None)
    );
}

#[test]
fn platform_without_asset_gets_nothing() {
    assert_eq!(
        check(BODY.as_bytes(), SIG, KEY, "1.0.0", "windows-x86_64"),
        Ok(None)
    );
}

#[test]
fn tampered_manifest_is_rejected() {
    let tampered = BODY.replace("a.tar.gz", "evil.tar.gz");
    assert_eq!(
        check(tampered.as_bytes(), SIG, KEY, "1.0.0", "linux-x86_64"),
        Err(UpdateError::BadSignature)
    );
}

#[test]
fn prerelease_sorts_below_release() {
    assert_eq!(
        compare_versions("1.0.0-beta.1", "1.0.0"),
        Ok(Ordering::Less)
    );
    assert_eq!(compare_versions("v0.10.0", "0.9.9"), Ok(Ordering::Greater));
}

#[test]
fn arch_aliases_are_normalised() {
    assert_eq!(platform_key("macos", "aarch64"), "macos-arm64");
    assert_eq!(platform_key("windows", "amd64"), "windows-x86_64");
}
