use lexflex::core::interlingua::LanguageId;
use lexflex::document::{
    DocumentInput, DocumentSegmentationOptions, DocumentSegmenter, LosslessDocumentSegmenter,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

fn source_paths(root: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for split in ["dev", "regression", "holdout"] {
        let split_dir = root.join("benchmarks/document_v1/cases").join(split);
        for entry in fs::read_dir(&split_dir).unwrap() {
            let path = entry.unwrap().path().join("source.pl.txt");
            if path.is_file() {
                paths.push(path);
            }
        }
    }
    paths.sort();
    paths
}

fn expected_counts() -> BTreeMap<&'static str, usize> {
    [
        ("doc-pl-en-001", 3), ("doc-pl-en-002", 3), ("doc-pl-en-003", 3),
        ("doc-pl-en-004", 4), ("doc-pl-en-005", 4), ("doc-pl-en-006", 3),
        ("doc-pl-en-007", 3), ("doc-pl-en-008", 3), ("doc-pl-en-009", 2),
        ("doc-pl-en-010", 3), ("doc-pl-en-011", 2), ("doc-pl-en-012", 1),
        ("doc-pl-en-013", 2), ("doc-pl-en-014", 2), ("doc-pl-en-015", 1),
        ("doc-pl-en-016", 2), ("doc-pl-en-017", 2), ("doc-pl-en-018", 3),
        ("doc-pl-en-019", 2), ("doc-pl-en-020", 4), ("doc-pl-en-021", 4),
        ("doc-pl-en-022", 3), ("doc-pl-en-023", 2), ("doc-pl-en-024", 5),
        ("doc-pl-en-025", 5), ("doc-pl-en-026", 3), ("doc-pl-en-027", 1),
        ("doc-pl-en-028", 1), ("doc-pl-en-029", 1), ("doc-pl-en-030", 5),
    ]
    .into_iter()
    .collect()
}

#[test]
fn document_corpus_has_exact_lossless_sentence_coverage() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let paths = source_paths(root);
    let expected = expected_counts();
    assert_eq!(paths.len(), 30);
    let mut total_sentences = 0;
    for path in paths {
        let case_id = path.parent().unwrap().file_name().unwrap().to_str().unwrap();
        let source = fs::read_to_string(&path).unwrap();
        let input = || DocumentInput {
            id: None,
            source_language: LanguageId::new("pl"),
            source: source.clone(),
        };
        let first = LosslessDocumentSegmenter
            .segment(input(), &DocumentSegmentationOptions::default())
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        let second = LosslessDocumentSegmenter
            .segment(input(), &DocumentSegmentationOptions::default())
            .unwrap();
        assert_eq!(first.reconstruct().unwrap(), source, "{case_id}");
        assert!(first.validate_structure().is_ok(), "{case_id}");
        assert_eq!(first, second, "{case_id}");
        assert_eq!(
            serde_json::to_string(&first).unwrap(),
            serde_json::to_string(&second).unwrap(),
            "{case_id}"
        );
        assert_eq!(first.sentences().len(), expected[case_id], "{case_id}");
        total_sentences += first.sentences().len();
    }
    assert_eq!(total_sentences, 82);
}
