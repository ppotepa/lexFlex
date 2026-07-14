use lexflex::document::{
    DocumentCapabilityStatus, DocumentErrorSeverity, DocumentProfile,
};
use std::fs;
use std::path::Path;

fn load_profile() -> DocumentProfile {
    let path = Path::new("data/profiles/document_mvp_v1.ron");
    let text = fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read {}: {}", path.display(), err));
    ron::from_str(&text).expect("profile should deserialize")
}

#[test]
fn test_document_mvp_profile_file_exists() {
    let path = Path::new("data/profiles/document_mvp_v1.ron");
    assert!(path.exists(), "missing profile file: {}", path.display());
}

#[test]
fn test_document_mvp_profile_deserializes() {
    let profile = load_profile();
    assert_eq!(profile.id, "document-mvp-v1");
}

#[test]
fn test_document_mvp_profile_validates() {
    let profile = load_profile();
    profile.validate().expect("profile should validate");
}

#[test]
fn test_document_mvp_profile_direction() {
    let profile = load_profile();
    assert_eq!(profile.source_language, "pl");
    assert_eq!(profile.target_language, "en");
    assert!(profile.blocking_direction);
}

#[test]
fn test_document_mvp_profile_is_not_page_based() {
    let profile = load_profile();
    let ron_text = fs::read_to_string("data/profiles/document_mvp_v1.ron").unwrap();
    let combined = format!("{:?}{:?}{}", profile.id, profile.display_name, ron_text);
    assert!(!combined.to_lowercase().contains("a4"));
    assert!(!ron_text.contains("page_size"));
    assert!(!ron_text.contains("page_count"));
}

#[test]
fn test_document_mvp_profile_has_blocking_micro_and_short_tiers() {
    let profile = load_profile();
    assert!(profile.length_tier("micro").unwrap().blocking);
    assert!(profile.length_tier("short").unwrap().blocking);
}

#[test]
fn test_document_mvp_profile_has_standard_tracking_tier() {
    let profile = load_profile();
    let standard = profile.length_tier("standard").expect("standard tier");
    assert!(!standard.blocking);
}

#[test]
fn test_document_mvp_profile_has_required_document_capabilities() {
    let profile = load_profile();
    for id in [
        "DOC-STR-001",
        "DOC-STR-002",
        "DOC-REF-001",
        "DOC-REF-002",
        "DOC-REF-003",
        "DOC-TIM-001",
        "DOC-SEM-001",
        "DOC-SEM-002",
        "DOC-SEM-003",
        "DOC-LEX-001",
        "DOC-FMT-001",
    ] {
        assert!(profile.capability(id).is_some(), "missing capability {id}");
    }
}

#[test]
fn test_document_mvp_profile_required_capabilities_are_blocking() {
    let profile = load_profile();
    for cap in profile.capabilities.iter().filter(|cap| cap.status == lexflex::document::DocumentCapabilityStatus::Required) {
        assert!(cap.blocking, "required capability {} must be blocking", cap.id);
    }
}

#[test]
fn test_document_mvp_profile_deferred_capabilities_are_not_blocking() {
    let profile = load_profile();
    for cap in profile.capabilities.iter().filter(|cap| cap.status == DocumentCapabilityStatus::Deferred) {
        assert!(!cap.blocking, "deferred capability {} must not be blocking", cap.id);
    }
}

#[test]
fn test_document_mvp_profile_thresholds_are_in_range() {
    let profile = load_profile();
    let thresholds = [
        profile.quality_gates.required_name_preservation,
        profile.quality_gates.required_number_preservation,
        profile.quality_gates.required_negation_preservation,
        profile.quality_gates.required_reference_accuracy,
        profile.quality_gates.required_zero_anaphora_accuracy,
        profile.quality_gates.required_frame_signature_recall,
        profile.quality_gates.required_glossary_compliance,
        profile.quality_gates.required_paragraph_preservation,
    ];
    assert!(thresholds.iter().all(|value| value.milli() <= 1000));
}

#[test]
fn test_document_mvp_profile_error_taxonomy_contains_fatal_semantic_errors() {
    let profile = load_profile();
    let categories: Vec<_> = profile
        .error_severity
        .iter()
        .map(|rule| (rule.category.as_str(), rule.severity))
        .collect();
    for category in ["lost_negation", "changed_number", "role_swap"] {
        assert!(categories.iter().any(|(name, severity)| *name == category && matches!(severity, DocumentErrorSeverity::Fatal)));
    }
}
