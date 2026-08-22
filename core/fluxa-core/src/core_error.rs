use crate::log_sink;

pub(crate) enum CoreError {
    BadInput {
        context: &'static str,
        detail: String,
    },
    NotFound {
        context: &'static str,
    },
}

impl CoreError {
    fn log(&self) {
        match self {
            CoreError::BadInput { context, detail } => log_sink::record(context, detail),
            CoreError::NotFound { context } => log_sink::record(context, "not found"),
        }
    }

    pub(crate) fn log_and_none<T>(self) -> Option<T> {
        self.log();
        None
    }
}

pub(crate) trait LogAndDiscard<T> {
    fn log_discard(self) -> Option<T>;
}

impl<T> LogAndDiscard<T> for Result<T, CoreError> {
    fn log_discard(self) -> Option<T> {
        match self {
            Ok(value) => Some(value),
            Err(err) => {
                err.log();
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bad_input_is_logged_and_becomes_none() {
        let result: Result<(), CoreError> = Err(CoreError::BadInput {
            context: "wire_test_core_error",
            detail: "malformed field".to_string(),
        });
        assert_eq!(result.log_discard(), None);
        let drained: Vec<String> = serde_json::from_str(&log_sink::drain_core_log_json()).unwrap();
        assert!(drained.contains(&"wire_test_core_error: malformed field".to_string()));
    }
}
