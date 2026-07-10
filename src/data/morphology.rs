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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operation {
    AddSuffix(String),
    ReplaceSuffix { from: String, to: String },
    Truncate(usize),
    ConsonantAlternate { from: String, to: String },
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
        }
    }

    result
}
