mod calendar;
mod cross_provider;
mod merge;

#[cfg(test)]
mod worker_policy {
    use super::super::plan::external_sync_worker_retry_action;

    #[test]
    fn worker_policy_retries_transient_statuses_and_first_auth_failure() {
        assert_eq!(external_sync_worker_retry_action(Some(503), 2), "retry");
        assert_eq!(external_sync_worker_retry_action(Some(401), 0), "retry");
        assert_eq!(external_sync_worker_retry_action(Some(401), 1), "failure");
        assert_eq!(external_sync_worker_retry_action(Some(409), 0), "success");
        assert_eq!(external_sync_worker_retry_action(Some(400), 0), "failure");
    }
}
mod wire_fixtures;
