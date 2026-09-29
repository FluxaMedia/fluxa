use crate::ffi::*;
use serde_json::{Value, json};

pub(crate) fn route_plugins(method: &str, args_json: &str) -> Outcome {
    match method {
        "pluginManifestParse" => {
            let normalized = crate::addons::plugins::parse_plugin_manifest_json(args_json)
                .map_err(|message| fail(ErrorKind::InvalidArgs, message))?;
            into_json(normalized)
        }
        "pluginExecutionPlan" => opt_json(crate::addons::plugins::plugin_execution_plan_json(
            args_json,
        )),
        "pluginUpdatePlan" => opt_json(crate::addons::plugins::plugin_update_plan_json(args_json)),
        "pluginNetworkAddressAllowed" => Ok(json!(
            crate::addons::plugin_network::plugin_network_address_allowed(&arg_str(
                args_json, "address"
            )?)
        )),
        "pluginNetworkAddressBytesAllowed" => {
            let args = object(args_json)?;
            let bytes = args
                .get("bytes")
                .and_then(Value::as_array)
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "bytes must be an array"))?
                .iter()
                .map(|value| {
                    value
                        .as_u64()
                        .and_then(|byte| u8::try_from(byte).ok())
                        .ok_or_else(|| fail(ErrorKind::InvalidArgs, "bytes must contain octets"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(json!(
                crate::addons::plugin_network::plugin_network_address_bytes_allowed(&bytes)
            ))
        }
        "pluginUrlAllowed" => Ok(json!(crate::addons::plugin_network::plugin_url_allowed(
            &arg_str(args_json, "url",)?
        ))),
        "pluginStreamResultsParse" => into_json(
            crate::addons::plugins::parse_plugin_stream_results_json(args_json),
        ),
        "pluginStreamResultsToStreams" => {
            into_json(crate::addons::plugins::plugin_stream_results_to_streams_json(args_json))
        }

        _ => Err(unknown_method()),
    }
}
