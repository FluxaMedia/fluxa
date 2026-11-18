#[derive(Clone)]
pub struct ActionMenuItem {
    pub icon: &'static str,
    pub label: String,
    pub app_icon: Option<&'static str>,
}
