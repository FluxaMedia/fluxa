use super::helpers::{build_url, extract_query, plan};
use serde_json::{Value, json};

pub(crate) fn mdblist_discussion_url(provider: &str, target_type: &str, target_id: i64) -> String {
    build_url(
        &format!("/discussion/{provider}/{target_type}/{target_id}"),
        &[],
    )
}

pub(crate) fn mdblist_discussion_summary_url(
    provider: &str,
    target_type: &str,
    target_id: i64,
) -> String {
    build_url(
        &format!("/discussion/{provider}/{target_type}/{target_id}/summary"),
        &[],
    )
}

pub(crate) fn mdblist_discussion_create_plan(
    provider: &str,
    target_type: &str,
    target_id: i64,
    comment: &str,
) -> Option<String> {
    if comment.trim().is_empty() {
        return None;
    }
    plan(
        "POST",
        build_url(
            &format!("/discussion/{provider}/{target_type}/{target_id}"),
            &[],
        ),
        Some(json!({ "content": comment })),
    )
}

pub(crate) fn mdblist_discussion_hot_url() -> String {
    build_url("/discussion/hot", &[])
}

pub(crate) fn mdblist_discussion_replies_url(comment_id: i64, args_json: &str) -> String {
    let args: Value = serde_json::from_str(args_json).unwrap_or(json!({}));
    build_url(
        &format!("/discussion/comments/{comment_id}/replies"),
        &extract_query(&args, &["limit", "offset"]),
    )
}

pub(crate) fn mdblist_discussion_reply_create_plan(
    comment_id: i64,
    comment: &str,
) -> Option<String> {
    if comment.trim().is_empty() {
        return None;
    }
    plan(
        "POST",
        build_url(&format!("/discussion/comments/{comment_id}/replies"), &[]),
        Some(json!({ "content": comment })),
    )
}

pub(crate) fn mdblist_discussion_comment_update_plan(
    comment_id: i64,
    comment: &str,
) -> Option<String> {
    if comment.trim().is_empty() {
        return None;
    }
    plan(
        "PATCH",
        build_url(&format!("/discussion/comments/{comment_id}"), &[]),
        Some(json!({ "content": comment })),
    )
}

pub(crate) fn mdblist_discussion_comment_delete_plan(comment_id: i64) -> Option<String> {
    plan(
        "DELETE",
        build_url(&format!("/discussion/comments/{comment_id}"), &[]),
        None,
    )
}

pub(crate) fn mdblist_discussion_comment_like_plan(comment_id: i64) -> Option<String> {
    plan(
        "POST",
        build_url(&format!("/discussion/comments/{comment_id}/like"), &[]),
        None,
    )
}

pub(crate) fn mdblist_discussion_reply_update_plan(reply_id: i64, comment: &str) -> Option<String> {
    if comment.trim().is_empty() {
        return None;
    }
    plan(
        "PATCH",
        build_url(&format!("/discussion/replies/{reply_id}"), &[]),
        Some(json!({ "content": comment })),
    )
}

pub(crate) fn mdblist_discussion_reply_delete_plan(reply_id: i64) -> Option<String> {
    plan(
        "DELETE",
        build_url(&format!("/discussion/replies/{reply_id}"), &[]),
        None,
    )
}

pub(crate) fn mdblist_discussion_reply_like_plan(reply_id: i64) -> Option<String> {
    plan(
        "POST",
        build_url(&format!("/discussion/replies/{reply_id}/like"), &[]),
        None,
    )
}
