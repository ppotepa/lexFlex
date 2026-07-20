use crate::meaning::MeaningInstance;
use crate::token::Token;
use crate::ParseScore;
use lexflex_language::{CompiledLexicalSense, FeatureStructure, Form, SyntacticCategory};

#[derive(Debug, Clone)]
pub(crate) struct Candidate {
    pub(crate) token: Token,
    pub(crate) sense: CompiledLexicalSense,
    pub(crate) category: SyntacticCategory,
    pub(crate) meaning: MeaningInstance,
    pub(crate) score: ParseScore,
}

pub(crate) fn score_candidate(form: &Form, sense: &CompiledLexicalSense) -> ParseScore {
    ParseScore::lexical(i64::from(form.priority) + i64::from(sense.priority))
}

pub(crate) fn candidate_order(left: &Candidate, right: &Candidate) -> std::cmp::Ordering {
    left.score
        .cmp(&right.score)
        .then_with(|| left.category.cmp(&right.category))
        .then_with(|| left.sense.id.cmp(&right.sense.id))
}

pub(crate) trait CategoryFeatureMerge {
    fn unify_features(&self, features: &FeatureStructure) -> Option<SyntacticCategory>;
}

impl CategoryFeatureMerge for SyntacticCategory {
    fn unify_features(&self, features: &FeatureStructure) -> Option<SyntacticCategory> {
        match self {
            SyntacticCategory::Atom {
                kind,
                semantic_type,
                features: category_features,
            } => Some(SyntacticCategory::Atom {
                kind: kind.clone(),
                semantic_type: semantic_type.clone(),
                features: category_features.unify(features).ok()?,
            }),
            SyntacticCategory::Function {
                result,
                argument,
                direction,
                semantic_parameter,
                features: category_features,
            } => Some(SyntacticCategory::Function {
                result: result.clone(),
                argument: argument.clone(),
                direction: *direction,
                semantic_parameter: semantic_parameter.clone(),
                features: category_features.unify(features).ok()?,
            }),
        }
    }
}
