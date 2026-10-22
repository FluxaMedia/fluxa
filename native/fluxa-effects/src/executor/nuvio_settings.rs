use super::*;

const SETTINGS_PLATFORM: &str = "fluxa";

pub struct NuvioSettingsPull {
    pub prefs: Option<Value>,
    pub credentials: Vec<Value>,
}

impl EffectExecutor {
    pub async fn pull_nuvio_settings(&self) -> Result<Option<NuvioSettingsPull>, String> {
        let (_, profile, _) = self.active_profile()?;
        let Some(session) = self.nuvio_session(&profile).await? else {
            return Ok(None);
        };
        let profile_id = session.profile_index;
        let blob = rpc_rows(
            &session,
            "sync_pull_profile_settings_blob",
            json!({"p_profile_id": profile_id, "p_platform": SETTINGS_PLATFORM}),
        )
        .await?;
        let credentials = rpc_rows(
            &session,
            "sync_pull_provider_credentials",
            json!({"p_profile_id": profile_id}),
        )
        .await?;
        let prefs = blob
            .first()
            .and_then(|row| row.pointer("/settings_json/prefs"))
            .filter(|prefs| prefs.is_object())
            .cloned();
        Ok(Some(NuvioSettingsPull { prefs, credentials }))
    }

    pub async fn push_nuvio_settings(&self, blob: Value) -> Result<(), String> {
        let (_, profile, _) = self.active_profile()?;
        let Some(session) = self.nuvio_session(&profile).await? else {
            return Ok(());
        };
        rpc_ok(
            &session,
            "sync_push_profile_settings_blob",
            json!({
                "p_profile_id": session.profile_index,
                "p_platform": SETTINGS_PLATFORM,
                "p_settings_json": blob,
            }),
        )
        .await
    }

    pub async fn push_nuvio_credentials(&self, rows: Vec<Value>) -> Result<(), String> {
        let (_, profile, _) = self.active_profile()?;
        let Some(session) = self.nuvio_session(&profile).await? else {
            return Ok(());
        };
        rpc_ok(
            &session,
            "sync_push_provider_credentials",
            json!({"p_profile_id": session.profile_index, "p_credentials": rows}),
        )
        .await
    }
}

async fn rpc_rows(
    session: &account::NuvioSession,
    name: &str,
    body: Value,
) -> Result<Vec<Value>, String> {
    let rows = session
        .rpc(name)
        .json(&body)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|error| format!("{name} failed: {error}"))?
        .json::<Value>()
        .await
        .map_err(|error| format!("{name} response was invalid: {error}"))?;
    Ok(rows.as_array().cloned().unwrap_or_default())
}

async fn rpc_ok(session: &account::NuvioSession, name: &str, body: Value) -> Result<(), String> {
    let response = session
        .rpc(name)
        .json(&body)
        .send()
        .await
        .map_err(|error| format!("{name} failed: {error}"))?;
    let status = response.status();
    if status.is_success() {
        return Ok(());
    }
    let body = response.text().await.unwrap_or_default();
    Err(format!("Nuvio rejected {name} ({status}): {body}"))
}
