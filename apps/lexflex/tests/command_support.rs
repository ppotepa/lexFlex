use std::process::Command;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LevelCaseResult {
    pub id: String,
    pub source_language: String,
    pub target_language: String,
    pub source: String,
    pub expected: String,
    pub actual: String,
    pub semantic_hash_equal: bool,
    pub passed: bool,
    pub error: Option<String>,
}

#[allow(dead_code)]
pub fn write_level_report(level: &str, results: &[LevelCaseResult]) {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/lexflex-reports");
    std::fs::create_dir_all(&root).expect("create level report directory");

    let passed = results.iter().filter(|result| result.passed).count();
    let mut markdown = format!(
        "# {level} Translation Report\n\nPassed: {passed}/{}\n\n| ID | Source | Expected | Actual | Semantic hash | Status |\n| --- | --- | --- | --- | --- | --- |\n",
        results.len()
    );
    for result in results {
        let status = if result.passed { "PASS" } else { "FAIL" };
        markdown.push_str(&format!(
            "| {} | `{}` | `{}` | `{}` | {} | {} |\n",
            result.id,
            result.source.replace('|', "\\|"),
            result.expected.replace('|', "\\|"),
            result.actual.replace('|', "\\|"),
            result.semantic_hash_equal,
            status,
        ));
    }
    std::fs::write(root.join(format!("{level}.md")), markdown)
        .expect("write level markdown report");
    std::fs::write(
        root.join(format!("{level}.json")),
        serde_json::to_vec_pretty(&results_to_json(results)).expect("serialize level report"),
    )
    .expect("write level JSON report");
}

#[allow(dead_code)]
pub fn assert_level_passed(level: &str, results: &[LevelCaseResult]) {
    let failures = results
        .iter()
        .filter(|result| !result.passed)
        .map(|result| {
            format!(
                "{}: {}",
                result.id,
                result.error.as_deref().unwrap_or("unknown")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        failures.is_empty(),
        "{level} has {} failed cases:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[allow(dead_code)]
fn results_to_json(results: &[LevelCaseResult]) -> serde_json::Value {
    serde_json::json!({
        "cases": results.iter().map(|result| serde_json::json!({
            "id": result.id,
            "source_language": result.source_language,
            "target_language": result.target_language,
            "source": result.source,
            "expected": result.expected,
            "actual": result.actual,
            "semantic_hash_equal": result.semantic_hash_equal,
            "passed": result.passed,
            "error": result.error,
        })).collect::<Vec<_>>(),
        "total": results.len(),
        "passed": results.iter().filter(|result| result.passed).count(),
        "failed": results.iter().filter(|result| !result.passed).count(),
    })
}

pub struct CommandOutput {
    pub code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

pub fn run(args: &[&str]) -> CommandOutput {
    let output = Command::new(env!("CARGO_BIN_EXE_lexflex-app"))
        .args(args)
        .current_dir(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../"))
        .output()
        .expect("launch lexflex-app");

    CommandOutput {
        code: output.status.code(),
        stdout: output.stdout,
        stderr: output.stderr,
    }
}

pub fn run_with_stdin(args: &[&str], input: &str) -> CommandOutput {
    use std::io::Write;
    use std::process::Stdio;

    let mut command = Command::new(env!("CARGO_BIN_EXE_lexflex-app"));
    let mut child = command
        .args(args)
        .current_dir(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("launch lexflex-app");
    child
        .stdin
        .as_mut()
        .expect("stdin pipe")
        .write_all(input.as_bytes())
        .expect("write stdin");
    let output = child.wait_with_output().expect("wait for lexflex-app");
    CommandOutput {
        code: output.status.code(),
        stdout: output.stdout,
        stderr: output.stderr,
    }
}
