use super::{CallError, ErrorKind, Outcome, fail};
use serde_json::Value;

pub(crate) fn opt_str(value: Option<String>) -> Outcome {
    Ok(value.map(Value::String).unwrap_or(Value::Null))
}

pub(crate) fn opt_json(value: Option<String>) -> Outcome {
    Ok(match value {
        Some(s) => serde_json::from_str(&s).map_err(|e| {
            fail(
                ErrorKind::Internal,
                format!("core produced invalid JSON: {e}"),
            )
        })?,
        None => Value::Null,
    })
}

pub(crate) fn object(args_json: &str) -> Result<Value, CallError> {
    let value: Value = serde_json::from_str(args_json).map_err(|e| {
        fail(
            ErrorKind::InvalidArgs,
            format!("args is not valid JSON: {e}"),
        )
    })?;
    if value.is_object() {
        Ok(value)
    } else {
        Err(fail(ErrorKind::InvalidArgs, "args must be a JSON object"))
    }
}

pub(crate) fn arg_str(args_json: &str, name: &str) -> Result<String, CallError> {
    let args = object(args_json)?;
    Ok(field_str(&args, name)?.to_string())
}

pub(crate) fn field<'a>(args: &'a Value, name: &str) -> Result<&'a Value, CallError> {
    args.get(name)
        .ok_or_else(|| fail(ErrorKind::InvalidArgs, format!("missing field `{name}`")))
}

pub(crate) fn field_str<'a>(args: &'a Value, name: &str) -> Result<&'a str, CallError> {
    field(args, name)?.as_str().ok_or_else(|| {
        fail(
            ErrorKind::InvalidArgs,
            format!("field `{name}` must be a string"),
        )
    })
}

pub(crate) fn field_u64(args: &Value, name: &str) -> Result<u64, CallError> {
    field(args, name)?.as_u64().ok_or_else(|| {
        fail(
            ErrorKind::InvalidArgs,
            format!("field `{name}` must be a non-negative integer"),
        )
    })
}

pub(crate) fn field_i64(args: &Value, name: &str) -> Result<i64, CallError> {
    field(args, name)?.as_i64().ok_or_else(|| {
        fail(
            ErrorKind::InvalidArgs,
            format!("field `{name}` must be an integer"),
        )
    })
}

pub(crate) fn handle(args_json: &str) -> Result<u64, CallError> {
    let value: Value = serde_json::from_str(args_json).map_err(|e| {
        fail(
            ErrorKind::InvalidArgs,
            format!("args is not valid JSON: {e}"),
        )
    })?;
    value
        .as_u64()
        .or_else(|| value.get("handle").and_then(Value::as_u64))
        .ok_or_else(|| {
            fail(
                ErrorKind::InvalidArgs,
                "expected a handle (number or { handle })",
            )
        })
}

pub(crate) fn result_json(value: Option<String>, method: &str) -> Outcome {
    match value {
        Some(s) => into_json(s),
        None => Err(fail(
            ErrorKind::NotFound,
            format!("`{method}` produced no result"),
        )),
    }
}

pub(crate) fn into_json(s: String) -> Outcome {
    serde_json::from_str(&s).map_err(|e| {
        fail(
            ErrorKind::Internal,
            format!("core produced invalid JSON: {e}"),
        )
    })
}
