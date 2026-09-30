use serde_json::Value;

pub(crate) fn parse(args_json: &str) -> Option<Value> {
    serde_json::from_str(args_json).ok()
}

pub(crate) fn str_field<'a>(value: &'a Value, name: &str) -> Option<&'a str> {
    value.get(name).and_then(Value::as_str)
}
