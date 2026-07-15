use super::types::EngineError;
use super::workspace::SessionWorkspace;
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone)]
pub struct SessionStore { root: PathBuf }

impl SessionStore {
    pub fn new(root: impl Into<PathBuf>) -> Self { Self { root: root.into() } }
    pub fn path_for(&self, session_id: &str) -> Result<PathBuf, EngineError> {
        if session_id.is_empty() || session_id.contains('/') || session_id.contains('\\') || session_id.contains("..") { return Err(EngineError::Persistence("invalid session id".into())); }
        let path = self.root.join("sessions").join(session_id).join("snapshots").join("current.json");
        if !is_safe_store_path(&path) { return Err(EngineError::Persistence("unsafe session path".into())); }
        Ok(path)
    }
    pub fn save(&self, workspace: &SessionWorkspace) -> Result<PathBuf, EngineError> {
        workspace.validate().map_err(EngineError::Persistence)?;
        let path = self.path_for(&workspace.session_id)?;
        let parent = path.parent().ok_or_else(|| EngineError::Persistence("invalid store path".into()))?;
        fs::create_dir_all(parent).map_err(|e| EngineError::Persistence(e.to_string()))?;
        let bytes = serde_json::to_vec_pretty(workspace).map_err(|e| EngineError::Persistence(e.to_string()))?;
        let session_dir = parent.parent().ok_or_else(|| EngineError::Persistence("invalid session path".into()))?;
        let sources_dir = session_dir.join("sources");
        let bundles_dir = session_dir.join("bundles");
        fs::create_dir_all(&sources_dir).map_err(|e| EngineError::Persistence(e.to_string()))?;
        fs::create_dir_all(&bundles_dir).map_err(|e| EngineError::Persistence(e.to_string()))?;
        for (source_id, source) in &workspace.sources {
            let source_path = safe_artifact_path(&sources_dir, source_id)?;
            let source_bytes = serde_json::to_vec_pretty(source).map_err(|e| EngineError::Persistence(e.to_string()))?;
            atomic_write(&source_path, &source_bytes)?;
        }
        for (bundle_id, bundle) in &workspace.bundles {
            let bundle_path = safe_artifact_path(&bundles_dir, bundle_id)?;
            let bundle_bytes = serde_json::to_vec_pretty(bundle).map_err(|e| EngineError::Persistence(e.to_string()))?;
            atomic_write(&bundle_path, &bundle_bytes)?;
        }
        let snapshot = parent.join(format!("{}.json", workspace.snapshot_id));
        atomic_write(&snapshot, &bytes)?;
        // `current.json` is only an index to the latest immutable snapshot.
        atomic_write(&path, workspace.snapshot_id.as_bytes())?;
        Ok(path)
    }
    pub fn load(&self, session_id: &str) -> Result<SessionWorkspace, EngineError> {
        let path = self.path_for(session_id)?;
        let snapshot_id = String::from_utf8(fs::read(&path).map_err(|e| EngineError::Persistence(e.to_string()))?)
            .map_err(|e| EngineError::Persistence(e.to_string()))?;
        if snapshot_id.is_empty() || snapshot_id.contains('/') || snapshot_id.contains('\\') || snapshot_id.contains("..") {
            return Err(EngineError::Persistence("invalid current snapshot id".into()));
        }
        let snapshot_path = path.parent().ok_or_else(|| EngineError::Persistence("invalid snapshot path".into()))?.join(format!("{snapshot_id}.json"));
        let bytes = fs::read(snapshot_path).map_err(|e| EngineError::Persistence(e.to_string()))?;
        let workspace: SessionWorkspace = serde_json::from_slice(&bytes).map_err(|e| EngineError::Persistence(e.to_string()))?;
        if workspace.session_id != session_id { return Err(EngineError::Persistence("session id mismatch".into())); }
        let session_dir = path.parent().and_then(Path::parent).ok_or_else(|| EngineError::Persistence("invalid session path".into()))?;
        for (source_id, expected) in &workspace.sources {
            let artifact = safe_artifact_path(&session_dir.join("sources"), source_id)?;
            let actual: super::types::SourceSnapshot = serde_json::from_slice(&fs::read(artifact).map_err(|e| EngineError::Persistence(e.to_string()))?).map_err(|e| EngineError::Persistence(e.to_string()))?;
            if &actual != expected { return Err(EngineError::Persistence(format!("source artifact mismatch: {source_id}"))); }
        }
        for (bundle_id, expected) in &workspace.bundles {
            let artifact = safe_artifact_path(&session_dir.join("bundles"), bundle_id)?;
            let actual: crate::runtime::DocumentArtifactBundle = serde_json::from_slice(&fs::read(artifact).map_err(|e| EngineError::Persistence(e.to_string()))?).map_err(|e| EngineError::Persistence(e.to_string()))?;
            if &actual != expected { return Err(EngineError::Persistence(format!("bundle artifact mismatch: {bundle_id}"))); }
        }
        workspace.validate().map_err(EngineError::Persistence)?;
        Ok(workspace)
    }
    pub fn save_trace(&self, session_id: &str, turn: &str, jsonl: &str) -> Result<PathBuf, EngineError> {
        if session_id.is_empty() || session_id.contains('/') || session_id.contains('\\') || session_id.contains("..") || turn.is_empty() || turn.contains('/') || turn.contains('\\') || turn.contains("..") { return Err(EngineError::Persistence("invalid trace path".into())); }
        let dir = self.root.join("sessions").join(session_id).join("traces");
        let path = dir.join(format!("{turn}.jsonl"));
        if !is_safe_store_path(&path) { return Err(EngineError::Persistence("unsafe trace path".into())); }
        fs::create_dir_all(&dir).map_err(|e| EngineError::Persistence(e.to_string()))?;
        let tmp = path.with_extension("jsonl.tmp");
        fs::write(&tmp, jsonl).map_err(|e| EngineError::Persistence(e.to_string()))?;
        fs::rename(&tmp, &path).map_err(|e| EngineError::Persistence(e.to_string()))?;
        Ok(path)
    }

    pub fn latest_trace(&self, session_id: &str) -> Result<(String, String), EngineError> {
        if session_id.is_empty() || session_id.contains('/') || session_id.contains('\\') || session_id.contains("..") {
            return Err(EngineError::Persistence("invalid trace path".into()));
        }
        let dir = self.root.join("sessions").join(session_id).join("traces");
        let mut entries = fs::read_dir(&dir)
            .map_err(|e| EngineError::Persistence(e.to_string()))?
            .filter_map(Result::ok)
            .filter(|entry| entry.path().extension().and_then(|value| value.to_str()) == Some("jsonl"))
            .collect::<Vec<_>>();
        let has_run_entries = entries.iter().any(|entry| {
            entry
                .path()
                .file_stem()
                .and_then(|value| value.to_str())
                .is_some_and(|name| name.starts_with("run:"))
        });
        if has_run_entries {
            entries.retain(|entry| {
                entry
                    .path()
                    .file_stem()
                    .and_then(|value| value.to_str())
                    .is_some_and(|name| name.starts_with("run:"))
            });
        }
        entries.sort_by_key(|entry| entry.file_name());
        let entry = entries.pop().ok_or_else(|| EngineError::Persistence("no trace available".into()))?;
        let run_id = entry.path().file_stem().and_then(|value| value.to_str()).unwrap_or_default().to_string();
        let trace = fs::read_to_string(entry.path()).map_err(|e| EngineError::Persistence(e.to_string()))?;
        Ok((run_id, trace))
    }
    pub fn load_trace(&self, session_id: &str, turn: &str) -> Result<String, EngineError> {
        if session_id.is_empty() || session_id.contains('/') || session_id.contains('\\') || session_id.contains("..") || turn.is_empty() || turn.contains('/') || turn.contains('\\') || turn.contains("..") { return Err(EngineError::Persistence("invalid trace path".into())); }
        let path = self.root.join("sessions").join(session_id).join("traces").join(format!("{turn}.jsonl"));
        if !is_safe_store_path(&path) { return Err(EngineError::Persistence("unsafe trace path".into())); }
        fs::read_to_string(path).map_err(|e| EngineError::Persistence(e.to_string()))
    }
}

fn safe_artifact_path(dir: &Path, id: &str) -> Result<PathBuf, EngineError> {
    if id.is_empty() || id.contains('/') || id.contains('\\') || id.contains("..") {
        return Err(EngineError::Persistence("invalid artifact id".into()));
    }
    let path = dir.join(format!("{id}.json"));
    if !is_safe_store_path(&path) { return Err(EngineError::Persistence("unsafe artifact path".into())); }
    Ok(path)
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), EngineError> {
    let tmp = path.with_extension(format!("{}.tmp", path.extension().and_then(|e| e.to_str()).unwrap_or("file")));
    fs::write(&tmp, bytes).map_err(|e| EngineError::Persistence(e.to_string()))?;
    fs::rename(&tmp, path).map_err(|e| EngineError::Persistence(e.to_string()))?;
    Ok(())
}

pub fn is_safe_store_path(path: &Path) -> bool { !path.components().any(|component| matches!(component, Component::ParentDir)) }
