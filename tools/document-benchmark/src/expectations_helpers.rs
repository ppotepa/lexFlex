use crate::model::{CanonicalDocumentSemantics, DocumentInvariant, ExpectationResult, KnownFailure};
use crate::corpus::LoadedDocumentCase;

pub fn result(
    invariant_index: usize,
    invariant_type: &str,
    passed: bool,
    known_failure: Option<KnownFailure>,
    evidence: Vec<String>,
) -> ExpectationResult {
    ExpectationResult {
        invariant_index,
        invariant_type: invariant_type.to_string(),
        passed,
        known_failure,
        severity: severity_for(passed, evidence.is_empty()),
        message: if passed {
            format!("{invariant_type} passed")
        } else {
            format!("{invariant_type} failed")
        },
        evidence,
    }
}

pub fn severity_for(passed: bool, weak_evidence: bool) -> String {
    if passed {
        "info".into()
    } else if weak_evidence {
        "minor".into()
    } else {
        "major".into()
    }
}

pub fn known_failure(
    case: &LoadedDocumentCase,
    index: usize,
    invariant_type: &str,
) -> Option<KnownFailure> {
    case.expectations
        .allowed_failures
        .iter()
        .find(|failure| {
            failure.id == format!("{}-{index}", case.metadata.id)
                || failure.id == format!("{}-{invariant_type}", case.metadata.id)
                || failure.id == invariant_type
        })
        .cloned()
}

pub fn invariant_kind(invariant: &DocumentInvariant) -> &'static str {
    match invariant {
        DocumentInvariant::PreserveName { .. } => "PreserveName",
        DocumentInvariant::PreserveNumber { .. } => "PreserveNumber",
        DocumentInvariant::PreserveNegation { .. } => "PreserveNegation",
        DocumentInvariant::RequireFrame { .. } => "RequireFrame",
        DocumentInvariant::RequireRoleBinding { .. } => "RequireRoleBinding",
        DocumentInvariant::RequireReference { .. } => "RequireReference",
        DocumentInvariant::RequireTemporal { .. } => "RequireTemporal",
        DocumentInvariant::RequireParagraphCount { .. } => "RequireParagraphCount",
        DocumentInvariant::RequireOutputContains { .. } => "RequireOutputContains",
        DocumentInvariant::ForbidOutputContains { .. } => "ForbidOutputContains",
    }
}

pub fn evidence_for(
    invariant: &DocumentInvariant,
    output: &str,
    source_semantics: Option<&CanonicalDocumentSemantics>,
    target_semantics: Option<&CanonicalDocumentSemantics>,
) -> Vec<String> {
    match invariant {
        DocumentInvariant::PreserveName { source, accepted_targets, .. } => {
            let mut evidence = vec![format!("source={source}")];
            if !accepted_targets.is_empty() {
                evidence.push(format!("accepted={}", accepted_targets.join("|")));
            }
            evidence.push(format!(
                "output_hits={}",
                accepted_targets
                    .iter()
                    .chain(std::iter::once(source))
                    .map(|needle| count_occurrences(output, needle, false))
                    .sum::<usize>()
            ));
            if let Some(semantics) = target_semantics {
                evidence.push(format!("target_names={}", names_of(semantics).join("|")));
            }
            evidence
        }
        DocumentInvariant::PreserveNumber { source_surface, normalized_value, accepted_targets } => {
            let mut evidence = vec![format!("source={source_surface}"), format!("normalized={normalized_value}")];
            if !accepted_targets.is_empty() {
                evidence.push(format!("accepted={}", accepted_targets.join("|")));
            }
            evidence.push(format!(
                "output_hits={}",
                accepted_targets
                    .iter()
                    .chain(std::iter::once(source_surface))
                    .chain(std::iter::once(normalized_value))
                    .map(|needle| count_occurrences(output, needle, false))
                    .sum::<usize>()
            ));
            evidence
        }
        DocumentInvariant::PreserveNegation { sentence_hint, predicate_concept } => {
            let mut evidence = vec![format!("sentence_hint={sentence_hint}")];
            if let Some(predicate) = predicate_concept {
                evidence.push(format!("predicate={predicate}"));
            }
            if let Some(source) = source_semantics {
                evidence.push(format!("source_polarity={}", sentence_polarity(source, *sentence_hint)));
            }
            if let Some(target) = target_semantics {
                evidence.push(format!("target_polarity={}", sentence_polarity(target, *sentence_hint)));
            }
            evidence.push(format!("negation_hits={}", count_occurrences(output, "nie", false)));
            evidence
        }
        DocumentInvariant::RequireFrame { frame_type, verb_concept, .. } => {
            let mut evidence = vec![format!("frame_type={frame_type}")];
            if let Some(verb_concept) = verb_concept {
                evidence.push(format!("verb_concept={verb_concept}"));
            }
            evidence.push(format!(
                "target_matches={}",
                count_frame_matches(target_semantics, frame_type, verb_concept.as_deref())
            ));
            evidence
        }
        DocumentInvariant::RequireRoleBinding { frame_type, role, concept, name, .. } => {
            let mut evidence = vec![format!("frame_type={frame_type}"), format!("role={role}"), format!("concept={concept}")];
            if let Some(name) = name {
                evidence.push(format!("name={name}"));
            }
            evidence.push(format!(
                "target_matches={}",
                count_role_matches(target_semantics, frame_type, role, concept, name.as_deref())
            ));
            evidence
        }
        DocumentInvariant::RequireReference { mention, antecedent_name, target_forms } => {
            let mut evidence = vec![format!("mention={mention}"), format!("antecedent={antecedent_name}")];
            if !target_forms.is_empty() {
                evidence.push(format!("target_forms={}", target_forms.join("|")));
            }
            if let Some(target) = target_semantics {
                evidence.push(format!("target_entities={}", target.entities.len()));
            }
            evidence
        }
        DocumentInvariant::RequireTemporal { kind, target_forms } => {
            let mut evidence = vec![format!("kind={kind}")];
            if !target_forms.is_empty() {
                evidence.push(format!("target_forms={}", target_forms.join("|")));
            }
            if let Some(target) = target_semantics {
                evidence.push(format!("target_temporals={}", target.sentences.iter().filter_map(|s| s.temporal.as_ref()).count()));
            }
            evidence
        }
        DocumentInvariant::RequireParagraphCount { count } => {
            vec![format!("output_paragraphs={}", count_paragraphs(output)), format!("expected={count}")]
        }
        DocumentInvariant::RequireOutputContains { any_of, .. } => any_of.to_vec(),
        DocumentInvariant::ForbidOutputContains { any_of, .. } => any_of.to_vec(),
    }
}

pub fn preserve_name(
    source: &str,
    accepted_targets: &[String],
    minimum_occurrences: usize,
    output: &str,
    target_semantics: Option<&CanonicalDocumentSemantics>,
) -> bool {
    let mut needles = vec![source.to_string()];
    needles.extend(accepted_targets.iter().cloned());
    let hits = needles
        .iter()
        .map(|needle| count_occurrences(output, needle, false))
        .sum::<usize>();
    if hits < minimum_occurrences {
        return false;
    }
    target_semantics.map_or(hits > 0, |semantics| {
        names_of(semantics)
            .iter()
            .any(|name| any_match(name, &needles))
    })
}

pub fn preserve_number(
    source_surface: &str,
    normalized_value: &str,
    accepted_targets: &[String],
    output: &str,
    target_semantics: Option<&CanonicalDocumentSemantics>,
) -> bool {
    let mut needles = vec![source_surface.to_string(), normalized_value.to_string()];
    needles.extend(accepted_targets.iter().cloned());
    if !needles.iter().any(|needle| any_match(output, &[needle.clone()])) {
        return false;
    }
    target_semantics.map_or(true, |semantics| {
        names_of(semantics).iter().any(|name| any_match(name, &needles))
            || semantics
                .sentences
                .iter()
                .any(|sentence| sentence.quantification.is_some())
    })
}

pub fn preserve_negation(
    sentence_hint: usize,
    _predicate_concept: Option<&str>,
    source_semantics: Option<&CanonicalDocumentSemantics>,
    target_semantics: Option<&CanonicalDocumentSemantics>,
    output: &str,
) -> bool {
    let source_negative = source_semantics
        .and_then(|sem| sem.sentences.get(sentence_hint.saturating_sub(1)))
        .map(|sentence| sentence.polarity.eq_ignore_ascii_case("Negative"))
        .unwrap_or_else(|| count_occurrences(output, "nie", false) > 0);
    if !source_negative {
        return false;
    }
    target_semantics
        .and_then(|sem| sem.sentences.get(sentence_hint.saturating_sub(1)))
        .map(|sentence| sentence.polarity.eq_ignore_ascii_case("Negative"))
        .unwrap_or_else(|| contains_any(output, &["not".into(), "nie".into()], false))
}

pub fn require_frame(
    frame_type: &str,
    verb_concept: Option<&str>,
    minimum_occurrences: usize,
    target_semantics: Option<&CanonicalDocumentSemantics>,
) -> bool {
    count_frame_matches(target_semantics, frame_type, verb_concept) >= minimum_occurrences
}

pub fn require_role_binding(
    frame_type: &str,
    role: &str,
    concept: &str,
    name: Option<&str>,
    minimum_occurrences: usize,
    target_semantics: Option<&CanonicalDocumentSemantics>,
) -> bool {
    count_role_matches(target_semantics, frame_type, role, concept, name) >= minimum_occurrences
}

pub fn require_reference(
    mention: &str,
    antecedent_name: &str,
    target_forms: &[String],
    output: &str,
    target_semantics: Option<&CanonicalDocumentSemantics>,
) -> bool {
    let mut needles = target_forms.to_vec();
    needles.push(mention.to_string());
    if !contains_any(output, &needles, false) {
        return false;
    }
    target_semantics.map_or(false, |semantics| {
        semantics.entities.iter().any(|entity| {
            name_or_reference(entity).contains(antecedent_name)
                || target_forms
                    .iter()
                    .any(|needle| any_match_case(&name_or_reference(entity), needle, false))
        })
    })
}

pub fn require_temporal(
    kind: &str,
    target_forms: &[String],
    output: &str,
    target_semantics: Option<&CanonicalDocumentSemantics>,
) -> bool {
    if !contains_any(output, target_forms, false) && !contains_any(output, &[kind.to_string()], false) {
        return false;
    }
    target_semantics.map_or(false, |semantics| {
        semantics.sentences.iter().any(|sentence| {
            sentence
                .temporal
                .as_deref()
                .map(|temporal| any_match(temporal, &target_forms.iter().cloned().chain(std::iter::once(kind.to_string())).collect::<Vec<_>>()))
                .unwrap_or(false)
        })
    })
}

pub fn require_paragraph_count(output: &str, count: usize) -> bool {
    count_paragraphs(output) == count
}

pub fn contains_any(text: &str, needles: &[String], case_sensitive: bool) -> bool {
    needles.iter().any(|needle| any_match_case(text, needle, case_sensitive))
}

pub fn any_match(text: &str, needles: &[String]) -> bool {
    needles.iter().any(|needle| any_match_case(text, needle, false))
}

pub fn any_match_case(text: &str, needle: &str, case_sensitive: bool) -> bool {
    if case_sensitive {
        text.contains(needle)
    } else {
        text.to_lowercase().contains(&needle.to_lowercase())
    }
}

pub fn count_occurrences(text: &str, needle: &str, case_sensitive: bool) -> usize {
    if needle.is_empty() {
        return 0;
    }
    let haystack = if case_sensitive { text.to_string() } else { text.to_lowercase() };
    let needle = if case_sensitive {
        needle.to_string()
    } else {
        needle.to_lowercase()
    };
    haystack.match_indices(&needle).count()
}

pub fn count_paragraphs(text: &str) -> usize {
    text.split("\n\n").filter(|part| !part.trim().is_empty()).count()
}

pub fn count_sentences(text: &str, target_semantics: Option<&CanonicalDocumentSemantics>) -> usize {
    target_semantics
        .map(|semantics| semantics.sentences.len())
        .unwrap_or_else(|| {
            text.split_terminator(['.', '!', '?'])
                .filter(|part| !part.trim().is_empty())
                .count()
        })
}

pub fn count_frame_matches(
    target_semantics: Option<&CanonicalDocumentSemantics>,
    frame_type: &str,
    verb_concept: Option<&str>,
) -> usize {
    target_semantics
        .map(|semantics| {
            semantics
                .sentences
                .iter()
                .flat_map(|sentence| sentence.frames.iter())
                .filter(|frame| {
                    frame.frame_type.eq_ignore_ascii_case(frame_type)
                        && verb_concept.map(|verb| frame.verb_concept.eq_ignore_ascii_case(verb)).unwrap_or(true)
                })
                .count()
        })
        .unwrap_or_default()
}

pub fn count_role_matches(
    target_semantics: Option<&CanonicalDocumentSemantics>,
    frame_type: &str,
    role: &str,
    concept: &str,
    name: Option<&str>,
) -> usize {
    target_semantics
        .map(|semantics| {
            semantics
                .sentences
                .iter()
                .flat_map(|sentence| sentence.frames.iter())
                .filter(|frame| frame.frame_type.eq_ignore_ascii_case(frame_type))
                .flat_map(|frame| frame.roles.iter())
                .filter(|binding| {
                    binding.role.eq_ignore_ascii_case(role)
                        && binding.concept.eq_ignore_ascii_case(concept)
                        && name.map(|name| binding.name.as_deref() == Some(name)).unwrap_or(true)
                })
                .count()
        })
        .unwrap_or_default()
}

pub fn names_of(semantics: &CanonicalDocumentSemantics) -> Vec<String> {
    semantics
        .entities
        .iter()
        .filter_map(|entity| entity.name.clone())
        .collect()
}

pub fn name_or_reference(entity: &crate::model::CanonicalEntityMention) -> String {
    entity
        .name
        .clone()
        .unwrap_or_else(|| entity.reference.clone())
}

pub fn sentence_polarity(
    semantics: &CanonicalDocumentSemantics,
    sentence_hint: usize,
) -> String {
    semantics
        .sentences
        .get(sentence_hint.saturating_sub(1))
        .map(|sentence| sentence.polarity.clone())
        .unwrap_or_else(|| "Unknown".into())
}
