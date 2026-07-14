use crate::error::BenchmarkError;
use crate::model::{
    CorpusSplit, DocumentCase, DocumentCaseRef, DocumentCorpusManifest, DocumentExpectations,
    GlossaryConstraint,
};
use lexflex::document::DocumentProfile;
use ron::de::from_str;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct LoadedCorpus {
    pub root: PathBuf,
    pub profile: DocumentProfile,
    pub manifest: DocumentCorpusManifest,
    pub cases: Vec<LoadedDocumentCase>,
}

#[derive(Debug, Clone)]
pub struct LoadedDocumentCase {
    pub reference: DocumentCaseRef,
    pub metadata: DocumentCase,
    pub source: String,
    pub references: Vec<String>,
    pub expectations: DocumentExpectations,
    pub glossary: Vec<GlossaryConstraint>,
    pub paths: DocumentCasePaths,
}

#[derive(Debug, Clone)]
pub struct DocumentCasePaths {
    pub directory: PathBuf,
    pub source: PathBuf,
    pub references: Vec<PathBuf>,
    pub expectations: PathBuf,
    pub glossary: PathBuf,
}

pub fn load_corpus(root: &Path) -> Result<LoadedCorpus, BenchmarkError> {
    let manifest_path = root.join("manifest.ron");
    let manifest: DocumentCorpusManifest = read_ron(&manifest_path)?;
    let profile_path = root.join(&manifest.profile_path);
    let profile: DocumentProfile = read_ron(&profile_path)?;
    if let Err(errs) = profile.validate() {
        return Err(BenchmarkError::ProfileValidation(errs));
    }

    let mut cases = Vec::new();
    for reference in &manifest.cases {
        let directory = root.join(&reference.directory);
        let case_path = directory.join("case.ron");
        let metadata: DocumentCase = read_ron(&case_path)?;
        let source = read_utf8(&directory.join(&metadata.source_file))?;
        let mut references = Vec::new();
        let mut reference_paths = Vec::new();
        for rel in &metadata.reference_files {
            let path = directory.join(rel);
            reference_paths.push(path.clone());
            references.push(read_utf8(&path)?);
        }
        let expectations_path = directory.join(&metadata.expectations_file);
        let expectations: DocumentExpectations = read_ron(&expectations_path)?;
        let glossary_path = directory.join(&metadata.glossary_file);
        let glossary: Vec<GlossaryConstraint> = read_ron(&glossary_path)?;

        cases.push(LoadedDocumentCase {
            reference: reference.clone(),
            metadata,
            source,
            references,
            expectations,
            glossary,
            paths: DocumentCasePaths {
                directory: directory.clone(),
                source: directory.join("source.pl.txt"),
                references: reference_paths,
                expectations: expectations_path,
                glossary: glossary_path,
            },
        });
    }

    Ok(LoadedCorpus {
        root: root.to_path_buf(),
        profile,
        manifest,
        cases,
    })
}

pub fn read_ron<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, BenchmarkError> {
    let text = read_utf8(path)?;
    from_str(&text).map_err(|source| BenchmarkError::Ron {
        path: path.display().to_string(),
        source,
    })
}

pub fn read_utf8(path: &Path) -> Result<String, BenchmarkError> {
    fs::read_to_string(path).map_err(|source| BenchmarkError::Io {
        path: path.display().to_string(),
        source,
    })
}

pub fn case_split(reference: &DocumentCaseRef) -> CorpusSplit {
    reference.split
}

pub fn sha256_json<T: serde::Serialize>(value: &T) -> String {
    let mut hasher = sha2::Sha256::new();
    use sha2::Digest;
    let json = serde_json::to_string(value).expect("deterministic serialization");
    hasher.update(json.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = sha2::Sha256::new();
    use sha2::Digest;
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}
