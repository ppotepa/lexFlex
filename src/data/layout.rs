use std::env;
use std::path::{Path, PathBuf};

use crate::error::DataError;

const REQUIRED_ASSETS: &[(&str, &str)] = &[
    ("concept definitions", "concepts/concepts.ron"),
    ("ontology", "ontology/ontology.ron"),
    ("English descriptor", "descriptors/en.ron"),
    ("Polish descriptor", "descriptors/pl.ron"),
    ("English lexicon", "lexicons/en/lexicon.ron"),
    ("Polish lexicon", "lexicons/pl/lexicon.ron"),
    ("English noun paradigms", "morphology/en/noun_paradigms.ron"),
    ("English verb paradigms", "morphology/en/verb_paradigms.ron"),
    ("Polish adjective paradigms", "morphology/pl/adj_paradigms.ron"),
    ("Polish noun paradigms", "morphology/pl/noun_paradigms.ron"),
    ("Polish verb paradigms", "morphology/pl/verb_paradigms.ron"),
];

#[derive(Debug, Clone)]
pub struct ResolvedDataRoot {
    pub path: PathBuf,
    pub source: DataRootSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataRootSource {
    Explicit,
    Environment,
    WorkspaceDiscovery,
    ManifestDir,
}

pub fn resolve_data_root(explicit: Option<&str>) -> Result<ResolvedDataRoot, DataError> {
    if let Some(value) = explicit {
        let path = canonicalize_existing(Path::new(value))?;
        validate_data_root(&path)?;
        return Ok(ResolvedDataRoot {
            path,
            source: DataRootSource::Explicit,
        });
    }

    if let Ok(value) = env::var("LEXFLEX_DATA_DIR") {
        let path = canonicalize_existing(Path::new(&value))?;
        validate_data_root(&path)?;
        return Ok(ResolvedDataRoot {
            path,
            source: DataRootSource::Environment,
        });
    }

    if let Some(path) = discover_from_cwd()? {
        validate_data_root(&path)?;
        return Ok(ResolvedDataRoot {
            path,
            source: DataRootSource::WorkspaceDiscovery,
        });
    }

    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("data");
    if manifest_root.exists() {
        let path = canonicalize_existing(&manifest_root)?;
        validate_data_root(&path)?;
        return Ok(ResolvedDataRoot {
            path,
            source: DataRootSource::ManifestDir,
        });
    }

    Err(DataError::InvalidDataRoot {
        path: env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .display()
            .to_string(),
        message: format!(
            "could not resolve runtime data root; checked LEXFLEX_DATA_DIR, workspace ancestors and {}",
            manifest_root.display()
        ),
    })
}

pub fn validate_data_root(root: &Path) -> Result<(), DataError> {
    if !root.exists() {
        return Err(DataError::InvalidDataRoot {
            path: root.display().to_string(),
            message: "path does not exist".into(),
        });
    }
    if !root.is_dir() {
        return Err(DataError::InvalidDataRoot {
            path: root.display().to_string(),
            message: "path is not a directory".into(),
        });
    }
    for (asset, relative) in REQUIRED_ASSETS {
        let candidate = root.join(relative);
        if !candidate.is_file() {
            return Err(DataError::MissingRequiredAsset {
                asset: (*asset).into(),
                path: candidate.display().to_string(),
            });
        }
    }
    Ok(())
}

fn discover_from_cwd() -> Result<Option<PathBuf>, DataError> {
    let mut current = env::current_dir().map_err(|error| DataError::InvalidDataRoot {
        path: ".".into(),
        message: error.to_string(),
    })?;
    loop {
        let candidate = current.join("data");
        if candidate.exists() && candidate.is_dir() {
            let canonical = canonicalize_existing(&candidate)?;
            if validate_data_root(&canonical).is_ok() {
                return Ok(Some(canonical));
            }
        }
        if !current.pop() {
            return Ok(None);
        }
    }
}

fn canonicalize_existing(path: &Path) -> Result<PathBuf, DataError> {
    std::fs::canonicalize(path).map_err(|error| DataError::InvalidDataRoot {
        path: path.display().to_string(),
        message: error.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::{resolve_data_root, validate_data_root, DataRootSource};
    use std::path::Path;

    #[test]
    fn default_root_resolves_from_workspace() {
        let root = resolve_data_root(None).expect("root should resolve");
        assert!(root.path.ends_with("data"));
        assert!(matches!(
            root.source,
            DataRootSource::WorkspaceDiscovery | DataRootSource::ManifestDir
        ));
    }

    #[test]
    fn explicit_invalid_root_is_rejected() {
        let invalid = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let error = validate_data_root(&invalid).expect_err("invalid root must fail");
        assert!(error.to_string().contains("Missing required runtime asset"));
    }
}
