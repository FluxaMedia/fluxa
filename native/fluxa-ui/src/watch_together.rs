use super::*;

const CHAT_LINES: usize = 8;

#[derive(Clone, Debug, Default)]
pub struct WatchMemberView {
    pub id: String,
    pub name: String,
    pub avatar: Option<String>,
    pub ready: bool,
    pub buffering: bool,
    pub status: String,
    pub host: bool,
    pub you: bool,
}

#[derive(Clone, Debug, Default)]
pub struct WatchChatView {
    pub id: u64,
    pub by: String,
    pub name: String,
    pub avatar: Option<String>,
    pub text: String,
    pub you: bool,
}

#[derive(Clone, Debug, Default)]
pub struct WatchTicket {
    pub room: Option<String>,
    pub connecting: bool,
    pub error: Option<String>,
    pub title: Option<String>,
    pub logo: Option<String>,
    pub backdrop: Option<String>,
    pub episode: Option<String>,
    pub host: Option<WatchMemberView>,
    pub members: Vec<WatchMemberView>,
    pub max_members: u32,
}

#[derive(Clone, Debug, Default)]
pub struct WatchTogetherModel {
    pub language: String,
    pub code_input: String,
    pub room_code: Option<String>,
    pub members: Vec<WatchMemberView>,
    pub chat: Vec<WatchChatView>,
    pub host: bool,
    pub locked: bool,
    pub share_stream: bool,
    pub max_members: u32,
    pub connecting: bool,
    pub error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WatchTogetherRequest {
    Create,
    Join(String),
    Leave,
    End,
    Close,
    CopyCode,
    Kick(String),
    SetLocked(bool),
    SetShareStream(bool),
    Chat(String),
}

fn status_text(member: &WatchMemberView, t: &dyn Fn(&str) -> String) -> String {
    t(&format!("watch.status.{}", member.status))
}

pub fn waiting_member(members: &[WatchMemberView]) -> Option<&WatchMemberView> {
    members
        .iter()
        .find(|member| member.buffering && !member.you)
}

pub fn watch_data(model: &WatchTogetherModel) -> serde_json::Value {
    let language = model.language.as_str();
    let t = |key: &str| localized(key, language);
    let ready = model.members.iter().filter(|member| member.ready).count();
    let summary = match waiting_member(&model.members) {
        Some(member) => t("watch.waiting_for").replace("{name}", &member.name),
        None if model.members.len() < 2 => t("watch.hint_alone"),
        None => t("watch.ready_count")
            .replace("{ready}", &ready.to_string())
            .replace("{total}", &model.members.len().to_string()),
    };
    let members: Vec<_> = model
        .members
        .iter()
        .enumerate()
        .map(|(index, member)| {
            serde_json::json!({
                "index": index,
                "name": member.name,
                "avatar": member.avatar,
                "statusLabel": status_text(member, &t),
                "kickable": model.host && !member.you,
            })
        })
        .collect();
    let chat: Vec<_> = model
        .chat
        .iter()
        .rev()
        .take(CHAT_LINES)
        .rev()
        .map(|line| serde_json::json!({"name": line.name, "text": line.text, "you": line.you}))
        .collect();
    serde_json::json!({
        "inRoom": model.room_code.is_some(),
        "code": model.room_code,
        "connecting": model.connecting,
        "error": model.error.as_deref().map(t),
        "host": model.host,
        "locked": model.locked,
        "shareStream": model.share_stream,
        "lockedDesc": t(if model.locked { "watch.host_only_desc" } else { "watch.guest_controls_desc" }),
        "shareDesc": t(if model.share_stream { "watch.share_stream_desc" } else { "watch.own_stream_desc" }),
        "codeHint": t("watch.code_hint"),
        "chatHint": t("watch.chat.hint"),
        "summary": summary,
        "members": members,
        "chat": chat,
    })
}
