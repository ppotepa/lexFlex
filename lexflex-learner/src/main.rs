use clap::{Parser, Subcommand};
use lexflex_learner::{LexicalDeductionService, Language, DeductionResult, bulk::run_bulk};
use std::process;

#[derive(Parser)]
#[command(name = "lexlearn", version, about = "Lexical Deduction Service for lexFlex - learn new words using free APIs")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Deduce information about a single word using multiple free APIs
    Deduce {
        /// The word to learn about
        word: String,

        /// Language (pl or en)
        #[arg(short, long, default_value = "pl")]
        lang: String,

        /// Optional context sentence to help disambiguate
        #[arg(short, long)]
        context: Option<String>,

        /// Output format: pretty, ron, json
        #[arg(short, long, default_value = "pretty")]
        output: String,

        /// Offline / local-only mode: use only LocalKnowledge (lexicon + concepts from data/*.ron).
        /// No network calls.
        #[arg(long)]
        offline: bool,
    },

    /// Bulk process a file with sentences or words (e.g. benchmarks/input_adam.txt)
    /// Supports lines like "PL->EN: Mieszkam razem z żoną..."
    /// or simple one word/sentence per line.
    Bulk {
        /// Input file path (sentences or words)
        #[arg(short, long)]
        input: String,

        /// Language (pl or en)
        #[arg(short, long, default_value = "pl")]
        lang: String,

        /// Output directory for results (will create JSON files + summary)
        #[arg(short, long, default_value = "results/learned")]
        output_dir: String,

        /// Maximum number of items (lines) to process (0 = all). Useful for testing ~100 sentences.
        #[arg(long, default_value = "0")]
        max: usize,

        /// Delay in ms between word deductions (to respect API rate limits)
        #[arg(long, default_value = "800")]
        delay_ms: u64,

        /// Offline / local-only mode: use only LocalKnowledge (lexicon + concepts from data/*.ron).
        /// No network calls to ConceptNet / DictionaryAPI / Wiktionary.
        #[arg(long)]
        offline: bool,
    },

    /// Show version and available sources
    Info,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Deduce { word, lang, context, output, offline } => {
            let language = match Language::from_str(&lang) {
                Some(l) => l,
                None => {
                    eprintln!("Unsupported language: {}. Use 'pl' or 'en'.", lang);
                    process::exit(1);
                }
            };

            println!("🔍 Learning '{}' (lang={}) ...", word, lang);
            if let Some(ctx) = &context {
                println!("   Context: {}", ctx);
            }

            let service = if offline {
                LexicalDeductionService::new_offline()
            } else {
                LexicalDeductionService::new()
            };

            match service.deduce(&word, language, context.as_deref()).await {
                Ok(result) => {
                    print_result(&result, &output);
                }
                Err(e) => {
                    eprintln!("Error learning word: {}", e);
                    process::exit(1);
                }
            }
        }
        Commands::Bulk { input, lang, output_dir, max, delay_ms, offline } => {
            let language = match Language::from_str(&lang) {
                Some(l) => l,
                None => {
                    eprintln!("Unsupported language: {}. Use 'pl' or 'en'.", lang);
                    process::exit(1);
                }
            };

            if let Err(e) = run_bulk(&input, language, &output_dir, max, delay_ms, offline).await {
                eprintln!("Bulk processing error: {}", e);
                process::exit(1);
            }
        }
        Commands::Info => {
            println!("lexflex-learner v0.1 — Word Concept Deducer");
            println!("Goal: for every new word found while reading, create a main Interlingua concept");
            println!("      + related surface words in English and Polish (proposals).");
            println!();
            println!("Sources (in priority order):");
            println!("  1. LocalLlm (if LEXFLEX_LLM_* env vars are set)   ← first now");
            println!("  2. LocalKnowledge (from data/*.ron)");
            println!("  3. ConceptNet + DictionaryAPI + Wiktionary");
            println!();
            println!("Optional Local LLM fallback (highly recommended for Polish):");
            println!("  Set these environment variables:");
            println!("    export LEXFLEX_LLM_BASE_URL=http://host.docker.internal:1234/v1");
            println!("    export LEXFLEX_LLM_MODEL=google/gemma-4-e2b   # or whatever tag you use");
            println!("  Then run without --offline. The LLM runs after lexicon but before web sources.");
            println!();
            println!("Examples:");
            println!("  Local-only:               lexlearn deduce \"słowo\" --lang pl --offline");
            println!("  With local LLM:           LEXFLEX_LLM_BASE_URL=... lexlearn deduce \"wielkimi\" --lang pl");
            println!("  In Docker:                docker run -e LEXFLEX_LLM_BASE_URL=... lexflex lexlearn deduce \"wielkimi\" --lang pl");
            println!("  Bulk:                     lexlearn bulk --input text.txt --lang pl --max 50 --offline");
            println!();
            println!("Key behavior:");
            println!("  - Strong preference for data from lexicon.ron");
            println!("  - LLM helps with difficult Polish inflections and novel words");
            println!("  - New concepts are proposed conservatively");
            println!();
            println!("Note: Main `lexflex` binary is 100% offline. Merge good proposals into data/.");
        }
    }
}

// run_bulk now from lexflex_learner::bulk (extracted for testability and AC4)

fn print_result(result: &DeductionResult, format: &str) {
    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(result).unwrap());
        }
        "ron" => {
            if let Some(ron) = &result.suggested_ron_entry {
                println!("{}", ron);
            } else {
                println!("// No strong concept match. Raw data:");
                println!("{}", ron::ser::to_string_pretty(result, ron::ser::PrettyConfig::default()).unwrap());
            }
        }
        _ => {
            // pretty
            println!("\n=== Result for '{}' ===", result.word);
            println!("Language: {}", result.lang);

            println!("\n[Surface]");
            if let Some(l) = &result.surface.lemma {
                println!("  lemma: {}", l);
            }
            if let Some(p) = &result.surface.pos {
                println!("  pos: {}", p);
            }
            if !result.surface.example_sentences.is_empty() {
                println!("  examples: {:?}", result.surface.example_sentences);
            }

            println!("\n[Semantics]");
            if !result.semantics.definitions.is_empty() {
                println!("  definitions:");
                for d in &result.semantics.definitions {
                    println!("    - {}", d);
                }
            }
            if !result.semantics.is_a.is_empty() {
                println!("  IsA (very useful for Concept): {:?}", result.semantics.is_a);
            }
            if !result.semantics.related.is_empty() {
                println!("  related: {:?}", result.semantics.related);
            }

            println!("\n[Concept Inference]");
            if let Some(c) = &result.best_concept {
                println!("  best_concept: {}", c.concept_id);
                println!("  confidence: {:.2}", c.confidence);
                println!("  reason: {}", c.reason);
            } else {
                println!("  No strong concept match found.");
            }

            if let Some(prop) = &result.concept_proposal {
                println!("\n[Proposed Main Interlingua Concept (new or primary)]");
                println!("  concept_id: {}", prop.concept_id);
                println!("  confidence: {:.2}", prop.confidence);
                println!("  reason: {}", prop.reason);
                if let Some(p) = &prop.parent_suggestion {
                    println!("  suggested_parent: {}", p);
                }
            }

            if let Some(ron) = &result.suggested_ron_entry {
                println!("\n[Suggested RON entry for lexicon.ron]");
                println!("{}", ron);
            }

            if !result.lexicon_proposals.is_empty() {
                println!("\n[Bilingual / Related Lexicon Proposals]");
                for lp in &result.lexicon_proposals {
                    println!("  [{}]: {}", lp.lang, lp.ron_line);
                }
            }

            println!("\nSources used: {:?}", result.sources_used);
            println!("Overall confidence: {:.2}", result.confidence);
        }
    }
}
