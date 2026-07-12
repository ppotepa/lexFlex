use clap::{Parser, Subcommand};
use lexflex::api::LexFlexAPI;
use std::collections::HashSet;

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
        text: String,
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
        text: String,
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

            let lines: Vec<&str> = content
                .lines()
                .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
                .collect();

            let process_lines = if max > 0 { &lines[..max.min(lines.len())] } else { &lines[..] };

            let mut suggestions = vec![];
            let mut seen: HashSet<String> = HashSet::new();
            let mut hints = vec![];

            for line in process_lines {
                let sentence = line.trim();
                let words: Vec<String> = sentence
                    .split(|c: char| !c.is_alphanumeric())
                    .filter(|w| w.len() > 2)
                    .map(|w| w.to_lowercase())
                    .collect();

                for word in words {
                    if seen.contains(&word) || existing_lexicon.contains(&word) {
                        continue;
                    }
                    seen.insert(word.clone());

                    // Real lexical deduction (the same one lexlearn uses in --offline mode)
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    let res = rt.block_on(async {
                        let svc = LexicalDeductionService::new_offline();
                        let ctx = extract_simple_context(sentence, &word, 5);
                        svc.deduce(&word, language, Some(&ctx)).await.ok()
                    });

                    if let Some(r) = res {
                        if let Some(c) = &r.best_concept {
                            let feat = if let Some(n) = &r.surface.number {
                                format!("number: Some({})", n)
                            } else { "number: Some(Singular)".to_string() };

                            let ron = format!(
                                r#"("{}" , (lemma: "{}", pos: "{}", concept: "{}", features: ({})))"#,
                                word,
                                r.surface.lemma.as_deref().unwrap_or(&word),
                                r.surface.pos.as_deref().unwrap_or("Noun"),
                                c.concept_id,
                                feat
                            );
                            suggestions.push(ron);

                            if graph_hints {
                                hints.push(serde_json::json!({
                                    "word": word, "evokes": c.concept_id, "is_a": r.semantics.is_a
                                }));
                            }
                            println!("   ✓ {} → {} ({})", word, c.concept_id, c.reason);
                        }
                    }
                }
            }

            if !suggestions.is_empty() {
                let mut out = String::new();
                out.push_str("// Generated by: lexflex learn\n");
                out.push_str("// Review and append the unique ones to your lexicon.ron\n");
                out.push_str("[\n");
                for s in &suggestions {
                    out.push_str(&format!("    {},\n", s));
                }
                out.push_str("]\n");

                std::fs::write(&output, out).expect("Failed to write suggestions");
                println!("\n✅ {} new suggestions written to {}", suggestions.len(), output);

                if graph_hints {
                    let hpath = output.replace(".ron", ".hints.json");
                    std::fs::write(&hpath, serde_json::to_string_pretty(&hints).unwrap()).ok();
                    println!("   Graph hints: {}", hpath);
                }
            } else {
                println!("\nAll words in the input were already covered by the local lexicon.");
            }
        }
    }
}

fn extract_simple_context(s: &str, w: &str, win: usize) -> String {
    let toks: Vec<&str> = s.to_lowercase().split_whitespace().collect();
    if let Some(p) = toks.iter().position(|t| t.contains(&w.to_lowercase())) {
        let a = p.saturating_sub(win);
        let b = (p + win + 1).min(toks.len());
        toks[a..b].join(" ")
    } else { s.to_string() }
}
