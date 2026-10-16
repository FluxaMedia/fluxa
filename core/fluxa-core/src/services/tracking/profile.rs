use serde_json::{Value, json};

fn text(body: &Value, pointer: &str) -> Option<String> {
    body.pointer(pointer)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn first(body: &Value, pointers: &[&str]) -> Option<String> {
    pointers.iter().find_map(|pointer| text(body, pointer))
}

pub(crate) fn provider_profile_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    if let Some(status) = args.get("status").and_then(Value::as_u64)
        && !(200..300).contains(&status)
    {
        return None;
    }
    let body = args.get("body")?;
    let (names, avatars): (&[&str], &[&str]) = match args.get("provider")?.as_str()? {
        "trakt" => (
            &["/user/username", "/user/name"],
            &["/user/images/avatar/full"],
        ),
        "simkl" => (&["/user/name"], &["/user/avatar"]),
        "anilist" => (&["/data/Viewer/name"], &["/data/Viewer/avatar/large"]),
        "mdblist" => (&["/username"], &[]),
        _ => return None,
    };
    let name = first(body, names)?;
    serde_json::to_string(&json!({"name": name, "avatarUrl": first(body, avatars)})).ok()
}
