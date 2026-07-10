use serde::{Deserialize, Serialize};

use crate::core::interlingua::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorphParadigm {
    pub name: String,
    pub rules: Vec<MorphRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorphRule {
    pub conditions: Vec<Condition>,
    pub operations: Vec<Operation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition {
    CaseIs(Case),
    NumberIs(Number),
    TenseIs(Tense),
    PersonIs(Person),
    GenderIs(Gender),
    AspectIs(Aspect),
    MoodIs(Mood),
    AnimacyIs(Animacy),
    DegreeIs(crate::core::interlingua::Degree),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operation {
    AddSuffix(String),
    ReplaceSuffix { from: String, to: String },
    Truncate(usize),
    ConsonantAlternate { from: String, to: String },
    Prefix(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorphException {
    pub conditions: Vec<Condition>,
    pub override_form: String,
}

pub fn apply_rules(
    lemma: &str,
    rules: &[MorphRule],
    target_features: &FeatureBundle,
) -> Option<String> {
    for rule in rules {
        if conditions_match(&rule.conditions, target_features) {
            return Some(apply_operations(lemma, &rule.operations));
        }
    }
    None
}

fn conditions_match(conditions: &[Condition], features: &FeatureBundle) -> bool {
    conditions.iter().all(|cond| match cond {
        Condition::CaseIs(c) => features.case == Some(*c),
        Condition::NumberIs(n) => features.number == Some(*n),
        Condition::TenseIs(t) => features.tense == Some(*t),
        Condition::PersonIs(p) => features.person == Some(*p),
        Condition::GenderIs(g) => features.gender == Some(*g),
        Condition::AspectIs(a) => features.aspect == Some(*a),
        Condition::MoodIs(m) => features.mood == Some(*m),
        Condition::AnimacyIs(a) => features.animacy == Some(*a),
        Condition::DegreeIs(d) => features.degree == Some(*d),
    })
}

fn apply_operations(stem: &str, operations: &[Operation]) -> String {
    let mut result = stem.to_string();

    for op in operations {
        match op {
            Operation::Truncate(n) => {
                let len = result.len().saturating_sub(*n);
                result.truncate(len);
            }
            Operation::AddSuffix(suffix) => {
                result.push_str(suffix);
            }
            Operation::ReplaceSuffix { from, to } => {
                if result.ends_with(from.as_str()) {
                    let new_len = result.len() - from.len();
                    result.truncate(new_len);
                    result.push_str(to);
                }
            }
            Operation::ConsonantAlternate { from, to } => {
                result = result.replace(from.as_str(), to.as_str());
            }
            Operation::Prefix(pre) => {
                result = format!("{}{}", pre, result);
            }
        }
    }

    result
}

/// Lightweight reverse of operations to recover a candidate stem from surface form.
/// Used for parser morphological analysis to exercise the RON-defined paradigms.
pub fn reverse_to_stem(form: &str, operations: &[Operation]) -> Option<String> {
    let mut result = form.to_string();
    for op in operations.iter().rev() {
        match op {
            Operation::AddSuffix(suf) => {
                if result.ends_with(suf.as_str()) {
                    let new_len = result.len() - suf.len();
                    result.truncate(new_len);
                } else {
                    return None;
                }
            }
            Operation::ReplaceSuffix { from, to } => {
                if result.ends_with(to.as_str()) {
                    let new_len = result.len() - to.len();
                    result.truncate(new_len);
                    result.push_str(from);
                } else {
                    return None;
                }
            }
            Operation::Truncate(n) => {
                // Reverse of truncate would require knowing removed content; approximate by no-op for analysis
                let _ = n;
            }
            Operation::ConsonantAlternate { from, to } => {
                result = result.replace(to.as_str(), from.as_str());
            }
            Operation::Prefix(pre) => {
                if result.starts_with(pre.as_str()) {
                    result = result[pre.len()..].to_string();
                } else {
                    return None;
                }
            }
        }
    }
    if result.is_empty() || result.len() > form.len() + 5 {
        return None;
    }
    Some(result)
}

// Supplet logic now data-driven exclusively via lexicon suppletive_* features + RON rules (no hardcoded tables).
// Parsers use analyze_via_paradigms; inflect uses features if present. See lexicon.ron for entries.

/// Attempt to analyze a surface form by reversing paradigm rules from RON data.
/// Returns enriched FeatureBundle if a plausible match found (tense/person etc from rule conditions).
pub fn analyze_via_paradigms(form: &str, paradigms: &[MorphParadigm]) -> Option<FeatureBundle> {
    for p in paradigms {
        for rule in &p.rules {
            if let Some(_stem) = reverse_to_stem(form, &rule.operations) {
                let mut fb = FeatureBundle::default();
                for cond in &rule.conditions {
                    match cond {
                        Condition::TenseIs(t) => fb.tense = Some(*t),
                        Condition::PersonIs(p) => fb.person = Some(*p),
                        Condition::NumberIs(n) => fb.number = Some(*n),
                        Condition::GenderIs(g) => fb.gender = Some(*g),
                        Condition::AspectIs(a) => fb.aspect = Some(*a),
                        Condition::CaseIs(c) => fb.case = Some(*c),
                        Condition::DegreeIs(d) => fb.degree = Some(*d),
                        _ => {}
                    }
                }
                return Some(fb);
            }
        }
    }
    None
}
