use clap::Parser;
use lexflex::api::LexFlexAPI;
use lexflex::core::graph::GraphSnapshot;
use lexflex::core::interlingua::Interlingua;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser)]
#[command(name = "benchmark")]
#[command(about = "lexFlex translation benchmark")]
struct Cli {
    /// Input file with sentences (one per line: PL->EN or EN->PL followed by sentence)
    #[arg(short, long)]
    input: String,
    /// Data directory
    #[arg(short, long, default_value = "data")]
    data: String,
}

fn main() {
    let cli = Cli::parse();

    let api = LexFlexAPI::builder()
        .data_dir(&cli.data)
        .build()
        .expect("Failed to initialize lexFlex");

    let content = fs::read_to_string(&cli.input).expect("Failed to read input file");
    let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();

    // === Per-query Node Graph / Trace support ===
    // When running benchmarks, every sentence now produces a detailed trace:
    // - The parsed Interlingua (the semantic "node graph")
    // - Key decisions from the pipeline
    // - Final output + why it may have succeeded or failed
    // This is saved to results/runs/ so we can fully analyze and draw conclusions.
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let input_stem = std::path::Path::new(&cli.input)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("benchmark");
    let run_dir: PathBuf = format!("results/runs/{}-{}-trace", input_stem, timestamp).into();
    fs::create_dir_all(&run_dir).expect("Failed to create trace directory");
    let trace_path = run_dir.join("detailed_traces.txt");
    let mut trace_file = fs::File::create(&trace_path).expect("Failed to open trace file");

    use std::io::Write;
    let _ = writeln!(trace_file, "# lexFlex Detailed Trace + Node Graph per query");
    let _ = writeln!(trace_file, "# Input file: {}", cli.input);
    let _ = writeln!(trace_file, "# Run directory: {}", run_dir.display());
    let _ = writeln!(trace_file, "# The Interlingua dump below is the primary semantic node graph for analysis.\n");

    let mut results = Vec::new();
    let mut pl_en_ok = 0;
    let mut pl_en_fail = 0;
    let mut en_pl_ok = 0;
    let mut en_pl_fail = 0;

    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                        lexFlex Benchmark Results                             ║");
    println!("╠══════════════════════════════════════════════════════════════════════════════╣");
    println!("║  {:<4}  {:<7}  {:<35}  {:<35}  {}  ║", "#", "Dir", "Source", "Output", "Status");
    println!("╠══════════════════════════════════════════════════════════════════════════════╣");

    for (i, line) in lines.iter().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let (direction, sentence) = if let Some(rest) = line.strip_prefix("PL->EN:") {
            ("PL->EN", rest.trim())
        } else if let Some(rest) = line.strip_prefix("PL->EN ") {
            ("PL->EN", rest)
        } else if let Some(rest) = line.strip_prefix("EN->PL:") {
            ("EN->PL", rest.trim())
        } else if let Some(rest) = line.strip_prefix("EN->PL ") {
            ("EN->PL", rest)
        } else {
            continue;
        };

        let (from, to) = match direction {
            "PL->EN" => ("pl", "en"),
            "EN->PL" => ("en", "pl"),
            _ => continue,
        };

        // Clear per-query trace collector (thread local instrumentation)
        lexflex::generation::pipeline::TRACE.with(|t| t.borrow_mut().clear());

        // Always parse first so we have the full semantic node graph (Interlingua) for analysis
        let il_result = api.parse(sentence, from);
        let parse_graph = match &il_result {
            Ok(il) => {
                // Prefer RON (project data format) for exact, queryable node graph
                ron::ser::to_string_pretty(il, ron::ser::PrettyConfig::default())
                    .unwrap_or_else(|_| format!("{:#?}", il))
            }
            Err(e) => format!("PARSE FAILED: {}", e),
        };

        let result = api.translate(sentence, from, to);

        // Capture real steps recorded during the generate call (instrumented in pipeline/resolve/realize)
        let collected_steps = lexflex::generation::pipeline::TRACE.with(|t| t.borrow().clone());

        let (output, status) = match &result {
            Ok(translated) => {
                if direction == "PL->EN" {
                    pl_en_ok += 1;
                } else {
                    en_pl_ok += 1;
                }
                (translated.clone(), "✓")
            }
            Err(e) => {
                if direction == "PL->EN" {
                    pl_en_fail += 1;
                } else {
                    en_pl_fail += 1;
                }
                (format!("ERROR: {}", e), "✗")
            }
        };
        // No cheats: write real output verbatim (per strategy to observe real api.translate)

        // Write rich trace for this query (the "node graph")
        let _ = writeln!(trace_file, "═══════════════════════════════════════════════════════════════");
        let _ = writeln!(trace_file, "#{}  {}  {}", i + 1, direction, sentence);
        let _ = writeln!(trace_file, "───────────────────────────────────────────────────────────────");
        let _ = writeln!(trace_file, "PARSED INTERLINGUA (semantic node graph, RON):\n{}", parse_graph);
        let _ = writeln!(trace_file, "───────────────────────────────────────────────────────────────");
        if let Ok(Interlingua::Natural(utt)) = &il_result {
            if let Some(s) = utt.sentences.first() {
                if let Some(ref g) = s.graph {
                    let snapshot = GraphSnapshot::from(g);
                    let graph_ron = ron::ser::to_string_pretty(&snapshot, ron::ser::PrettyConfig::default())
                        .unwrap_or_else(|_| format!("{:#?}", snapshot));
                    let _ = writeln!(trace_file, "LINGUISTIC GRAPH (surface + semantic nodes, RON):\n{}", graph_ron);
                    let _ = writeln!(trace_file, "───────────────────────────────────────────────────────────────");
                }
            }
        }
        // Real steps from instrumentation (generate_frame, resolve_surface_verb, realize_* calls via thread local)
        let mut step_lines = vec![];
        for st in &collected_steps {
            step_lines.push(format!(
                " - [{}] {} (reason: {:?}, nodes: {:?}, edges: {:?})",
                st.stage, st.decision, st.reason, st.involved_nodes, st.involved_edges
            ));
        }
        if step_lines.is_empty() {
            step_lines.push(" - (no steps recorded; collector active in pipeline path)".to_string());
        }
        // Also extract from IL for the node graph view
        if let Ok(Interlingua::Natural(utt)) = &il_result {
            for s in &utt.sentences {
                for f in &s.frames {
                    step_lines.push(format!(" - IL frame: {:?}", std::mem::discriminant(f)));
                }
                if let Some(q) = &s.quantification {
                    step_lines.push(format!(" - IL quant: {:?}", q));
                }
            }
        }
        let _ = writeln!(trace_file, "DECISION STEPS (instrumented):\n{}", step_lines.join("\n"));
        let _ = writeln!(trace_file, "───────────────────────────────────────────────────────────────");
        let _ = writeln!(trace_file, "OUTPUT: {}", output);
        let _ = writeln!(trace_file, "STATUS: {}", status);
        let _ = writeln!(trace_file, "═══════════════════════════════════════════════════════════════\n");

        let trunc_source = if sentence.chars().count() > 33 {
            let truncated: String = sentence.chars().take(30).collect();
            format!("{}...", truncated)
        } else {
            sentence.to_string()
        };
        let trunc_output = if output.chars().count() > 33 {
            let truncated: String = output.chars().take(30).collect();
            format!("{}...", truncated)
        } else {
            output.clone()
        };

        println!("║  {:<4}  {:<7}  {:<35}  {:<35}  {}  ║",
            i + 1, direction, trunc_source, trunc_output, status);

        results.push((i + 1, direction.to_string(), sentence.to_string(), output, status.to_string()));
    }

    let pl_en_total = pl_en_ok + pl_en_fail;
    let en_pl_total = en_pl_ok + en_pl_fail;
    let total = pl_en_total + en_pl_total;
    let total_ok = pl_en_ok + en_pl_ok;

    println!("╠══════════════════════════════════════════════════════════════════════════════╣");
    println!("║  Summary                                                                     ║");
    println!("╠══════════════════════════════════════════════════════════════════════════════╣");
    println!("║  PL->EN: {}/{} ({:.0}%)                                                       ║", pl_en_ok, pl_en_total, if pl_en_total > 0 { pl_en_ok as f64 / pl_en_total as f64 * 100.0 } else { 0.0 });
    println!("║  EN->PL: {}/{} ({:.0}%)                                                       ║", en_pl_ok, en_pl_total, if en_pl_total > 0 { en_pl_ok as f64 / en_pl_total as f64 * 100.0 } else { 0.0 });
    println!("║  Total:  {}/{} ({:.0}%)                                                       ║", total_ok, total, if total > 0 { total_ok as f64 / total as f64 * 100.0 } else { 0.0 });
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");

    // Finalize traces
    let _ = writeln!(trace_file, "\n# Summary");
    let _ = writeln!(trace_file, "PL->EN: {}/{} ({:.0}%)", pl_en_ok, pl_en_total, if pl_en_total > 0 { pl_en_ok as f64 / pl_en_total as f64 * 100.0 } else { 0.0 });
    let _ = writeln!(trace_file, "EN->PL: {}/{} ({:.0}%)", en_pl_ok, en_pl_total, if en_pl_total > 0 { en_pl_ok as f64 / en_pl_total as f64 * 100.0 } else { 0.0 });
    let _ = writeln!(trace_file, "Total:  {}/{} ({:.0}%)", total_ok, total, if total > 0 { total_ok as f64 / total as f64 * 100.0 } else { 0.0 });
    let _ = writeln!(trace_file, "\n# Detailed node graphs and traces saved to: {}", run_dir.display());
    let _ = writeln!(trace_file, "# Use this to fully analyze what the parser produced as Interlingua and why generation produced the surface form.");

    println!();
    println!("Detailed traces + Interlingua node graphs saved to: {}", run_dir.display());
    println!("See {}/detailed_traces.txt for per-sentence semantic graphs and decisions.", run_dir.display());
}
