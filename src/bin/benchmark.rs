use clap::Parser;
use lexflex::api::LexFlexAPI;
use std::fs;

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

        let (direction, sentence) = if let Some(rest) = line.strip_prefix("PL->EN ") {
            ("PL->EN", rest)
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

        let result = api.translate(sentence, from, to);

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
}
