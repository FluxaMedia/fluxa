use super::*;

pub(super) fn route_calendar(method: &str, args_json: &str) -> Outcome {
    match method {
        "calendarCandidatePlan" => opt_json(calendar_plan::calendar_candidate_plan_json(args_json)),
        "calendarVisibilityPlan" => {
            opt_json(calendar_plan::calendar_visibility_plan_json(args_json))
        }
        "calendarReleaseRows" => opt_json(calendar_plan::calendar_release_rows_json(args_json)),
        "calendarContentPlan" => opt_json(calendar_plan::calendar_content_plan_json(args_json)),
        "desktopCalendarReadPlan" => {
            opt_json(calendar_plan::desktop_calendar_read_plan_json(args_json))
        }
        "calendarSeasonCandidates" => {
            opt_json(calendar_plan::calendar_season_candidates_json(args_json))
        }
        "calendarWidgetRows" => opt_json(calendar_plan::calendar_widget_rows_json(args_json)),
        "calendarNotificationContent" => {
            opt_json(calendar_plan::calendar_notification_content_json(args_json))
        }
        "calendarReleaseDetection" => {
            opt_json(calendar_plan::calendar_release_detection_json(args_json))
        }
        "calendarItemsFromMeta" => {
            let args = object(args_json)?;
            opt_json(calendar_plan::calendar_items_from_meta_json(
                field_str(&args, "metaJson")?,
                field_str(&args, "monthPrefix")?,
            ))
        }
        "calendarItemMatchesMonth" => {
            let args = object(args_json)?;
            Ok(json!(calendar_plan::calendar_item_matches_month_json(
                field_str(&args, "itemJson")?,
                field_str(&args, "monthPrefix")?,
            )))
        }
        "nextUnairedEpisode" => {
            let args = object(args_json)?;
            let now_ms = field(&args, "nowMs")?
                .as_i64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "nowMs must be a number"))?;
            opt_json(calendar_plan::next_unaired_episode_json(
                field_str(&args, "videosJson")?,
                now_ms,
            ))
        }
        "partitionThisWeek" => {
            let args = object(args_json)?;
            let now_ms = field(&args, "nowMs")?
                .as_i64()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "nowMs must be a number"))?;
            let keep_scheduled = field(&args, "keepScheduled")?
                .as_bool()
                .ok_or_else(|| fail(ErrorKind::InvalidArgs, "keepScheduled must be bool"))?;
            opt_json(calendar_plan::partition_this_week_json(
                field_str(&args, "itemsJson")?,
                now_ms,
                keep_scheduled,
            ))
        }

        _ => Err(unknown_method()),
    }
}
