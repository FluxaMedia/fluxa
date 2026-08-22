use super::{fail, object, ErrorKind, Outcome};
use crate::local_media;

pub(super) fn route_local_media(method: &str, args_json: &str) -> Outcome {
    let args = object(args_json)?;
    local_media::route(method, &args).ok_or_else(|| {
        fail(ErrorKind::UnknownMethod, format!("no such local-media method `{method}`"))
    })
}
