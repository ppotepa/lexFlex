use lexflex_document_benchmark::corpus::{
    DocumentCasePaths, LoadedDocumentCase,
};
use lexflex_document_benchmark::expectations::evaluate_expectations;
use lexflex_document_benchmark::glossary::evaluate_glossary;
use lexflex_document_benchmark::model::{
    CanonicalDocumentSemantics, CanonicalEntityMention, CanonicalFrame, CanonicalRole,
    CanonicalSentence, CorpusSplit, DocumentCase, DocumentCaseRef, DocumentErrorCategory,
    DocumentExpectations, DocumentInvariant, GlossaryConstraint, KnownFailure,
};
use std::path::PathBuf;

fn semantics() -> CanonicalDocumentSemantics {
    CanonicalDocumentSemantics {
        sentences: vec![
            CanonicalSentence {
                index: 0,
                frames: vec![CanonicalFrame {
                    frame_type: "Motion".into(),
                    verb_concept: "GO".into(),
                    roles: vec![CanonicalRole {
                        role: "Agent".into(),
                        concept: "PERSON".into(),
                        name: Some("Tomek".into()),
                        reference: "Direct".into(),
                    }],
                }],
                tense: Some("Past".into()),
                aspect: Some("Perfective".into()),
                polarity: "Positive".into(),
                modality: Some("Realis".into()),
                illocution: "Statement".into(),
                voice: Some("Active".into()),
                temporal: Some("Yesterday".into()),
                quantification: Some("Numerical(2)".into()),
                constructions: vec!["Alpha".into(), "Beta".into()],
            },
            CanonicalSentence {
                index: 1,
                frames: vec![CanonicalFrame {
                    frame_type: "Communication".into(),
                    verb_concept: "SAY".into(),
                    roles: vec![CanonicalRole {
                        role: "Speaker".into(),
                        concept: "PERSON".into(),
                        name: Some("Anna".into()),
                        reference: "Anaphoric(\"Tomek\")".into(),
                    }],
                }],
                tense: Some("Present".into()),
                aspect: None,
                polarity: "Negative".into(),
                modality: Some("Realis".into()),
                illocution: "Statement".into(),
                voice: Some("Active".into()),
                temporal: Some("Today".into()),
                quantification: None,
                constructions: vec![],
            },
        ],
        entities: vec![
            CanonicalEntityMention {
                concept: "PERSON".into(),
                name: Some("Tomek".into()),
                reference: "Direct".into(),
            },
            CanonicalEntityMention {
                concept: "NUMBER".into(),
                name: Some("2".into()),
                reference: "Direct".into(),
            },
        ],
        unresolved_count: 0,
    }
}

fn case(expectations: DocumentExpectations, glossary: Vec<GlossaryConstraint>) -> LoadedDocumentCase {
    LoadedDocumentCase {
        reference: DocumentCaseRef {
            id: "doc-pl-en-001".into(),
            split: CorpusSplit::Dev,
            directory: "cases/dev/doc-pl-en-001".into(),
            enabled: true,
            blocking: true,
        },
        metadata: DocumentCase {
            schema_version: 1,
            id: "doc-pl-en-001".into(),
            title: "t".into(),
            source_language: "pl".into(),
            target_language: "en".into(),
            length_tier: "micro".into(),
            domain: "daily".into(),
            difficulty: 1,
            tags: vec![],
            source_file: "source.pl.txt".into(),
            reference_files: vec!["reference.en.txt".into()],
            expectations_file: "expectations.ron".into(),
            glossary_file: "glossary.ron".into(),
            known_limitations: vec![],
            notes: None,
        },
        source: "Tomek poszedł do sklepu.\n\nNie wrócił.".into(),
        references: vec!["Tomek went to a store.\n\nHe did not return.".into()],
        expectations,
        glossary,
        paths: DocumentCasePaths {
            directory: PathBuf::from("/tmp/doc-pl-en-001"),
            source: PathBuf::from("/tmp/doc-pl-en-001/source.pl.txt"),
            references: vec![PathBuf::from("/tmp/doc-pl-en-001/reference.en.txt")],
            expectations: PathBuf::from("/tmp/doc-pl-en-001/expectations.ron"),
            glossary: PathBuf::from("/tmp/doc-pl-en-001/glossary.ron"),
        },
    }
}

#[test]
fn expectations_cover_required_invariants() {
    let expectations = DocumentExpectations {
        expected_paragraph_count: Some(2),
        expected_min_sentence_count: Some(2),
        invariants: vec![
            DocumentInvariant::PreserveName {
                source: "Tomek".into(),
                accepted_targets: vec!["Tomek".into()],
                minimum_occurrences: 1,
            },
            DocumentInvariant::PreserveNumber {
                source_surface: "2".into(),
                normalized_value: "2".into(),
                accepted_targets: vec!["two".into(), "2".into()],
            },
            DocumentInvariant::PreserveNegation {
                sentence_hint: 2,
                predicate_concept: Some("RETURN".into()),
            },
            DocumentInvariant::RequireFrame {
                frame_type: "Motion".into(),
                verb_concept: Some("GO".into()),
                minimum_occurrences: 1,
            },
            DocumentInvariant::RequireRoleBinding {
                frame_type: "Motion".into(),
                role: "Agent".into(),
                concept: "PERSON".into(),
                name: Some("Tomek".into()),
                minimum_occurrences: 1,
            },
            DocumentInvariant::RequireReference {
                mention: "He".into(),
                antecedent_name: "Tomek".into(),
                target_forms: vec!["He".into(), "he".into()],
            },
            DocumentInvariant::RequireTemporal {
                kind: "Today".into(),
                target_forms: vec!["today".into()],
            },
            DocumentInvariant::RequireParagraphCount { count: 2 },
            DocumentInvariant::RequireOutputContains {
                any_of: vec!["Tomek".into()],
                case_sensitive: false,
            },
            DocumentInvariant::ForbidOutputContains {
                any_of: vec!["BAD".into()],
                case_sensitive: false,
            },
        ],
        allowed_failures: vec![KnownFailure {
            id: "doc-pl-en-001-1".into(),
            category: DocumentErrorCategory::Syntax,
            description: "expected known failure".into(),
            expected_until_chapter: 99,
        }],
    };
    let case = case(expectations, vec![]);
    let output = Some("Tomek went to a store with 2 bags.\n\nHe did not return today.");
    let results = evaluate_expectations(&case, output, Some(&semantics()), Some(&semantics()));
    assert!(
        results.iter().all(|result| result.passed),
        "{:?}",
        results
            .iter()
            .filter(|result| !result.passed)
            .map(|result| result.invariant_type.clone())
            .collect::<Vec<_>>()
    );
    assert!(results.iter().any(|result| result.invariant_type == "ExpectedParagraphCount"));
    assert!(results.iter().any(|result| result.invariant_type == "ExpectedMinSentenceCount"));
}

#[test]
fn expectations_report_failures_and_known_failures() {
    let expectations = DocumentExpectations {
        expected_paragraph_count: Some(1),
        expected_min_sentence_count: Some(1),
        invariants: vec![DocumentInvariant::PreserveName {
            source: "Tomek".into(),
            accepted_targets: vec!["Tomek".into()],
            minimum_occurrences: 1,
        }],
        allowed_failures: vec![KnownFailure {
            id: "doc-pl-en-001-0".into(),
            category: DocumentErrorCategory::Coreference,
            description: "known".into(),
            expected_until_chapter: 99,
        }],
    };
    let case = case(expectations, vec![]);
    let results = evaluate_expectations(&case, Some(""), None, None);
    assert!(!results[0].passed);
    assert!(results[0].known_failure.is_some());
}

#[test]
fn glossary_evaluation_uses_constraint_rules() {
    let case = case(
        DocumentExpectations {
            expected_paragraph_count: None,
            expected_min_sentence_count: None,
            invariants: vec![],
            allowed_failures: vec![],
        },
        vec![GlossaryConstraint {
            id: "TERM-001".into(),
            source: "silnik renderujący".into(),
            preferred_target: "rendering engine".into(),
            allowed_targets: vec!["rendering engine".into()],
            forbidden_targets: vec!["render motor".into()],
            case_sensitive: false,
            minimum_occurrences: 2,
            consistency_required: true,
        }],
    );
    let passed = evaluate_glossary(&case, Some("The rendering engine is stable. Rendering engine wins."));
    assert!(passed[0].passed);
    let failed = evaluate_glossary(&case, Some("render motor"));
    assert!(!failed[0].passed);
}

#[test]
fn glossary_rejects_mixed_variants_when_consistency_required() {
    let case = case(
        DocumentExpectations {
            expected_paragraph_count: None,
            expected_min_sentence_count: None,
            invariants: vec![],
            allowed_failures: vec![],
        },
        vec![GlossaryConstraint {
            id: "TERM-002".into(),
            source: "silnik renderujący".into(),
            preferred_target: "rendering engine".into(),
            allowed_targets: vec!["render engine".into()],
            forbidden_targets: vec![],
            case_sensitive: false,
            minimum_occurrences: 2,
            consistency_required: true,
        }],
    );
    let mixed = evaluate_glossary(
        &case,
        Some("rendering engine runs here. render engine appears here too."),
    );
    assert!(!mixed[0].passed);
    assert!(mixed[0]
        .evidence
        .iter()
        .any(|line| line.contains("distinct_accepted_variants")));
}
