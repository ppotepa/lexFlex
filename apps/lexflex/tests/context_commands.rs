use lexflex_documents::{ingest, DocumentStore};
use lexflex_model::SemanticExpression;
use lexflex_provider_llm::{FixtureProvider, KnowledgeProvider, ProviderBudget, ProviderRequest};
use serde_json::Value;
use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn app() -> Command {
    Command::new(env!("CARGO_BIN_EXE_lexflex-app"))
}

fn workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../")
}

fn temp_file(name: &str, contents: &str) -> std::path::PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("lexflex-{name}-{stamp}"));
    fs::write(&path, contents).expect("write fixture");
    path
}

#[test]
fn document_ingest_command_returns_provenance_json() {
    let path = temp_file("document", "Paris\nFrance");
    let output = app()
        .args(["document-ingest", "doc-1", path.to_str().expect("path")])
        .output()
        .expect("launch");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(json["document_id"], "doc-1");
    assert_eq!(json["segments"].as_array().map(Vec::len), Some(2));
    let _ = fs::remove_file(path);
}

#[test]
fn conversation_turn_command_returns_verified_state() {
    let path = temp_file("expression", r#"{"Value":{"Boolean":true}}"#);
    let output = app()
        .args(["conversation-turn", "turn-1", path.to_str().expect("path")])
        .output()
        .expect("launch");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(json["turns"].as_array().map(Vec::len), Some(1));
    let _ = fs::remove_file(path);
}

#[test]
fn context_commands_persist_and_reload_verified_state() {
    let document = temp_file("document-persist", "Paris");
    let store = temp_file("document-store", "");
    fs::remove_file(&store).expect("remove empty store fixture");
    let first = app()
        .args([
            "document-ingest",
            "doc-1",
            document.to_str().expect("path"),
            "--store",
            store.to_str().expect("path"),
        ])
        .output()
        .expect("launch");
    assert!(first.status.success());
    assert!(fs::metadata(&store).is_ok());

    let expression = temp_file("conversation-expression", r#"{"Value":{"Boolean":true}}"#);
    let state = temp_file("conversation-state", "");
    fs::remove_file(&state).expect("remove empty state fixture");
    for turn in ["turn-1", "turn-2"] {
        let output = app()
            .args([
                "conversation-turn",
                turn,
                expression.to_str().expect("path"),
                "--state",
                state.to_str().expect("path"),
            ])
            .output()
            .expect("launch");
        assert!(output.status.success());
    }
    let persisted: Value =
        serde_json::from_slice(&fs::read(&state).expect("state")).expect("state JSON");
    assert_eq!(persisted["turns"].as_array().map(Vec::len), Some(2));
    for path in [document, store, expression, state] {
        let _ = fs::remove_file(path);
    }
}

#[test]
fn document_remove_and_inspect_commands_use_verified_store() {
    let document = temp_file("document-remove", "Paris");
    let store = temp_file("document-remove-store", "");
    let document_arg = document.to_str().expect("document path");
    let store_arg = store.to_str().expect("store path");
    fs::remove_file(&store).expect("remove empty store fixture");
    let ingest = Command::new(env!("CARGO_BIN_EXE_lexflex-app"))
        .current_dir(workspace_root())
        .args([
            "document-ingest",
            "doc-remove",
            document_arg,
            "--store",
            store_arg,
        ])
        .output()
        .expect("ingest");
    assert!(ingest.status.success());

    let inspected = Command::new(env!("CARGO_BIN_EXE_lexflex-app"))
        .current_dir(workspace_root())
        .args(["document-inspect", "--store", store_arg])
        .output()
        .expect("inspect");
    assert!(inspected.status.success());
    let value: Value = serde_json::from_slice(&inspected.stdout).expect("inspect JSON");
    assert_eq!(value.as_array().map(Vec::len), Some(1));

    let removed = Command::new(env!("CARGO_BIN_EXE_lexflex-app"))
        .current_dir(workspace_root())
        .args(["document-remove", "doc-remove", "--store", store_arg])
        .output()
        .expect("remove");
    assert!(removed.status.success());
    let _ = fs::remove_file(document);
    let _ = fs::remove_file(store);
}

#[test]
fn document_replace_command_refreshes_verified_provenance() {
    let original = temp_file("document-replace-original", "Paris");
    let replacement = temp_file("document-replace-new", "France\nEurope");
    let store = temp_file("document-replace-store", "");
    fs::remove_file(&store).expect("remove empty store fixture");
    let ingest = app()
        .current_dir(workspace_root())
        .args([
            "document-ingest",
            "doc-replace",
            original.to_str().expect("original path"),
            "--store",
            store.to_str().expect("store path"),
        ])
        .output()
        .expect("ingest");
    assert!(ingest.status.success());
    let replaced = app()
        .current_dir(workspace_root())
        .args([
            "document-replace",
            "doc-replace",
            replacement.to_str().expect("replacement path"),
            "--store",
            store.to_str().expect("store path"),
        ])
        .output()
        .expect("replace");
    assert!(
        replaced.status.success(),
        "{}",
        String::from_utf8_lossy(&replaced.stderr)
    );
    let value: Value = serde_json::from_slice(&replaced.stdout).expect("document JSON");
    assert_eq!(value["source_text"], "France\nEurope");
    assert_eq!(value["segments"].as_array().map(Vec::len), Some(2));
    let _ = fs::remove_file(original);
    let _ = fs::remove_file(replacement);
    let _ = fs::remove_file(store);
}

#[test]
fn document_query_returns_matching_segment_provenance() {
    let document_path = temp_file("document-query-source", "Paris");
    let store = temp_file("document-query-store", "");
    fs::remove_file(&store).expect("remove empty store fixture");
    let mut documents = DocumentStore::default();
    documents
        .insert(ingest("doc-query".into(), "Paris").expect("ingest"))
        .expect("insert");
    documents
        .attach_analysis(
            "doc-query",
            "doc-query:segment:0",
            SemanticExpression::Value(true.into()),
        )
        .expect("analysis");
    documents.save_to_file(&store).expect("save");
    let expression = temp_file("document-query-expression", r#"{"Value":{"Boolean":true}}"#);
    let output = app()
        .current_dir(workspace_root())
        .args([
            "document-query",
            expression.to_str().expect("expression path"),
            "--store",
            store.to_str().expect("store path"),
        ])
        .output()
        .expect("query");
    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).expect("query JSON");
    assert_eq!(value[0]["document_id"], "doc-query");
    assert_eq!(value[0]["segment_id"], "doc-query:segment:0");
    assert_eq!(value[0]["text"], "Paris");
    for path in [document_path, store, expression] {
        let _ = fs::remove_file(path);
    }
}

#[test]
fn document_analyze_commits_all_segment_analyses_atomically() {
    let document = temp_file("document-analyze", "Paris is the capital of France.");
    let store = temp_file("document-analyze-store", "");
    fs::remove_file(&store).expect("remove empty store fixture");
    let ingest = app()
        .current_dir(workspace_root())
        .args([
            "document-ingest",
            "doc-analyze",
            document.to_str().expect("document path"),
            "--store",
            store.to_str().expect("store path"),
        ])
        .output()
        .expect("ingest");
    assert!(ingest.status.success());
    let analyzed = app()
        .current_dir(workspace_root())
        .args([
            "document-analyze",
            "--store",
            store.to_str().expect("store path"),
            "--language",
            "en",
        ])
        .output()
        .expect("analyze");
    assert!(
        analyzed.status.success(),
        "{}",
        String::from_utf8_lossy(&analyzed.stderr)
    );
    let value: Value = serde_json::from_slice(&analyzed.stdout).expect("analyzed JSON");
    assert!(value["documents"]["doc-analyze"].is_object());
    let persisted: Value =
        serde_json::from_slice(&fs::read(&store).expect("store")).expect("store JSON");
    assert!(persisted["documents"]["doc-analyze"]["segments"][0]["analysis"].is_object());
    let _ = fs::remove_file(document);
    let _ = fs::remove_file(store);
}

#[test]
fn conversation_inspect_command_reads_verified_state() {
    let expression = temp_file(
        "conversation-inspect-expression",
        r#"{"Value":{"Boolean":true}}"#,
    );
    let state = temp_file("conversation-inspect-state", "");
    fs::remove_file(&state).expect("remove empty state fixture");
    let expression_arg = expression.to_str().expect("expression path");
    let state_arg = state.to_str().expect("state path");
    let turn = app()
        .current_dir(workspace_root())
        .args([
            "conversation-turn",
            "turn-inspect",
            expression_arg,
            "--state",
            state_arg,
        ])
        .output()
        .expect("turn");
    assert!(turn.status.success());
    let inspected = app()
        .current_dir(workspace_root())
        .args(["conversation-inspect", "--state", state_arg])
        .output()
        .expect("inspect");
    assert!(inspected.status.success());
    let value: Value = serde_json::from_slice(&inspected.stdout).expect("state JSON");
    assert_eq!(value["turns"].as_array().map(Vec::len), Some(1));
    let _ = fs::remove_file(expression);
    let _ = fs::remove_file(state);
}

#[test]
fn conversation_text_turn_parses_before_persisting_state() {
    let state = temp_file("conversation-text-state", "");
    fs::remove_file(&state).expect("remove empty state fixture");
    let output = app()
        .current_dir(workspace_root())
        .args([
            "conversation-text-turn",
            "turn-text",
            "--language",
            "en",
            "--text",
            "Paris is the capital of France.",
            "--state",
            state.to_str().expect("state path"),
        ])
        .output()
        .expect("text turn");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout).expect("state JSON");
    assert_eq!(value["turns"].as_array().map(Vec::len), Some(1));
    assert!(value["turns"][0]["expression"].is_object());
    let persisted: Value =
        serde_json::from_slice(&fs::read(&state).expect("state")).expect("persisted JSON");
    assert_eq!(persisted["turns"].as_array().map(Vec::len), Some(1));
    let _ = fs::remove_file(state);
}

#[test]
fn provider_ingest_command_verifies_and_commits_artifact() {
    let provider = FixtureProvider::new(ProviderBudget {
        max_prompt_bytes: 64,
        max_response_bytes: 64,
    });
    let artifact = provider
        .fetch(ProviderRequest {
            prompt: "Paris is the capital of France.".into(),
            model: "fixture".into(),
            prompt_version: "v1".into(),
        })
        .expect("artifact");
    let artifact_path = temp_file(
        "provider-artifact",
        &serde_json::to_string(&artifact).expect("artifact JSON"),
    );
    let store = temp_file("provider-store", "");
    fs::remove_file(&store).expect("remove empty store fixture");
    let output = app()
        .current_dir(workspace_root())
        .args([
            "provider-ingest",
            "provider-doc",
            artifact_path.to_str().expect("artifact path"),
            "--store",
            store.to_str().expect("store path"),
        ])
        .output()
        .expect("provider ingest");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout).expect("document JSON");
    assert_eq!(value["document_id"], "provider-doc");
    let persisted: Value =
        serde_json::from_slice(&fs::read(&store).expect("store")).expect("store JSON");
    assert!(persisted["documents"]["provider-doc"].is_object());
    let _ = fs::remove_file(artifact_path);
    let _ = fs::remove_file(store);
}

#[test]
fn conversation_resolve_command_preserves_ambiguity() {
    let state = temp_file(
        "conversation-resolve-state",
        r#"{"turns":[{"turn_id":"t1","expression":{"Value":{"Boolean":true}}},{"turn_id":"t2","expression":{"Value":{"Boolean":true}}}],"mentions":[{"entity":"entity:paris","turn_id":"t1","salience":4},{"entity":"entity:london","turn_id":"t2","salience":4}]}"#,
    );
    let output = app()
        .current_dir(workspace_root())
        .args([
            "conversation-resolve",
            "entity:",
            "--state",
            state.to_str().expect("state path"),
        ])
        .output()
        .expect("resolve");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout).expect("resolution JSON");
    assert!(value.get("Ambiguous").is_some());
    let _ = fs::remove_file(state);
}
