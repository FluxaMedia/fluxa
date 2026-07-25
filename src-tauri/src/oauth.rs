use fluxa_core::FluxaCore;
use serde_json::json;

const TRAKT_CLIENT_ID: &str = env!("FLUXA_TRAKT_CLIENT_ID");
const TRAKT_CLIENT_SECRET: &str = env!("FLUXA_TRAKT_CLIENT_SECRET");
const ANILIST_CLIENT_ID: &str = match option_env!("FLUXA_ANILIST_CLIENT_ID") {
    Some(value) => value,
    None => "",
};
const ANILIST_CLIENT_SECRET: &str = match option_env!("FLUXA_ANILIST_CLIENT_SECRET") {
    Some(value) => value,
    None => "",
};
pub const SIMKL_CLIENT_ID: &str = env!("FLUXA_SIMKL_CLIENT_ID");
const SIMKL_CLIENT_SECRET: &str = env!("FLUXA_SIMKL_CLIENT_SECRET");
const NUVIO_SUPABASE_URL: &str = env!("FLUXA_NUVIO_SUPABASE_URL");
const NUVIO_SUPABASE_KEY: &str = env!("FLUXA_NUVIO_SUPABASE_KEY");

// Proxies Nuvio/Supabase REST calls so the anon key stays on the Rust side --
// JS only ever sees response bodies, never the key itself.
#[tauri::command]
pub async fn nuvio_request(
    method: String,
    path: String,
    body: Option<String>,
    token: Option<String>,
) -> Result<(u16, String), String> {
    if !path.starts_with('/') || path.starts_with("//") || path.starts_with("/\\") {
        return Err("invalid path".to_string());
    }
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;
    let url = format!("{}{}", NUVIO_SUPABASE_URL.trim_end_matches('/'), path);
    let mut req = match method.as_str() {
        "GET" => client.get(&url),
        "POST" => client.post(&url),
        "PATCH" => client.patch(&url),
        "DELETE" => client.delete(&url),
        other => return Err(format!("unsupported method: {other}")),
    };
    req = req
        .header("apikey", NUVIO_SUPABASE_KEY)
        .header("Content-Type", "application/json");
    if let Some(token) = &token {
        req = req.header("Authorization", format!("Bearer {token}"));
    }
    req = match &body {
        Some(body) => req.body(body.clone()),
        None if method == "POST" => req.body("{}"),
        None => req,
    };
    let res = req.send().await.map_err(describe_reqwest_error)?;
    let status = res.status().as_u16();
    let text = res.text().await.map_err(describe_reqwest_error)?;
    Ok((status, text))
}

fn describe_reqwest_error(e: reqwest::Error) -> String {
    let mut message = e.to_string();
    let mut source = std::error::Error::source(&e);
    while let Some(err) = source {
        message.push_str(": ");
        message.push_str(&err.to_string());
        source = err.source();
    }
    message
}

fn oauth_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .user_agent("Fluxa Desktop/1.0")
        .build()
        .map_err(|e| e.to_string())
}

async fn execute_oauth_request(
    service: &str,
    operation: &str,
    code: Option<&str>,
    refresh_token: Option<&str>,
) -> Result<(u16, String), String> {
    let (client_id, client_secret) = match service {
        "trakt" => (TRAKT_CLIENT_ID, TRAKT_CLIENT_SECRET),
        "anilist" => (ANILIST_CLIENT_ID, ANILIST_CLIENT_SECRET),
        "simkl" => (SIMKL_CLIENT_ID, SIMKL_CLIENT_SECRET),
        _ => return Err("unsupported OAuth service".to_string()),
    };
    if client_id.is_empty()
        || ((operation == "exchange" || operation == "refresh") && client_secret.is_empty())
    {
        return Err(format!("{service} OAuth client is not configured"));
    }
    let request = json!({"service": service, "operation": operation, "clientId": client_id, "clientSecret": client_secret, "code": code, "refreshToken": refresh_token});
    let plan_json = FluxaCore::oauth_request_plan_json(&request.to_string())
        .ok_or_else(|| "invalid OAuth request".to_string())?;
    let plan: serde_json::Value =
        serde_json::from_str(&plan_json).map_err(|error| error.to_string())?;
    let response = oauth_client()?
        .post(
            plan.get("url")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| "OAuth request has no URL".to_string())?,
        )
        .header("Content-Type", "application/json")
        .json(plan.get("body").unwrap_or(&serde_json::Value::Null))
        .send()
        .await
        .map_err(describe_reqwest_error)?;
    let status = response.status().as_u16();
    let text = response.text().await.map_err(describe_reqwest_error)?;
    Ok((status, text))
}

#[tauri::command]
pub fn get_oauth_client_id(service: &str) -> &'static str {
    match service {
        "trakt" => TRAKT_CLIENT_ID,
        "anilist" => ANILIST_CLIENT_ID,
        "simkl" => SIMKL_CLIENT_ID,
        _ => "",
    }
}

#[tauri::command]
pub async fn trakt_device_start() -> Result<String, String> {
    let (status, text) = execute_oauth_request("trakt", "device_start", None, None).await?;
    (FluxaCore::oauth_response_outcome("trakt", "device_start", status) == "success")
        .then_some(text)
        .ok_or_else(|| format!("Trakt device code request failed: HTTP {status}"))
}

#[tauri::command]
pub async fn trakt_device_poll(device_code: String) -> Result<String, String> {
    let (status, text) =
        execute_oauth_request("trakt", "device_poll", Some(&device_code), None).await?;
    match FluxaCore::oauth_response_outcome("trakt", "device_poll", status) {
        "success" => Ok(text),
        "pending" => Ok("pending".to_string()),
        _ => Err("expired".to_string()),
    }
}

#[tauri::command]
pub async fn trakt_oauth_exchange(code: String) -> Result<String, String> {
    let (status, text) = execute_oauth_request("trakt", "exchange", Some(&code), None).await?;
    (FluxaCore::oauth_response_outcome("trakt", "exchange", status) == "success")
        .then_some(text.clone())
        .ok_or_else(|| format!("Trakt token exchange failed: HTTP {status}: {text}"))
}

#[tauri::command]
pub async fn trakt_oauth_refresh(refresh_token: String) -> Result<String, String> {
    let (status, text) = execute_oauth_request("trakt", "refresh", None, Some(&refresh_token)).await?;
    (FluxaCore::oauth_response_outcome("trakt", "refresh", status) == "success")
        .then_some(text.clone())
        .ok_or_else(|| format!("Trakt token refresh failed: HTTP {status}: {text}"))
}

#[tauri::command]
pub async fn anilist_oauth_exchange(code: String) -> Result<String, String> {
    let (status, text) = execute_oauth_request("anilist", "exchange", Some(&code), None).await?;
    (FluxaCore::oauth_response_outcome("anilist", "exchange", status) == "success")
        .then_some(text.clone())
        .ok_or_else(|| format!("AniList token exchange failed: HTTP {status}: {text}"))
}

#[tauri::command]
pub async fn anilist_oauth_refresh(refresh_token: String) -> Result<String, String> {
    let (status, text) =
        execute_oauth_request("anilist", "refresh", None, Some(&refresh_token)).await?;
    (FluxaCore::oauth_response_outcome("anilist", "refresh", status) == "success")
        .then_some(text.clone())
        .ok_or_else(|| format!("AniList token refresh failed: HTTP {status}: {text}"))
}

#[tauri::command]
pub async fn simkl_oauth_exchange(code: String) -> Result<String, String> {
    let (status, text) = execute_oauth_request("simkl", "exchange", Some(&code), None).await?;
    (FluxaCore::oauth_response_outcome("simkl", "exchange", status) == "success")
        .then_some(text.clone())
        .ok_or_else(|| format!("SIMKL token exchange failed: HTTP {status}: {text}"))
}
