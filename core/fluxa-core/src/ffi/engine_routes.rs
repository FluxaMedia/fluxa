use super::*;

pub(super) fn route_engine_lifecycle(method: &str, args_json: &str) -> Outcome {
    match method {
        "engine.create" => {
            let handle = headless_engine::create_headless_engine(args_json);
            if handle == 0 {
                Err(fail(ErrorKind::InvalidArgs, "invalid initial engine state"))
            } else {
                Ok(json!(handle as i64))
            }
        }
        "engine.snapshot" => result_json(
            headless_engine::headless_engine_snapshot_json(handle(args_json)?),
            method,
        ),
        "engine.dispatch" => {
            let args = object(args_json)?;
            result_json(
                headless_engine::headless_engine_dispatch_json(
                    field_u64(&args, "handle")?,
                    &field(&args, "action")?.to_string(),
                ),
                method,
            )
        }
        "engine.completeEffect" => {
            let args = object(args_json)?;
            result_json(
                headless_engine::headless_engine_complete_effect_json(
                    field_u64(&args, "handle")?,
                    &field(&args, "result")?.to_string(),
                ),
                method,
            )
        }
        "engine.destroy" => Ok(json!(headless_engine::destroy_headless_engine(handle(
            args_json
        )?))),
        "core.drainErrorLog" => opt_json(Some(crate::log_sink::drain_core_log_json())),
        "app.create" => {
            let handle = app_state::create_app_core_state(args_json);
            if handle == 0 {
                Err(fail(ErrorKind::InvalidArgs, "invalid initial app state"))
            } else {
                Ok(json!(handle as i64))
            }
        }
        "app.state" => result_json(app_state::app_core_state_json(handle(args_json)?), method),
        "app.dispatch" => {
            let args = object(args_json)?;
            result_json(
                app_state::app_core_dispatch_json(
                    field_u64(&args, "handle")?,
                    &field(&args, "action")?.to_string(),
                ),
                method,
            )
        }
        "app.dispatchDelta" => {
            let args = object(args_json)?;
            result_json(
                app_state::app_core_dispatch_delta_json(
                    field_u64(&args, "handle")?,
                    &field(&args, "action")?.to_string(),
                ),
                method,
            )
        }
        "app.destroy" => Ok(json!(app_state::destroy_app_core_state(handle(args_json)?))),
        _ => Err(unknown_method()),
    }
}
