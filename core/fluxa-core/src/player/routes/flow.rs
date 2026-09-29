use crate::ffi::*;

pub(crate) fn route_player_flow(method: &str, args_json: &str) -> Outcome {
    match method {
        "playerFlowDispatch" => {
            let args = object(args_json)?;
            opt_json(crate::player::flow::player_flow_dispatch_json(
                field_str(&args, "state")?,
                field_str(&args, "action")?,
            ))
        }

        _ => Err(unknown_method()),
    }
}
