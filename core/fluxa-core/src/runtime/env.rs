pub trait FluxaEnv: Send + Sync + 'static {
    fn fetch_http(url: &str, timeout_ms: u32) -> Result<(u16, String), String>;
    fn get_storage(key: &str) -> Result<Option<String>, String>;
    fn set_storage(key: &str, value: Option<&str>) -> Result<(), String>;
    fn now_utc_ms() -> i64;
}
