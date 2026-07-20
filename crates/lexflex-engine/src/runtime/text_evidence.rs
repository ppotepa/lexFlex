use super::evidence_error_mapping::{evidence_error_response, EvidenceOrigin};
use crate::api::input::TextInput;
use crate::api::response::EngineResponse;
use lexflex_model::{canonical_hash, Evidence, EvidenceSet, SourceSpan};

pub(crate) fn build_text_evidence(
    input: &TextInput,
    span: SourceSpan,
) -> Result<EvidenceSet, EngineResponse> {
    let source_hash = canonical_hash(&input.text).map_err(|error| {
        evidence_error_response(
            lexflex_model::EvidenceError::CanonicalHash(error),
            EvidenceOrigin::GeneratedText,
        )
    })?;
    let evidence = Evidence::create(input.source_id.clone(), Some(span), Some(source_hash))
        .map_err(|error| evidence_error_response(error, EvidenceOrigin::GeneratedText))?;
    EvidenceSet::singleton(evidence)
        .map_err(|error| evidence_error_response(error, EvidenceOrigin::GeneratedText))
}

pub(crate) struct TextBuildResult {
    pub(crate) span: SourceSpan,
    pub(crate) evidence: EvidenceSet,
}

pub(crate) fn build_full_text_evidence(
    input: &TextInput,
) -> Result<TextBuildResult, EngineResponse> {
    let span = SourceSpan::new(0, input.text.len() as u64)
        .map_err(|error| evidence_error_response(error, EvidenceOrigin::GeneratedText))?;
    let evidence = build_text_evidence(input, span.clone())?;
    Ok(TextBuildResult { span, evidence })
}
