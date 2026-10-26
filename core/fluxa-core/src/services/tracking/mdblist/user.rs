use super::helpers::build_url;

pub(crate) fn mdblist_user_url() -> String {
    build_url("/user", &[])
}
