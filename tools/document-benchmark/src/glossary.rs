use crate::corpus::LoadedDocumentCase;
use crate::model::ExpectationResult;

pub fn evaluate_glossary(case: &LoadedDocumentCase, output: Option<&str>) -> Vec<ExpectationResult> {
    let text = output.unwrap_or("");
    case.glossary
        .iter()
        .enumerate()
        .map(|(index, constraint)| {
            let case_sensitive = constraint.case_sensitive;
            let search_text = if case_sensitive {
                text.to_string()
            } else {
                text.to_lowercase()
            };
            let preferred = normalize(&constraint.preferred_target, case_sensitive);
            let mut accepted = vec![preferred.clone()];
            accepted.extend(
                constraint
                    .allowed_targets
                    .iter()
                    .map(|target| normalize(target, case_sensitive)),
            );
            accepted.sort();
            accepted.dedup();
            let forbidden: Vec<String> = constraint
                .forbidden_targets
                .iter()
                .map(|target| normalize(target, case_sensitive))
                .collect();
            let counts: Vec<(String, usize)> = accepted
                .iter()
                .map(|needle| (needle.clone(), count_occurrences(&search_text, needle)))
                .collect();
            let preferred_occurrences = counts
                .iter()
                .find(|(needle, _)| *needle == preferred)
                .map(|(_, count)| *count)
                .unwrap_or_default();
            let distinct_accepted_variants = counts.iter().filter(|(_, count)| *count > 0).count();
            let forbidden_occurrences = forbidden
                .iter()
                .map(|needle| count_occurrences(&search_text, needle))
                .sum::<usize>();
            let allowed_max = counts.iter().map(|(_, count)| *count).max().unwrap_or_default();
            let consistency_ok = !constraint.consistency_required || distinct_accepted_variants <= 1;
            let passed = allowed_max >= constraint.minimum_occurrences
                && forbidden_occurrences == 0
                && consistency_ok;
            let mut evidence = vec![
                format!("preferred_occurrences={preferred_occurrences}"),
                format!("forbidden_occurrences={forbidden_occurrences}"),
                format!("distinct_accepted_variants={distinct_accepted_variants}"),
            ];
            for (needle, count) in counts {
                evidence.push(format!("target[{needle}]={count}"));
            }
            if constraint.consistency_required {
                evidence.push("consistency_required=true".into());
            }
            if !case_sensitive {
                evidence.push("case_insensitive".into());
            }
            ExpectationResult {
                invariant_index: index,
                invariant_type: format!("Glossary({})", constraint.id),
                passed,
                known_failure: None,
                severity: if passed { "info".into() } else { "major".into() },
                message: if passed {
                    format!("glossary constraint {} satisfied", constraint.id)
                } else {
                    format!("glossary constraint {} failed", constraint.id)
                },
                evidence,
            }
        })
        .collect()
}

fn normalize(text: &str, case_sensitive: bool) -> String {
    if case_sensitive {
        text.to_string()
    } else {
        text.to_lowercase()
    }
}

fn count_occurrences(text: &str, needle: &str) -> usize {
    if needle.is_empty() {
        return 0;
    }
    text.match_indices(needle).count()
}
