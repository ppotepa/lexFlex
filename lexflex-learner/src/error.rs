use thiserror::Error;

#[derive(Error, Debug)]
pub enum LearnerError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON parsing failed: {0}")]
    Json(#[from] serde_json::Error),

    #[error("No data found for word '{word}' in language {lang}")]
    NoData { word: String, lang: String },

    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),

    #[error("API rate limit or temporary error: {0}")]
    ApiError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, LearnerError>;
