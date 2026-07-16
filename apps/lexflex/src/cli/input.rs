use lexflex_engine::api::input::TextInput;
use lexflex_model::canonical_hash;
use lexflex_model::LanguageId;
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

impl TextInputArgs {
    pub fn into_text_input(self) -> Result<TextInput, String> {
        let TextInputArgs {
            language,
            source_id,
            text,
            file,
        } = self;
        let has_inline_text = text.is_some();
        let text = match (text, file.as_ref()) {
            (Some(text), _) => text,
            (None, Some(file)) => {
                fs::read_to_string(file).map_err(|error| format!("{}: {error}", file.display()))?
            }
            (None, None) => {
                let mut buffer = String::new();
                std::io::stdin()
                    .read_to_string(&mut buffer)
                    .map_err(|error| error.to_string())?;
                buffer
            }
        };
        let source_id = source_id.unwrap_or_else(|| {
            let digest = canonical_hash(&text);
            if let Some(file) = file {
                format!("file:{}", file.display())
            } else if has_inline_text {
                format!("inline:{digest}")
            } else {
                format!("stdin:{digest}")
            }
        });
        Ok(TextInput {
            source_id,
            language: LanguageId::new(language).map_err(|error| error.to_string())?,
            text,
        })
    }
}
