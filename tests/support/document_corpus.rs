use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DocumentCorpusCase {
    pub id: String,
    pub source_path: PathBuf,
    pub source: String,
    pub expected_sentences: usize,
    pub expected_paragraphs: usize,
}

pub fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn load_document_v1_cases() -> Vec<DocumentCorpusCase> {
    let root = repository_root();
    let manifest = expected_counts();
    let paragraph_counts = expected_paragraph_counts();
    let mut cases = Vec::new();

    for split in ["dev", "regression", "holdout"] {
        let split_dir = root.join("benchmarks/document_v1/cases").join(split);
        let mut dirs = fs::read_dir(&split_dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect::<Vec<_>>();
        dirs.sort();
        for dir in dirs {
            let case_id = dir.file_name().unwrap().to_str().unwrap().to_string();
            let source_path = dir.join("source.pl.txt");
            if !source_path.is_file() {
                continue;
            }
            let source = fs::read_to_string(&source_path).unwrap();
            cases.push(DocumentCorpusCase {
                id: case_id.clone(),
                source_path,
                source,
                expected_sentences: *manifest.get(case_id.as_str()).unwrap(),
                expected_paragraphs: paragraph_counts.get(case_id.as_str()).copied().unwrap_or(1),
            });
        }
    }

    cases.sort_by(|left, right| left.id.cmp(&right.id));
    assert_eq!(cases.len(), 30);
    cases
}

fn expected_counts() -> BTreeMap<&'static str, usize> {
    BTreeMap::from([
        ("doc-pl-en-001", 3),
        ("doc-pl-en-002", 3),
        ("doc-pl-en-003", 3),
        ("doc-pl-en-004", 4),
        ("doc-pl-en-005", 4),
        ("doc-pl-en-006", 3),
        ("doc-pl-en-007", 3),
        ("doc-pl-en-008", 3),
        ("doc-pl-en-009", 2),
        ("doc-pl-en-010", 3),
        ("doc-pl-en-011", 2),
        ("doc-pl-en-012", 1),
        ("doc-pl-en-013", 2),
        ("doc-pl-en-014", 2),
        ("doc-pl-en-015", 1),
        ("doc-pl-en-016", 2),
        ("doc-pl-en-017", 2),
        ("doc-pl-en-018", 3),
        ("doc-pl-en-019", 2),
        ("doc-pl-en-020", 4),
        ("doc-pl-en-021", 4),
        ("doc-pl-en-022", 3),
        ("doc-pl-en-023", 2),
        ("doc-pl-en-024", 5),
        ("doc-pl-en-025", 5),
        ("doc-pl-en-026", 3),
        ("doc-pl-en-027", 1),
        ("doc-pl-en-028", 1),
        ("doc-pl-en-029", 1),
        ("doc-pl-en-030", 5),
    ])
}

fn expected_paragraph_counts() -> BTreeMap<&'static str, usize> {
    BTreeMap::from([
        ("doc-pl-en-004", 2),
        ("doc-pl-en-005", 2),
        ("doc-pl-en-020", 2),
        ("doc-pl-en-021", 2),
        ("doc-pl-en-024", 3),
        ("doc-pl-en-025", 3),
        ("doc-pl-en-030", 2),
    ])
}

