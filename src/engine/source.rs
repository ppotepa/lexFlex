use super::types::*;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

pub trait SourceProvider: Send + Sync {
    fn resolve(&self, request: &SourceRequest) -> Result<SourceSnapshot, EngineError>;
}

#[derive(Debug, Clone)]
pub struct LocalSnapshotSourceProvider { root: PathBuf, offline: bool }

impl LocalSnapshotSourceProvider {
    fn kind_name(kind: SourceKind) -> &'static str {
        match kind {
            SourceKind::Wikipedia => "wikipedia",
            SourceKind::File => "file",
            SourceKind::Inline => "inline",
        }
    }
    pub fn new(root: impl Into<PathBuf>, offline: bool) -> Self { Self { root: root.into(), offline } }
    fn slug(title: &str) -> String {
        title.trim().chars().map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '_' }).collect()
    }
    fn hash(text: &str) -> String { let mut h = Sha256::new(); h.update(text.as_bytes()); format!("{:x}", h.finalize()) }
    fn candidate(&self, request: &SourceRequest) -> PathBuf {
        self.root.join("sources").join(match request.source_kind { SourceKind::Wikipedia => "wikipedia", SourceKind::File | SourceKind::Inline => "files" }).join(&request.language.0).join(Self::slug(&request.title))
    }
}

impl SourceProvider for LocalSnapshotSourceProvider {
    fn resolve(&self, request: &SourceRequest) -> Result<SourceSnapshot, EngineError> {
        if matches!(request.policy, SourceFetchPolicy::Live) && self.offline {
            return Err(EngineError::SourceUnavailable("live source access disabled in offline mode".into()));
        }
        let dir = self.candidate(request);
        let entries = fs::read_dir(&dir);
        if entries.is_err() {
            if matches!(request.policy, SourceFetchPolicy::Live | SourceFetchPolicy::CacheFirst) && !self.offline && request.source_kind == SourceKind::Wikipedia {
                let snapshot = self.fetch_wikipedia(request)?;
                self.cache_snapshot(request, &snapshot)?;
                return Ok(snapshot);
            }
            return Err(EngineError::SourceUnavailable(format!("no local snapshot for {}", request.title)));
        }
        let entries = entries.map_err(|e| EngineError::Source(e.to_string()))?;
        let mut files = entries.map(|entry| entry.map(|value| value.path()).map_err(|error| EngineError::Source(error.to_string()))).collect::<Result<Vec<_>, _>>()?;
        files.retain(|path| path.extension().and_then(|x| x.to_str()) == Some("json"));
        files.sort();
        let Some(path) = files.pop() else {
            if matches!(request.policy, SourceFetchPolicy::Live | SourceFetchPolicy::CacheFirst) && !self.offline && request.source_kind == SourceKind::Wikipedia {
                let snapshot = self.fetch_wikipedia(request)?;
                self.cache_snapshot(request, &snapshot)?;
                return Ok(snapshot);
            }
            return Err(EngineError::SourceUnavailable(format!("no snapshot for {}", request.title)));
        };
        let raw = fs::read(&path).map_err(|e| EngineError::Source(e.to_string()))?;
        let mut snapshot: SourceSnapshot = serde_json::from_slice(&raw).map_err(|e| EngineError::Source(e.to_string()))?;
        let actual = Self::hash(&snapshot.text);
        if actual != snapshot.content_sha256 { return Err(EngineError::Source("snapshot content hash mismatch".into())); }
        snapshot.source_id = format!("source:{}:{}:{}", Self::kind_name(request.source_kind), request.language, Self::slug(&request.title));
        Ok(snapshot)
    }
}

impl LocalSnapshotSourceProvider {
    fn cache_snapshot(&self, request: &SourceRequest, snapshot: &SourceSnapshot) -> Result<(), EngineError> {
        let dir = self.candidate(request);
        fs::create_dir_all(&dir).map_err(|e| EngineError::Source(e.to_string()))?;
        let revision = snapshot.revision.as_deref().unwrap_or(&snapshot.content_sha256);
        let path = dir.join(format!("{revision}.json"));
        let tmp = path.with_extension("json.tmp");
        let bytes = serde_json::to_vec_pretty(snapshot).map_err(|e| EngineError::Source(e.to_string()))?;
        fs::write(&tmp, bytes).map_err(|e| EngineError::Source(e.to_string()))?;
        fs::rename(tmp, path).map_err(|e| EngineError::Source(e.to_string()))?;
        Ok(())
    }

    fn fetch_wikipedia(&self, request: &SourceRequest) -> Result<SourceSnapshot, EngineError> {
        let lang = match request.language.0.as_str() { "pl" | "en" => request.language.0.clone(), _ => return Err(EngineError::Source("unsupported Wikipedia language".into())) };
        let encoded = urlencoding::encode(&request.title);
        let url = format!("https://{lang}.wikipedia.org/w/api.php?action=query&prop=extracts|revisions&explaintext=1&exsectionformat=plain&titles={encoded}&redirects=1&format=json");
        let runtime = tokio::runtime::Runtime::new().map_err(|e| EngineError::Source(e.to_string()))?;
        let (title, revision, text) = runtime.block_on(async {
            let payload: serde_json::Value = reqwest::Client::new().get(&url).header("User-Agent", "lexflex-engine/0.1").send().await.map_err(|e| EngineError::Source(e.to_string()))?.error_for_status().map_err(|e| EngineError::Source(e.to_string()))?.json().await.map_err(|e| EngineError::Source(e.to_string()))?;
            let page = payload["query"]["pages"].as_object().and_then(|pages| pages.values().next()).ok_or_else(|| EngineError::SourceUnavailable(format!("Wikipedia article not found: {}", request.title)))?;
            let title = page["title"].as_str().unwrap_or(&request.title).to_string();
            let revision = page["revisions"].as_array().and_then(|items| items.first()).and_then(|item| item["revid"].as_i64()).map(|value| value.to_string());
            let text = page["extract"].as_str().filter(|value| !value.trim().is_empty()).ok_or_else(|| EngineError::SourceUnavailable(format!("Wikipedia article has no text: {}", request.title)))?.to_string();
            Ok::<_, EngineError>((title, revision, text))
        })?;
        let mut snapshot = source_snapshot_from_text(request, text);
        snapshot.title = title.clone();
        snapshot.revision = revision.clone();
        snapshot.uri = Some(format!("https://{lang}.wikipedia.org/wiki/{}", urlencoding::encode(&title).replace("%20", "_")));
        snapshot.source_id = format!("source:{}:{}:{}", Self::kind_name(request.source_kind), request.language, Self::slug(&request.title));
        Ok(snapshot)
    }
}

pub fn source_snapshot_from_text(request: &SourceRequest, text: String) -> SourceSnapshot {
    let mut h = Sha256::new(); h.update(text.as_bytes());
    let hash = format!("{:x}", h.finalize());
    SourceSnapshot { source_id: format!("source:{}:{}:{}", LocalSnapshotSourceProvider::kind_name(request.source_kind), request.language, LocalSnapshotSourceProvider::slug(&request.title)), title: request.title.clone(), language: request.language.clone(), uri: None, revision: None, fetched_at: None, content_sha256: hash, text }
}

#[allow(dead_code)]
fn _safe_path(path: &Path) -> bool { !path.components().any(|c| matches!(c, std::path::Component::ParentDir)) }
