use thiserror::Error;

#[derive(Debug, Error)]
pub enum BenchmarkError {
    #[error("{0}")]
    Message(String),

    #[error("corpus invalid: {0}")]
    CorpusInvalid(String),

    #[error("critical regressions detected: {0}")]
    CriticalRegressions(usize),

    #[error("freeze refused")]
    FreezeRefused,

    #[error("io error at {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("RON error at {path}: {source}")]
    Ron {
        path: String,
        #[source]
        source: ron::error::SpannedError,
    },

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("profile validation failed: {0:?}")]
    ProfileValidation(Vec<lexflex::document::DocumentProfileValidationError>),
}

impl BenchmarkError {
    pub fn message(msg: impl Into<String>) -> Self {
        Self::Message(msg.into())
    }
}
