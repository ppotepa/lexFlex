use clap::{Parser, Subcommand};
use lexflex::chat::{ChatOptions, TraceMode};
use lexflex::engine::{ConversationEngine, EngineRequest, EngineResponse, EngineStatus, InspectTarget, LanguageMode, LocalSnapshotSourceProvider, SourceFetchPolicy, SourceKind, SourceProvider, SourceRequest, SourceSnapshot};
use std::ffi::OsString;
use std::path::Path;
use std::process;
use serde::Deserialize;
use sha2::{Digest, Sha256};

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
    /// Interactive terminal chat UI
    Chat {
        /// Source language (pl, en)
        #[arg(short, long, default_value = "pl")]
        from: String,
        /// Target language (pl, en)
        #[arg(short, long, default_value = "en")]
        to: String,
        /// Data directory
        #[arg(short, long)]
        data: Option<String>,
        /// Disable all live source access
        #[arg(long)]
        offline: bool,
        /// Default trace mode shown in the transcript
        #[arg(long, default_value = "full")]
        trace: String,
        /// Automatic source policy: live, cache-first or snapshot-only
        #[arg(long, default_value = "live")]
        source_policy: String,
    },

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
        #[arg(short, long)]
        data: Option<String>,
        /// Output format: json, pretty-json, summary, answer
        #[arg(long, default_value = "answer")]
        format: String,
    },
    /// Fetch or inspect raw source snapshots
    Source {
        #[command(subcommand)]
        command: SourceCommands,
    },
    /// Ingest a source snapshot through the conversation engine
    Ingest {
        title: String,
        #[arg(short, long, default_value = "en")] lang: String,
        #[arg(short, long)] data: Option<String>,
        #[arg(long, default_value = "default")] session: String,
        #[arg(long)] live: bool,
        #[arg(long, default_value = "pretty-json")] format: String,
    },
    /// Answer a natural-language question from a saved engine session
    Answer {
        text: OsString,
        #[arg(short, long, default_value = "auto")] lang: String,
        #[arg(short, long)] data: Option<String>,
        #[arg(long, default_value = "default")] session: String,
        #[arg(long, default_value = "live")] source_policy: String,
        #[arg(long, default_value = "pretty-json")] format: String,
    },
    /// Execute a serialized QueryInterlingua against a saved engine session
    Query {
        query: OsString,
        #[arg(short, long)] data: Option<String>,
        #[arg(long, default_value = "default")] session: String,
        #[arg(long, default_value = "pretty-json")] format: String,
    },
    /// Inspect a persisted engine session
    Inspect {
        #[arg(short, long)] data: Option<String>,
        #[arg(long, default_value = "default")] session: String,
        #[arg(long, default_value = "session")] target: String,
        #[arg(long, default_value = "pretty-json")] format: String,
    },
    /// Read a persisted deterministic engine trace
    Trace {
        #[arg(short, long)] data: Option<String>,
        #[arg(long, default_value = "default")] session: String,
        turn: String,
    },
    /// Validate or compare a structured benchmark corpus
    Corpus {
        #[command(subcommand)]
        command: CorpusCommands,
    },
    /// Save a deterministic session snapshot
    SessionSave {
        #[arg(short, long)] data: Option<String>,
        #[arg(long, default_value = "default")] session: String,
    },
    /// Load and validate a deterministic session snapshot
    SessionLoad {
        #[arg(short, long)] data: Option<String>,
        #[arg(long, default_value = "default")] session: String,
    },
}

#[derive(Subcommand)]
enum CorpusCommands {
    /// Validate a corpus manifest and report its deterministic case count
    Run {
        #[arg(default_value = "benchmarks/wikipedia_paris_v1")]
        path: String,
        #[arg(long, default_value = "summary")]
        format: String,
    },
    /// Compare two corpus manifests by canonical file hash
    Compare {
        baseline: String,
        candidate: String,
    },
}

#[derive(Subcommand)]
enum SourceCommands {
    /// Resolve and print a source snapshot
    Fetch {
        title: String,
        #[arg(short, long, default_value = "en")] lang: String,
        #[arg(short, long)] data: Option<String>,
        #[arg(long)] live: bool,
        #[arg(long, default_value = "pretty-json")] format: String,
    },
    /// Inspect a locally available source snapshot without network access
    Inspect {
        title: String,
        #[arg(short, long, default_value = "en")] lang: String,
        #[arg(short, long)] data: Option<String>,
        #[arg(long, default_value = "pretty-json")] format: String,
    },
}

#[derive(Debug, Deserialize)]
struct CorpusManifestCli {
    schema: u32,
    #[serde(default)]
    corpus_id: String,
    #[serde(default)]
    id: String,
    #[serde(default)]
    total_cases: usize,
    #[serde(default)]
    cases: Vec<ron::Value>,
    #[serde(default)]
    sources: Vec<ron::Value>,
    #[serde(default)]
    required_question_count: usize,
}

fn main() {
    let cli = Cli::parse();
    init_tracing(&cli);

    match cli.command {
        Commands::Chat {
            from,
            to,
            data,
            offline,
            trace,
            source_policy,
        } => {
            let trace_mode = match trace.as_str() {
                "off" => TraceMode::Off,
                "full" => TraceMode::Full,
                _ => TraceMode::Brief,
            };
            let source_policy = parse_source_policy(&source_policy);
            let data_dir = resolve_data_dir_or_exit(data.as_deref());

            let options = ChatOptions {
                from,
                to,
                data_dir,
                offline,
                trace_mode,
                source_policy,
            };

            if let Err(e) = lexflex::chat::run(options) {
                eprintln!("Chat UI error: {}", e);
                process::exit(1);
            }
        }
        Commands::Translate { text, from, to, data, format } => {
            let data = resolve_data_dir_or_exit(data.as_deref());
            let mut engine = engine_or_exit(&data, "translate", true);
            let response = engine.handle(EngineRequest::Translate { text: text.to_string_lossy().into_owned(), from: lexflex::core::interlingua::LanguageId::new(&from), to: lexflex::core::interlingua::LanguageId::new(&to) });
            print_engine_response(response, &format);
        }
        Commands::Source { command } => match command {
            SourceCommands::Fetch { title, lang, data, live, format } => {
                let data = resolve_data_dir_or_exit(data.as_deref());
                let request = source_request(title, &lang, if live { SourceFetchPolicy::Live } else { SourceFetchPolicy::CacheFirst });
                let provider = LocalSnapshotSourceProvider::new(&data, !live);
                match provider.resolve(&request) {
                    Ok(snapshot) => print_source_snapshot(snapshot, &format),
                    Err(error) => {
                        eprintln!("Source fetch error: {error:?}");
                        process::exit(1);
                    }
                }
            }
            SourceCommands::Inspect { title, lang, data, format } => {
                let data = resolve_data_dir_or_exit(data.as_deref());
                let request = source_request(title, &lang, SourceFetchPolicy::SnapshotOnly);
                let provider = LocalSnapshotSourceProvider::new(&data, true);
                match provider.resolve(&request) {
                    Ok(snapshot) => print_source_snapshot(snapshot, &format),
                    Err(error) => {
                        eprintln!("Source inspect error: {error:?}");
                        process::exit(1);
                    }
                }
            }
        },
        Commands::Ingest { title, lang, data, session, live, format } => {
            let data = resolve_data_dir_or_exit(data.as_deref());
            let mut engine = engine_or_exit(&data, &session, !live);
            let request = source_request(title, &lang, if live { SourceFetchPolicy::Live } else { SourceFetchPolicy::SnapshotOnly });
            let response = engine.handle(EngineRequest::IngestSource { source: request });
            print_engine_response(response, &format);
            if let Err(error) = engine.save_session() { eprintln!("Session save error: {error:?}"); process::exit(1); }
        }
        Commands::Answer { text, lang, data, session, source_policy, format } => {
            let data = resolve_data_dir_or_exit(data.as_deref());
            let policy = parse_source_policy(&source_policy);
            let mut engine = engine_or_exit(&data, &session, matches!(policy, SourceFetchPolicy::SnapshotOnly));
            engine = engine.with_auto_source_policy(policy);
            let has_session = engine.store.as_ref().and_then(|store| store.path_for(&session).ok()).is_some_and(|path| path.exists());
            if has_session {
                if let Err(error) = engine.load_session() { eprintln!("Session load error: {error:?}"); process::exit(1); }
            }
            let language = if lang == "auto" { LanguageMode::Auto } else { LanguageMode::Explicit(lang) };
            let response = engine.handle(EngineRequest::UserTurn { text: text.to_string_lossy().into_owned(), language });
            print_engine_response(response, &format);
            if let Err(error) = engine.save_session() { eprintln!("Session save error: {error:?}"); process::exit(1); }
        }
        Commands::Query { query, data, session, format } => {
            let data = resolve_data_dir_or_exit(data.as_deref());
            let mut engine = engine_or_exit(&data, &session, true);
            if let Err(error) = engine.load_session() { eprintln!("Session load error: {error:?}"); process::exit(1); }
            let query = serde_json::from_str(&query.to_string_lossy()).unwrap_or_else(|error| { eprintln!("Query JSON error: {error}"); process::exit(1); });
            print_engine_response(engine.handle(EngineRequest::Query { query }), &format);
        }
        Commands::Inspect { data, session, target, format } => {
            let data = resolve_data_dir_or_exit(data.as_deref());
            let mut engine = engine_or_exit(&data, &session, true);
            if let Err(error) = engine.load_session() { eprintln!("Session load error: {error:?}"); process::exit(1); }
            let target = match target.as_str() { "sources" => InspectTarget::Sources, "bundles" => InspectTarget::Bundles, _ => InspectTarget::Session };
            print_engine_response(engine.handle(EngineRequest::Inspect { target }), &format);
        }
        Commands::Trace { data, session, turn } => {
            let data = resolve_data_dir_or_exit(data.as_deref());
            let store = lexflex::engine::SessionStore::new(&data);
            match store.load_trace(&session, &turn) {
                Ok(trace) => println!("{trace}"),
                Err(error) => { eprintln!("Trace load error: {error:?}"); process::exit(1); }
            }
        }
        Commands::Corpus { command } => match command {
            CorpusCommands::Run { path, format } => match corpus_manifest(&path) {
                Ok((manifest, hash)) => {
                    let corpus_id = if manifest.corpus_id.is_empty() { &manifest.id } else { &manifest.corpus_id };
                    let case_count = if manifest.total_cases > 0 { manifest.total_cases } else { manifest.required_question_count.max(manifest.sources.len()) };
                    let valid = if manifest.cases.is_empty() { !manifest.sources.is_empty() && case_count > 0 } else { manifest.cases.len() == case_count && case_count > 0 };
                    if !valid { eprintln!("Corpus validation error: case count or source manifest mismatch"); process::exit(1); }
                    if format == "summary" { println!("corpus={} schema={} cases={} sha256={}", corpus_id, manifest.schema, case_count, hash); }
                    else { println!("{}", serde_json::json!({"corpus_id": corpus_id, "schema": manifest.schema, "cases": case_count, "manifest_sha256": hash})); }
                }
                Err(error) => { eprintln!("Corpus validation error: {error}"); process::exit(1); }
            },
            CorpusCommands::Compare { baseline, candidate } => match (manifest_hash(&baseline), manifest_hash(&candidate)) {
                (Ok(left), Ok(right)) => println!("{}", serde_json::json!({"equal": left == right, "baseline_sha256": left, "candidate_sha256": right})),
                (Err(error), _) | (_, Err(error)) => { eprintln!("Corpus compare error: {error}"); process::exit(1); }
            },
        },
        Commands::SessionSave { data, session } => {
            let data = resolve_data_dir_or_exit(data.as_deref());
            let mut engine = engine_or_exit(&data, &session, true);
            let existing = engine
                .store
                .as_ref()
                .and_then(|store| store.path_for(&session).ok())
                .is_some_and(|path| path.exists());
            if existing {
                if let Err(error) = engine.load_session() {
                    eprintln!("Session load error: {error:?}");
                    process::exit(1);
                }
            }
            match engine.save_session() { Ok(path) => println!("{}", path.display()), Err(error) => { eprintln!("Session save error: {error:?}"); process::exit(1); } }
        }
        Commands::SessionLoad { data, session } => {
            let data = resolve_data_dir_or_exit(data.as_deref());
            let mut engine = engine_or_exit(&data, &session, true);
            match engine.load_session() { Ok(()) => println!("{}", engine.session.snapshot_id), Err(error) => { eprintln!("Session load error: {error:?}"); process::exit(1); } }
        }
    }
}

fn parse_source_policy(value: &str) -> SourceFetchPolicy {
    match value {
        "snapshot-only" | "snapshot" => SourceFetchPolicy::SnapshotOnly,
        "cache-first" | "cache" => SourceFetchPolicy::CacheFirst,
        _ => SourceFetchPolicy::Live,
    }
}

fn manifest_path(path: &str) -> std::path::PathBuf {
    let path = Path::new(path);
    if path.file_name().and_then(|name| name.to_str()) == Some("manifest.ron") { path.to_path_buf() } else { path.join("manifest.ron") }
}

fn corpus_manifest(path: &str) -> Result<(CorpusManifestCli, String), String> {
    let path = manifest_path(path);
    let bytes = std::fs::read(&path).map_err(|error| error.to_string())?;
    let manifest: CorpusManifestCli = ron::de::from_bytes(&bytes).map_err(|error| error.to_string())?;
    let mut hash = Sha256::new(); hash.update(&bytes);
    Ok((manifest, format!("{:x}", hash.finalize())))
}

fn manifest_hash(path: &str) -> Result<String, String> { corpus_manifest(path).map(|(_, hash)| hash) }

fn source_request(title: String, lang: &str, policy: SourceFetchPolicy) -> SourceRequest {
    SourceRequest {
        source_kind: SourceKind::Wikipedia,
        title,
        language: lexflex::core::interlingua::LanguageId::new(lang),
        policy,
    }
}

fn engine_or_exit(data: &str, session: &str, offline: bool) -> ConversationEngine {
    ConversationEngine::new(data, session, offline).unwrap_or_else(|error| { eprintln!("Engine initialization error: {error:?}"); process::exit(1); })
}

fn resolve_data_dir_or_exit(explicit: Option<&str>) -> String {
    let resolved = lexflex::data::layout::resolve_data_root(explicit).unwrap_or_else(|error| {
        eprintln!("Runtime data root error: {error}");
        process::exit(1);
    });
    resolved.path.display().to_string()
}

fn print_engine_response(response: EngineResponse, format: &str) {
    if format == "summary" {
        match &response { EngineResponse::Ingest(value) => println!("status={:?} source={} bundle={} snapshot={}", value.meta.status, value.source_id, value.bundle_id, value.meta.session_snapshot_id), EngineResponse::Error { error, .. } => println!("status=Error error={error:?}"), _ => println!("status={:?}", engine_status(&response)) }
    } else {
        let output = if format == "json" { serde_json::to_string(&response) } else { serde_json::to_string_pretty(&response) };
        match output { Ok(value) => println!("{value}"), Err(error) => { eprintln!("Engine response serialization error: {error}"); process::exit(1); } }
    }
}

fn engine_status(response: &EngineResponse) -> EngineStatus {
    match response { EngineResponse::Conversation(value) => value.meta.status, EngineResponse::Translation(value) => value.meta.status, EngineResponse::Ingest(value) => value.meta.status, EngineResponse::Answer { meta, .. } => meta.status, EngineResponse::Inspection(value) => value.meta.status, EngineResponse::Error { meta, .. } => meta.status }
}

fn print_source_snapshot(snapshot: SourceSnapshot, format: &str) {
    if format == "summary" {
        println!(
            "source={} title={} language={} revision={} sha256={}",
            snapshot.source_id,
            snapshot.title,
            snapshot.language,
            snapshot.revision.as_deref().unwrap_or("none"),
            snapshot.content_sha256
        );
        return;
    }
    let output = if format == "json" {
        serde_json::to_string(&snapshot)
    } else {
        serde_json::to_string_pretty(&snapshot)
    };
    match output {
        Ok(value) => println!("{value}"),
        Err(error) => {
            eprintln!("Source serialization error: {error}");
            process::exit(1);
        }
    }
}

fn init_tracing(cli: &Cli) {
    let is_chat = matches!(cli.command, Commands::Chat { .. });
    let chat_logs_enabled = std::env::var("LEXFLEX_CHAT_LOG")
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let default_filter = if is_chat && !chat_logs_enabled {
        "off"
    } else {
        "lexflex=info"
    };
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(default_filter));
    tracing_subscriber::fmt().with_env_filter(env_filter).init();
}
