use super::*;

const ACCOUNT_KEY: &str = "fluxa_account";
const ANONYMOUS_KEY: &str = "watch_anonymous";
const SYNC_SUFFIX: &str = "/functions/v1/fluxa-sync";

#[derive(Clone)]
pub(super) struct Supabase {
    pub base: String,
    pub sync: String,
    pub key: &'static str,
}

impl Supabase {
    pub fn configured() -> Result<Self, String> {
        let (Some(sync), Some(key)) =
            (option_env!("FLUXA_SYNC_URL"), option_env!("FLUXA_SYNC_KEY"))
        else {
            return Err("Fluxa Sync is not configured in this build".to_owned());
        };
        let sync = sync.trim_end_matches('/').to_owned();
        let base = sync.strip_suffix(SYNC_SUFFIX).unwrap_or(&sync).to_owned();
        Ok(Self { base, sync, key })
    }
}

pub(super) fn client() -> Result<Client, String> {
    Client::builder()
        .bundled_roots()
        .user_agent("Fluxa/1.0")
        .native_timeout(Duration::from_secs(20))
        .build()
        .map_err(|error| error.to_string())
}

fn read_error(body: &Value, status: reqwest::StatusCode) -> String {
    ["error", "msg", "message", "error_description"]
        .iter()
        .find_map(|key| body.get(key).and_then(Value::as_str))
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("request failed with status {status}"))
}

fn session_record(body: &Value) -> Option<Value> {
    let access = body.get("access_token").and_then(Value::as_str)?;
    let refresh = body.get("refresh_token").and_then(Value::as_str)?;
    let expires_in = body
        .get("expires_in")
        .and_then(Value::as_i64)
        .unwrap_or(3600);
    let user = body.get("user").cloned().unwrap_or(Value::Null);
    Some(json!({
        "accessToken": access,
        "refreshToken": refresh,
        "expiresAt": chrono_unix_seconds() + expires_in,
        "userId": user.get("id"),
        "email": user.get("email"),
    }))
}

pub(super) struct Credentials {
    pub token: String,
}

impl EffectExecutor {
    async fn sync_auth(&self, path: &str, body: Value) -> Result<Value, String> {
        let config = Supabase::configured()?;
        let response = client()?
            .post(format!("{}/{path}", config.sync))
            .header("apikey", config.key)
            .json(&body)
            .send()
            .await
            .map_err(|error| error.to_string())?;
        let status = response.status();
        let value = response.json::<Value>().await.unwrap_or(Value::Null);
        if status.is_success() {
            Ok(value)
        } else {
            Err(read_error(&value, status))
        }
    }

    pub async fn fluxa_sign_in(&self, email: &str, password: &str) -> Result<Value, String> {
        let body = self
            .sync_auth("auth/login", json!({"email": email, "password": password}))
            .await?;
        self.store_fluxa_session(&body)
    }

    pub async fn fluxa_sign_up(&self, email: &str, password: &str) -> Result<Value, String> {
        let body = self
            .sync_auth(
                "auth/register",
                json!({"email": email, "password": password}),
            )
            .await?;
        if body.get("access_token").and_then(Value::as_str).is_none() {
            return Ok(json!({"confirmationRequired": true}));
        }
        self.store_fluxa_session(&body)
    }

    pub async fn fluxa_sign_out(&self) -> Result<(), String> {
        let Some(account) = self
            .storage
            .read_json(ACCOUNT_KEY)?
            .filter(Value::is_object)
        else {
            return Ok(());
        };
        self.storage.write_json(ACCOUNT_KEY, &Value::Null)?;
        let (Ok(config), Some(token)) = (
            Supabase::configured(),
            account.get("accessToken").and_then(Value::as_str),
        ) else {
            return Ok(());
        };
        let _ = client()?
            .post(format!("{}/auth/logout", config.sync))
            .header("apikey", config.key)
            .bearer_auth(token)
            .json(&json!({}))
            .send()
            .await;
        Ok(())
    }

    pub fn fluxa_account(&self) -> Option<Value> {
        let account = self.storage.read_json(ACCOUNT_KEY).ok().flatten()?;
        account
            .is_object()
            .then(|| json!({"userId": account.get("userId"), "email": account.get("email")}))
    }

    pub async fn fluxa_profiles(&self) -> Result<Value, String> {
        let config = Supabase::configured()?;
        let token = self
            .fluxa_token()
            .await?
            .ok_or_else(|| "not signed in to Fluxa".to_owned())?;
        let response = client()?
            .get(format!("{}/profiles", config.sync))
            .header("apikey", config.key)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|error| error.to_string())?;
        let status = response.status();
        let value = response.json::<Value>().await.unwrap_or(Value::Null);
        if status.is_success() {
            Ok(value)
        } else {
            Err(read_error(&value, status))
        }
    }

    fn store_fluxa_session(&self, body: &Value) -> Result<Value, String> {
        let record = session_record(body).ok_or_else(|| "login response was invalid".to_owned())?;
        self.storage.write_json(ACCOUNT_KEY, &record)?;
        Ok(json!({"userId": record["userId"], "email": record["email"]}))
    }

    async fn refreshed(&self, key: &str) -> Result<Option<Value>, String> {
        let Some(record) = self.storage.read_json(key)?.filter(Value::is_object) else {
            return Ok(None);
        };
        let expires_at = record["expiresAt"].as_i64().unwrap_or_default();
        if expires_at > chrono_unix_seconds() + 120 {
            return Ok(Some(record));
        }
        let config = Supabase::configured()?;
        let Some(refresh) = record["refreshToken"].as_str() else {
            return Ok(None);
        };
        let response = client()?
            .post(format!(
                "{}/auth/v1/token?grant_type=refresh_token",
                config.base
            ))
            .header("apikey", config.key)
            .json(&json!({"refresh_token": refresh}))
            .send()
            .await
            .map_err(|error| error.to_string())?;
        let status = response.status();
        let body = response.json::<Value>().await.unwrap_or(Value::Null);
        if !status.is_success() {
            if status.is_client_error() {
                self.storage.write_json(key, &Value::Null)?;
                return Ok(None);
            }
            return Err(read_error(&body, status));
        }
        let record =
            session_record(&body).ok_or_else(|| "refresh response was invalid".to_owned())?;
        self.storage.write_json(key, &record)?;
        Ok(Some(record))
    }

    async fn fluxa_token(&self) -> Result<Option<String>, String> {
        Ok(self
            .refreshed(ACCOUNT_KEY)
            .await?
            .and_then(|record| record["accessToken"].as_str().map(ToOwned::to_owned)))
    }

    pub(super) async fn watch_credentials(&self) -> Result<Credentials, String> {
        if let Some(record) = self.refreshed(ACCOUNT_KEY).await? {
            return Ok(credentials(&record));
        }
        if let Some(record) = self.refreshed(ANONYMOUS_KEY).await? {
            return Ok(credentials(&record));
        }
        let config = Supabase::configured()?;
        let response = client()?
            .post(format!("{}/auth/v1/signup", config.base))
            .header("apikey", config.key)
            .json(&json!({}))
            .send()
            .await
            .map_err(|error| error.to_string())?;
        let status = response.status();
        let body = response.json::<Value>().await.unwrap_or(Value::Null);
        if !status.is_success() {
            return Err(read_error(&body, status));
        }
        let record =
            session_record(&body).ok_or_else(|| "sign-in response was invalid".to_owned())?;
        self.storage.write_json(ANONYMOUS_KEY, &record)?;
        Ok(credentials(&record))
    }
}

fn credentials(record: &Value) -> Credentials {
    Credentials {
        token: record["accessToken"]
            .as_str()
            .unwrap_or_default()
            .to_owned(),
    }
}
