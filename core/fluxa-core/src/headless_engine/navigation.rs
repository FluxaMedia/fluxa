use super::HeadlessEngine;
use crate::runtime::EffectEnvelope;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct NavigationState {
    route: String,
    params: Value,
}

impl Default for NavigationState {
    fn default() -> Self {
        Self {
            route: "home".to_string(),
            params: Value::Null,
        }
    }
}

pub(crate) fn dispatch(
    engine: &mut HeadlessEngine,
    route: String,
    params: Option<Value>,
) -> Vec<EffectEnvelope> {
    *engine.state.navigation = NavigationState {
        route,
        params: params.unwrap_or(Value::Null),
    };
    vec![]
}
