use clap::{Parser, Subcommand};
use lexflex::api::LexFlexAPI;
use std::ffi::OsString;

#[derive(Parser)]
#[command(name = "lexflex")]
#[command(about = "Universal meaning representation framework")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Translate text between languages
    Translate {
        /// Input text
        text: OsString,
        /// Source language (pl, en)
        #[arg(short, long, default_value = "pl")]
        from: String,
        /// Target language (pl, en)
        #[arg(short, long, default_value = "en")]
        to: String,
        /// Data directory
        #[arg(short, long, default_value = "data")]
        data: String,
    },
    /// Parse text to Interlingua representation
    Parse {
        /// Input text
        text: OsString,
        /// Source language (pl, en)
        #[arg(short, long, default_value = "pl")]
        lang: String,
        /// Data directory
        #[arg(short, long, default_value = "data")]
        data: String,
    },
    /// List supported languages
    Languages {
        /// Data directory
        #[arg(short, long, default_value = "data")]
        data: String,
    },
    /// Semantic digest: actors, actions, roles, discourse (IL-derived)
    Explain {
        /// Input text
        text: OsString,
        /// Source language (pl, en)
        #[arg(short, long, default_value = "pl")]
        lang: String,
        /// Output format: human or json
        #[arg(short, long, default_value = "human")]
        format: String,
        /// Data directory
        #[arg(short, long, default_value = "data")]
        data: String,
    },
}

fn main() {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("lexflex=info"));
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Translate { text, from, to, data } => {
            let text = text.to_string_lossy().to_string();
            let api = LexFlexAPI::builder()
                .data_dir(&data)
                .build()
                .expect("Failed to initialize lexFlex");

            match api.translate(&text, &from, &to) {
                Ok(result) => println!("{}", result),
                Err(e) => {
                    eprintln!("Translation error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Parse { text, lang, data } => {
            let text = text.to_string_lossy().to_string();
            let api = LexFlexAPI::builder()
                .data_dir(&data)
                .build()
                .expect("Failed to initialize lexFlex");

            match api.parse(&text, &lang) {
                Ok(il) => println!("{:#?}", il),
                Err(e) => {
                    eprintln!("Parse error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Languages { data } => {
            let api = LexFlexAPI::builder()
                .data_dir(&data)
                .build()
                .expect("Failed to initialize lexFlex");

            let langs = api.supported_languages();
            println!("Supported languages:");
            for lang in langs {
                println!("  - {}", lang);
            }
        }
        Commands::Explain {
            text,
            lang,
            format,
            data,
        } => {
            let text = text.to_string_lossy().to_string();
            let api = LexFlexAPI::builder()
                .data_dir(&data)
                .build()
                .expect("Failed to initialize lexFlex");

            let result = match format.as_str() {
                "json" => api.explain_json(&text, &lang),
                "human" | _ => api.explain_human(&text, &lang),
            };
            match result {
                Ok(out) => println!("{}", out),
                Err(e) => {
                    eprintln!("Explain error: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}