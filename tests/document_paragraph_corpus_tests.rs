use lexflex::core::interlingua::LanguageId;
use lexflex::document::{
    DocumentBlockKind, DocumentInput, DocumentReconstructor, DocumentSegmentationOptions,
    DocumentSegmenter, LosslessParagraphSegmenter, SpanPrecision,
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

fn corpus_sources(root: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for split in ["dev", "regression", "holdout"] {
        let split_dir = root
            .join("benchmarks/document_v1/cases")
            .join(split);
        for entry in fs::read_dir(&split_dir).unwrap() {
            let candidate = entry.unwrap().path().join("source.pl.txt");
            if candidate.is_file() {
                paths.push(candidate);
            }
        }
    }
    paths.sort();
    paths
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

#[test]
fn all_document_corpus_sources_segment_losslessly() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let paths = corpus_sources(root);
    assert_eq!(paths.len(), 30, "expected the complete Document V1 corpus");
    let expected_counts = expected_paragraph_counts();
    let segmenter = LosslessParagraphSegmenter;

    for path in paths {
        let source = fs::read_to_string(&path).unwrap();
        let case_id = path
            .parent()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            .unwrap();
        let build = || {
            segmenter
                .segment(
                    DocumentInput {
                        id: None,
                        source_language: LanguageId::new("pl"),
                        source: source.clone(),
                    },
                    &DocumentSegmentationOptions::default(),
                )
                .unwrap()
        };
        let first = build();
        let second = build();
        assert!(first.validate_structure().is_ok(), "invalid structure: {case_id}");
        assert!(
            DocumentReconstructor::verify_lossless(&first).is_ok(),
            "reconstruction failed: {case_id}"
        );
        assert_eq!(first.reconstruct().unwrap(), source, "source changed: {case_id}");
        assert_eq!(first, second, "nondeterministic document: {case_id}");
        assert_eq!(
            serde_json::to_string(&first).unwrap(),
            serde_json::to_string(&second).unwrap(),
            "nondeterministic JSON: {case_id}"
        );
        assert!(first.sentences().is_empty(), "sentences created too early: {case_id}");

        let expected = expected_counts.get(case_id).copied().unwrap_or(1);
        assert_eq!(first.paragraphs().len(), expected, "paragraph count: {case_id}");
        let expected_hash = format!("{:x}", Sha256::digest(source.as_bytes()));
        assert_eq!(first.source_sha256(), expected_hash, "source hash: {case_id}");

        let mut cursor = 0;
        for block_id in first.block_order() {
            let block = first.block(block_id).unwrap();
            assert_eq!(block.span.precision, SpanPrecision::Exact);
            let span = block.span.span.unwrap();
            assert!(!span.is_empty(), "empty block: {case_id}");
            assert!(span.validate_for(&source).is_ok(), "invalid span: {case_id}");
            assert_eq!(span.start, cursor, "block gap or overlap: {case_id}");
            if matches!(block.kind, DocumentBlockKind::PreservedWhitespace) {
                assert!(span.slice(&source).unwrap().chars().all(char::is_whitespace));
            }
            cursor = span.end;
        }
        assert_eq!(cursor, source.len(), "incomplete coverage: {case_id}");
    }
}
