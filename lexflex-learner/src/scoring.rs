//! Pure, testable scoring logic for concept inference.
//! Extracted so AC3 invariants (exact match preferred, no fallback when is_a present,
//! no language-specific surface ifs) can be unit-tested without I/O or async.

use std::collections::HashSet;

/// Pure scoring from collected is_a labels.
/// Returns Some ONLY on exact isa.to_uppercase() membership in known_concepts (from concepts.ron).
/// Per AC3 + strategy: no maps, no contains heuristics.
pub fn score_from_is_a(is_a_lower: &[String], known_concepts: &[String]) -> Option<(String, f32, String)> {
    let known_set: HashSet<_> = known_concepts.iter().cloned().collect();
    for isa in is_a_lower {
        let u = isa.to_uppercase();
        if known_set.contains(&u) {
            return Some((u, 0.90, format!("direct from concepts.ron: {}", isa)));
        }
    }
    None
}

/// Narrow scoped context rule for age expressions (numeric + age/year marker in context).
/// Extracted per strategy for AC3: pure, testable, only exception kept.
pub fn narrow_context_score(word: &str, context_lower: &str) -> Option<(String, f32, String)> {
    if word.parse::<i32>().is_ok() && (context_lower.contains("age") || context_lower.contains("year")) {
        return Some(("YEAR".to_string(), 0.75, "scoped age context (number near age marker)".to_string()));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_from_is_a_exact() {
        let known = vec!["WIFE".to_string(), "PERSON".to_string()];
        let res = score_from_is_a(&["wife".to_string()], &known);
        assert!(res.is_some());
        let (c, _, r) = res.unwrap();
        assert_eq!(c, "WIFE");
        assert!(r.contains("direct from concepts"));
    }

    #[test]
    fn test_score_from_is_a_unmapped_returns_none() {
        // Critical AC3: is_a present but unmappable -> None (no bucket, exact only)
        let known = vec!["PERSON".to_string()];
        let res = score_from_is_a(&["together".to_string()], &known);
        assert!(res.is_none());
    }

    #[test]
    fn test_score_from_is_a_known_id() {
        let known = vec!["SWEET_ADJ".to_string()];
        let res = score_from_is_a(&["sweet_adj".to_string()], &known);
        assert!(res.is_some());
        let (c, _, r) = res.unwrap();
        assert_eq!(c, "SWEET_ADJ");
    }

    #[test]
    fn test_narrow_context_score_age() {
        let res = narrow_context_score("27", "mam 27 lat");
        assert!(res.is_none()); // no age/year marker
        let res = narrow_context_score("27", "age 27");
        assert!(res.is_some());
        let (c, _, _) = res.unwrap();
        assert_eq!(c, "YEAR");
    }
}
