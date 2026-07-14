use clap::{Parser, Subcommand};
use lexflex::api::LexFlexAPI;
use lexflex::chat::{ChatOptions, TraceMode};
use lexflex::document::temporal_discourse::DocumentTemporalDiscourse;
use std::ffi::OsString;
use std::process;

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
        #[arg(short, long, default_value = "data")]
        data: String,
        /// Disable any LLM-backed learner integration
        #[arg(long)]
        offline: bool,
        /// Default trace mode shown in the transcript
        #[arg(long, default_value = "brief")]
        trace: String,
        /// Optional label shown in the header for the configured endpoint
        #[arg(long)]
        base_url: Option<String>,
        /// Optional label shown in the header for the configured model
        #[arg(long)]
        model: Option<String>,
        /// Optional system prompt label shown in the header
        #[arg(long)]
        system_prompt: Option<String>,
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
    /// Build a document graph from source text
    DocumentGraph {
        /// Input text
        text: OsString,
        /// Source language (pl, en)
        #[arg(short, long, default_value = "pl")]
        lang: String,
        /// Data directory
        #[arg(short, long, default_value = "data")]
        data: String,
        /// Output format: json, pretty-json, summary
        #[arg(short, long, default_value = "json")]
        format: String,
    },
    /// Resolve entities from a document graph
    DocumentResolve {
        /// Input text
        text: OsString,
        /// Source language (pl, en)
        #[arg(short, long, default_value = "pl")]
        lang: String,
        /// Data directory
        #[arg(short, long, default_value = "data")]
        data: String,
        /// Output format: json, pretty-json, summary
        #[arg(short, long, default_value = "json")]
        format: String,
    },
    /// Resolve temporal, event, and discourse structure
    DocumentTemporalDiscourse {
        /// Input text
        text: OsString,
        /// Source language (pl, en)
        #[arg(short, long, default_value = "pl")]
        lang: String,
        /// Data directory
        #[arg(short, long, default_value = "data")]
        data: String,
        /// Output format: json, pretty-json, summary, timeline, plan
        #[arg(short, long, default_value = "summary")]
        format: String,
    },
    /// Translate using the resolved document path
    DocumentTranslateResolved {
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
            base_url,
            model,
            system_prompt,
        } => {
            let trace_mode = match trace.as_str() {
                "off" => TraceMode::Off,
                "full" => TraceMode::Full,
                _ => TraceMode::Brief,
            };

            let options = ChatOptions {
                from,
                to,
                data_dir: data,
                offline,
                trace_mode,
                base_url,
                model,
                system_prompt,
            };

            if let Err(e) = lexflex::chat::run(options) {
                eprintln!("Chat UI error: {}", e);
                process::exit(1);
            }
        }
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
        Commands::DocumentGraph {
            text,
            lang,
            data,
            format,
        } => {
            let text = text.to_string_lossy().to_string();
            let api = LexFlexAPI::builder()
                .data_dir(&data)
                .build()
                .expect("Failed to initialize lexFlex");
            match api.compile_document_graph(&text, &lang) {
                Ok(graph) => match format.as_str() {
                    "pretty-json" => match graph.to_pretty_json() {
                        Ok(out) => println!("{}", out),
                        Err(e) => {
                            eprintln!("Document graph serialization error: {}", e);
                            std::process::exit(1);
                        }
                    },
                    "summary" => {
                        println!("{}", graph_summary_text(&graph));
                    }
                    _ => match graph.to_canonical_json() {
                        Ok(out) => println!("{}", out),
                        Err(e) => {
                            eprintln!("Document graph serialization error: {}", e);
                            std::process::exit(1);
                        }
                    },
                },
                Err(e) => {
                    eprintln!("Document graph error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::DocumentResolve {
            text,
            lang,
            data,
            format,
        } => {
            let text = text.to_string_lossy().to_string();
            let api = LexFlexAPI::builder()
                .data_dir(&data)
                .build()
                .expect("Failed to initialize lexFlex");
            match api.compile_resolved_document(&text, &lang) {
                Ok(resolution) => match format.as_str() {
                    "pretty-json" => match resolution.to_pretty_json() {
                        Ok(out) => println!("{}", out),
                        Err(e) => {
                            eprintln!("Document resolution serialization error: {}", e);
                            std::process::exit(1);
                        }
                    },
                    "summary" => {
                        println!("{}", resolution_summary_text(&resolution));
                    }
                    _ => match resolution.to_canonical_json() {
                        Ok(out) => println!("{}", out),
                        Err(e) => {
                            eprintln!("Document resolution serialization error: {}", e);
                            std::process::exit(1);
                        }
                    },
                },
                Err(e) => {
                    eprintln!("Document resolution error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::DocumentTemporalDiscourse {
            text,
            lang,
            data,
            format,
        } => {
            let text = text.to_string_lossy().to_string();
            let api = LexFlexAPI::builder()
                .data_dir(&data)
                .build()
                .expect("Failed to initialize lexFlex");
            match api.compile_document_temporal_discourse(&text, &lang) {
                Ok(artifact) => match format.as_str() {
                    "pretty-json" => match serde_json::to_string_pretty(&artifact) {
                        Ok(out) => println!("{}", out),
                        Err(e) => {
                            eprintln!("Temporal discourse serialization error: {}", e);
                            std::process::exit(1);
                        }
                    },
                    "json" => match serde_json::to_string(&artifact) {
                        Ok(out) => println!("{}", out),
                        Err(e) => {
                            eprintln!("Temporal discourse serialization error: {}", e);
                            std::process::exit(1);
                        }
                    },
                    "timeline" => {
                        println!("{}", temporal_discourse_timeline_text(&artifact));
                    }
                    "plan" => {
                        println!("{}", temporal_discourse_plan_text(&artifact));
                    }
                    _ => {
                        println!("{}", temporal_discourse_summary_text(&artifact));
                    }
                },
                Err(e) => {
                    eprintln!("Temporal discourse error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::DocumentTranslateResolved {
            text,
            from,
            to,
            data,
        } => {
            let text = text.to_string_lossy().to_string();
            let api = LexFlexAPI::builder()
                .data_dir(&data)
                .build()
                .expect("Failed to initialize lexFlex");
            match api.translate_document_resolved(&text, &from, &to) {
                Ok(result) => println!("{}", result.output),
                Err(e) => {
                    eprintln!("Resolved translation error: {}", e);
                    std::process::exit(1);
                }
            }
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

fn graph_summary_text(graph: &lexflex::document::graph::DocumentGraph) -> String {
    [
        format!("document_id: {}", graph.source_document_id),
        format!("graph_id: {}", graph.id),
        format!("schema_version: {}", graph.schema.schema_version),
        format!("algorithm_version: {}", graph.schema.algorithm_version),
        format!("nodes_total: {}", graph.summary.nodes_total),
        format!("edges_total: {}", graph.summary.edges_total),
        format!("source_sentence_nodes: {}", graph.summary.source_sentence_nodes),
        format!("semantic_sentence_nodes: {}", graph.summary.semantic_sentence_nodes),
        format!("frame_occurrence_nodes: {}", graph.summary.frame_occurrence_nodes),
        format!("mention_nodes: {}", graph.summary.mention_nodes),
        format!("entity_candidate_nodes: {}", graph.summary.entity_candidate_nodes),
        format!("event_nodes: {}", graph.summary.event_nodes),
        format!("unresolved_fragment_nodes: {}", graph.summary.unresolved_fragment_nodes),
        format!("graph_sha256: {}", graph.graph_sha256),
    ]
    .join("\n")
        + "\n"
}

fn resolution_summary_text(
    resolution: &lexflex::document::resolution::DocumentEntityResolution,
) -> String {
    [
        format!("resolution_id: {}", resolution.id),
        format!("source_document_id: {}", resolution.source_document_id),
        format!("source_graph_id: {}", resolution.source_graph_id),
        format!("mentions_total: {}", resolution.summary.mentions_total),
        format!(
            "synthetic_mentions_total: {}",
            resolution.summary.synthetic_mentions_total
        ),
        format!("decisions_total: {}", resolution.summary.decisions_total),
        format!("accepted: {}", resolution.summary.accepted),
        format!("hard_accepted: {}", resolution.summary.hard_accepted),
        format!("ambiguous: {}", resolution.summary.ambiguous),
        format!("deferred: {}", resolution.summary.deferred),
        format!("unresolved: {}", resolution.summary.unresolved),
        format!("clusters_total: {}", resolution.summary.clusters_total),
        format!("diagnostics_fatal: {}", resolution.summary.diagnostics_fatal),
        format!("resolution_sha256: {}", resolution.resolution_sha256),
    ]
    .join("\n")
        + "\n"
}

fn temporal_discourse_summary_text(artifact: &DocumentTemporalDiscourse) -> String {
    [
        format!("temporal_discourse_id: {}", artifact.id),
        format!("source_document_id: {}", artifact.source_document_id),
        format!("source_graph_id: {}", artifact.source_graph_id),
        format!("event_profiles: {}", artifact.summary.event_profiles_total),
        format!("temporal_expressions: {}", artifact.summary.temporal_expressions_total),
        format!("temporal_relations: {}", artifact.summary.temporal_relations_total),
        format!("event_clusters: {}", artifact.summary.event_clusters_total),
        format!("discourse_relations: {}", artifact.summary.discourse_relations_total),
        format!("temporal_discourse_sha256: {}", artifact.temporal_discourse_sha256),
    ]
    .join("\n")
        + "\n"
}

fn temporal_discourse_timeline_text(artifact: &DocumentTemporalDiscourse) -> String {
    let mut lines = vec![format!("artifact: {}", artifact.id)];
    for relation_id in &artifact.temporal_relation_order {
        if let Some(relation) = artifact.temporal_relations.get(relation_id) {
            lines.push(format!(
                "{} | {:?} | {} -> {}",
                relation.id, relation.kind, relation.from_event_id, relation.to_event_id
            ));
        }
    }
    lines.join("\n") + "\n"
}

fn temporal_discourse_plan_text(artifact: &DocumentTemporalDiscourse) -> String {
    let mut lines = vec![format!("artifact: {}", artifact.id)];
    if let Some(plan) = &artifact.generation_plan {
        lines.push(format!("entity_clusters: {}", plan.entity_cluster_refs.len()));
        for step in &plan.ordered_steps {
            lines.push(step.clone());
        }
    }
    lines.join("\n") + "\n"
}
