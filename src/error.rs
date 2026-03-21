use crate::exit_codes;

#[derive(Debug, thiserror::Error)]
pub enum ShrugError {
    #[error("Authentication failed: {0}")]
    AuthError(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Rate limited. Retry after {retry_after:?} seconds")]
    RateLimited { retry_after: Option<u64> },

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Server error (HTTP {status}): {message}")]
    ServerError { status: u16, message: String },

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Specification error: {0}")]
    SpecError(String),

    #[error("Usage error: {0}")]
    UsageError(String),

    #[error("Profile error: {0}")]
    ProfileError(String),
}

impl ShrugError {
    pub fn exit_code(&self) -> i32 {
        match self {
            ShrugError::AuthError(_) => exit_codes::AUTH_ERROR,
            ShrugError::NotFound(_) => exit_codes::NOT_FOUND,
            ShrugError::PermissionDenied(_) => exit_codes::PERMISSION_DENIED,
            ShrugError::RateLimited { .. } => exit_codes::RATE_LIMITED,
            ShrugError::NetworkError(_) => exit_codes::NETWORK_ERROR,
            ShrugError::ServerError { .. } => exit_codes::SERVER_ERROR,
            ShrugError::ConfigError(_) => exit_codes::GENERAL_ERROR,
            ShrugError::SpecError(_) => exit_codes::GENERAL_ERROR,
            ShrugError::UsageError(_) => exit_codes::USAGE_ERROR,
            ShrugError::ProfileError(_) => exit_codes::GENERAL_ERROR,
        }
    }

    pub fn remediation(&self) -> &'static str {
        match self {
            ShrugError::AuthError(_) => {
                "Check your API token or run `shrug auth setup` to reconfigure."
            }
            ShrugError::NotFound(_) => {
                "Verify the resource ID/key. Use `shrug jira +search` to find items."
            }
            ShrugError::PermissionDenied(_) => {
                "Check your Atlassian permissions for this resource."
            }
            ShrugError::RateLimited { .. } => {
                "Wait and retry. Reduce request frequency or use --limit."
            }
            ShrugError::NetworkError(_) => "Check your internet connection and site URL.",
            ShrugError::ServerError { .. } => "This is an Atlassian server issue. Retry later.",
            ShrugError::ConfigError(_) => {
                "Check your config file. Run `shrug --help` for defaults."
            }
            ShrugError::SpecError(_) => {
                "Try clearing the cache: delete the cache directory and retry."
            }
            ShrugError::UsageError(_) => "Run `shrug --help` for usage information.",
            ShrugError::ProfileError(_) => "Run `shrug profile list` to see available profiles.",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_error_exit_code() {
        let err = ShrugError::AuthError("bad token".into());
        assert_eq!(err.exit_code(), exit_codes::AUTH_ERROR);
    }

    #[test]
    fn not_found_exit_code() {
        let err = ShrugError::NotFound("issue PROJ-1".into());
        assert_eq!(err.exit_code(), exit_codes::NOT_FOUND);
    }

    #[test]
    fn permission_denied_exit_code() {
        let err = ShrugError::PermissionDenied("no access".into());
        assert_eq!(err.exit_code(), exit_codes::PERMISSION_DENIED);
    }

    #[test]
    fn rate_limited_exit_code() {
        let err = ShrugError::RateLimited {
            retry_after: Some(30),
        };
        assert_eq!(err.exit_code(), exit_codes::RATE_LIMITED);
    }

    #[test]
    fn server_error_exit_code() {
        let err = ShrugError::ServerError {
            status: 500,
            message: "internal".into(),
        };
        assert_eq!(err.exit_code(), exit_codes::SERVER_ERROR);
    }

    #[test]
    fn config_error_exit_code() {
        let err = ShrugError::ConfigError("missing file".into());
        assert_eq!(err.exit_code(), exit_codes::GENERAL_ERROR);
    }

    #[test]
    fn spec_error_exit_code() {
        let err = ShrugError::SpecError("invalid spec".into());
        assert_eq!(err.exit_code(), exit_codes::GENERAL_ERROR);
    }

    #[test]
    fn usage_error_exit_code() {
        let err = ShrugError::UsageError("missing argument".into());
        assert_eq!(err.exit_code(), exit_codes::USAGE_ERROR);
    }

    #[test]
    fn profile_error_exit_code() {
        let err = ShrugError::ProfileError("bad profile".into());
        assert_eq!(err.exit_code(), exit_codes::GENERAL_ERROR);
    }

    #[test]
    fn remediation_hints_are_non_empty() {
        let errors: Vec<ShrugError> = vec![
            ShrugError::AuthError("test".into()),
            ShrugError::NotFound("test".into()),
            ShrugError::PermissionDenied("test".into()),
            ShrugError::RateLimited {
                retry_after: Some(10),
            },
            ShrugError::ServerError {
                status: 503,
                message: "unavailable".into(),
            },
            ShrugError::ConfigError("test".into()),
            ShrugError::SpecError("test".into()),
            ShrugError::UsageError("test".into()),
            ShrugError::ProfileError("test".into()),
        ];
        for err in &errors {
            let hint = err.remediation();
            assert!(
                !hint.is_empty(),
                "Remediation for {err:?} should not be empty"
            );
        }
    }

    #[test]
    fn auth_error_remediation_mentions_setup() {
        let err = ShrugError::AuthError("bad token".into());
        assert!(err.remediation().contains("shrug auth setup"));
    }

    #[test]
    fn display_messages_are_non_empty() {
        let errors: Vec<ShrugError> = vec![
            ShrugError::AuthError("test".into()),
            ShrugError::NotFound("test".into()),
            ShrugError::PermissionDenied("test".into()),
            ShrugError::RateLimited {
                retry_after: Some(10),
            },
            ShrugError::ServerError {
                status: 503,
                message: "unavailable".into(),
            },
            ShrugError::ConfigError("test".into()),
            ShrugError::SpecError("test".into()),
            ShrugError::UsageError("test".into()),
            ShrugError::ProfileError("test".into()),
        ];
        for err in &errors {
            let msg = format!("{err}");
            assert!(!msg.is_empty(), "Display for {err:?} should not be empty");
        }
    }
}
