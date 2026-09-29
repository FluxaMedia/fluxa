use super::helpers::{build_url, plan};

pub(crate) fn mdblist_user_url() -> String {
    build_url("/user", &[])
}

pub(crate) fn mdblist_user_stats_url() -> String {
    build_url("/user/stats", &[])
}

pub(crate) fn mdblist_public_user_url(username: &str) -> String {
    build_url(&format!("/users/{username}"), &[])
}

pub(crate) fn mdblist_user_follow_plan(username: &str, follow: bool) -> Option<String> {
    let method = if follow { "POST" } else { "DELETE" };
    plan(
        method,
        build_url(&format!("/users/{username}/follow"), &[]),
        None,
    )
}
