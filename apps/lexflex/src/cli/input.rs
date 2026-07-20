use crate::cli::input_error::{CliInputError, CliLocalError};
use crate::cli::internal_error::CliInternalError;
use crate::cli::output::CliExit;
use lexflex_engine::api::input::TextInput;
use lexflex_model::canonical_hash;
use lexflex_model::{LanguageId, ResourceBudget};
use std::fs;
use std::io::Read;
use std::path::PathBuf;

#[derive(clap::Args, Debug, Clone)]
pub struct TextInputArgs {
    #[arg(long)]
    pub language: String,

    #[arg(long)]
    pub source_id: Option<String>,

    #[arg(long, conflicts_with = "file")]
    pub text: Option<String>,

    #[arg(long, conflicts_with = "text")]
    pub file: Option<PathBuf>,
}

fn max_text_input_bytes() -> usize {
    ResourceBudget::default().max_text_bytes
}

fn read_text_bounded(reader: impl Read) -> Result<String, CliExit> {
    let mut bytes = Vec::new();
    let max_bytes = max_text_input_bytes();
    let mut limited = reader.take((max_bytes + 1) as u64);
    limited.read_to_end(&mut bytes).map_err(|error| {
        CliExit::Local(CliLocalError::Io {
            path: None,
            message: error.to_string(),
        })
    })?;
    if bytes.len() > max_bytes {
        return Err(CliExit::Input(CliInputError::InputTooLarge { max_bytes }));
    }
    String::from_utf8(bytes).map_err(|error| {
        CliExit::Local(CliLocalError::Io {
            path: None,
            message: format!("input is not valid UTF-8: {error}"),
        })
    })
}

pub(crate) fn read_file_bounded(path: &std::path::Path) -> Result<String, CliExit> {
    let handle = fs::File::open(path).map_err(|error| {
        CliExit::Local(CliLocalError::Io {
            path: Some(path.to_owned()),
            message: error.to_string(),
        })
    })?;
    read_text_bounded(handle).map_err(|error| match error {
        CliExit::Local(CliLocalError::Io { path: _, message }) => {
            CliExit::Local(CliLocalError::Io {
                path: Some(path.to_owned()),
                message,
            })
        }
        other => other,
    })
}

impl TextInputArgs {
    pub fn into_text_input(self) -> Result<TextInput, CliExit> {
        let TextInputArgs {
            language,
            source_id,
            text,
            file,
        } = self;
        let has_inline_text = text.is_some();
        let text = match (text, file.as_ref()) {
            (Some(text), _) => text,
            (None, Some(file)) => read_file_bounded(file)?,
            (None, None) => read_text_bounded(std::io::stdin())?,
        };
        if text.trim().is_empty() {
            return Err(CliExit::Input(CliInputError::EmptyText));
        }
        let source_id = match source_id {
            Some(source_id) => source_id,
            None => {
                let digest = canonical_hash(&text).map_err(|error| {
                    CliExit::Internal(CliInternalError::CanonicalHash {
                        message: error.to_string(),
                    })
                })?;
                if let Some(file) = file {
                    format!("file:{}", file.display())
                } else if has_inline_text {
                    format!("inline:{}", digest.as_str())
                } else {
                    format!("stdin:{}", digest.as_str())
                }
            }
        };
        Ok(TextInput {
            source_id,
            language: LanguageId::new(language)
                .map_err(CliInputError::InvalidLanguage)
                .map_err(CliExit::Input)?,
            text,
        })
    }
}
