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
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("lexflex=info".parse().unwrap()),
        )
        .init();

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
    }
}