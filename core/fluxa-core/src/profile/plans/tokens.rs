use super::*;

pub(crate) fn token_merge_plan_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<TokenMergeRequest>(request_json).ok()?;
    let mut profile = request.profile.clone();
    let auth = &request.auth_result;
    let provider = request.provider.as_str();

    if profile.is_null() || !profile.is_object() {
        profile = json!({});
    }

    let obj = profile.as_object_mut()?;

    match AuthProvider::from(provider) {
        AuthProvider::Trakt => {
            let token = auth.get("accessToken").or_else(|| auth.get("access_token"));
            let refresh = auth
                .get("refreshToken")
                .or_else(|| auth.get("refresh_token"));
            let expires_at = auth
                .get("expiresAt")
                .or_else(|| auth.get("expires_at"))
                .or_else(|| auth.get("traktTokenExpiresAt"));
            if let Some(t) = token {
                obj.insert("traktAccessToken".to_string(), t.clone());
            }
            if let Some(r) = refresh {
                obj.insert("traktRefreshToken".to_string(), r.clone());
            }
            if let Some(e) = expires_at {
                obj.insert("traktTokenExpiresAt".to_string(), e.clone());
            }
            obj.insert("traktLastSyncAt".to_string(), Value::Null);
        }
        AuthProvider::Simkl => {
            let token = auth.get("accessToken").or_else(|| auth.get("access_token"));
            let refresh = auth
                .get("refreshToken")
                .or_else(|| auth.get("refresh_token"));
            let expires_at = auth.get("expiresAt").or_else(|| auth.get("expires_at"));
            if let Some(t) = token {
                obj.insert("simklAccessToken".to_string(), t.clone());
            }
            if let Some(r) = refresh {
                obj.insert("simklRefreshToken".to_string(), r.clone());
            }
            if let Some(e) = expires_at {
                obj.insert("simklTokenExpiresAt".to_string(), e.clone());
            }
        }
        AuthProvider::Anilist => {
            let token = auth.get("accessToken").or_else(|| auth.get("access_token"));
            let refresh = auth
                .get("refreshToken")
                .or_else(|| auth.get("refresh_token"));
            let expires_at = auth
                .get("expiresAt")
                .or_else(|| auth.get("expires_at"))
                .or_else(|| auth.get("anilistTokenExpiresAt"));
            if let Some(t) = token {
                obj.insert("anilistAccessToken".to_string(), t.clone());
            }
            if let Some(r) = refresh {
                obj.insert("anilistRefreshToken".to_string(), r.clone());
            }
            if let Some(e) = expires_at {
                obj.insert("anilistTokenExpiresAt".to_string(), e.clone());
            }
        }
        AuthProvider::Mdblist => {
            let token = auth.get("accessToken").or_else(|| auth.get("access_token"));
            let refresh = auth
                .get("refreshToken")
                .or_else(|| auth.get("refresh_token"));
            let expires_at = auth.get("expiresAt").or_else(|| auth.get("expires_at"));
            if let Some(t) = token {
                obj.insert("mdblistAccessToken".to_string(), t.clone());
            }
            if let Some(r) = refresh {
                obj.insert("mdblistRefreshToken".to_string(), r.clone());
            }
            if let Some(e) = expires_at {
                obj.insert("mdblistTokenExpiresAt".to_string(), e.clone());
            }
        }
        AuthProvider::Stremio => {
            if let Some(auth_key) = auth.get("authKey").or_else(|| auth.get("apiKey")) {
                obj.insert("authKey".to_string(), auth_key.clone());
            }
            if let Some(id) = auth.get("id") {
                obj.insert("id".to_string(), id.clone());
            }
            if let Some(email) = auth.get("email") {
                obj.insert("email".to_string(), email.clone());
            }
            obj.insert("isGuest".to_string(), json!(false));
        }
        AuthProvider::Unknown => {}
    }

    serde_json::to_string(&json!({
        "mergedProfile": profile,
        "provider": provider
    }))
    .ok()
}
