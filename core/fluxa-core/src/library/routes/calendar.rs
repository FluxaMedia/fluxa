use crate::ffi::*;

pub(crate) fn route_calendar(method: &str, args_json: &str) -> Outcome {
    match method {
        "desktopCalendarReadPlan" => opt_json(
            crate::library::calendar::desktop_calendar_read_plan_json(args_json),
        ),
        _ => Err(unknown_method()),
    }
}
