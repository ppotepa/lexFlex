use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityLimits {
    pub max_json_depth: usize,
    pub max_path_components: usize,
}

impl Default for SecurityLimits {
    fn default() -> Self {
        Self {
            max_json_depth: 64,
            max_path_components: 32,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityPolicy {
    pub offline_only: bool,
    pub redact_source_text: bool,
    pub limits: SecurityLimits,
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            offline_only: true,
            redact_source_text: true,
            limits: SecurityLimits::default(),
        }
    }
}

pub fn validate_relative_path(path: &Path, max_components: usize) -> Result<(), String> {
    if path.is_absolute() {
        return Err("absolute paths are not allowed".into());
    }
    let mut components = 0usize;
    for component in path.components() {
        match component {
            Component::CurDir | Component::Normal(_) => {
                components += 1;
            }
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err("path traversal detected".into());
            }
        }
    }
    if components > max_components {
        return Err("too many path components".into());
    }
    Ok(())
}

pub fn ensure_within_root(root: &Path, path: &Path) -> Result<PathBuf, String> {
    validate_relative_path(path, 32)?;
    let joined = root.join(path);
    if !joined.starts_with(root) {
        return Err("path escaped root".into());
    }
    Ok(joined)
}

pub fn redact_text(text: &str, enabled: bool) -> String {
    if enabled {
        "[redacted]".into()
    } else {
        text.to_string()
    }
}
