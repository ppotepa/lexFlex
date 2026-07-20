use crate::category::resolve_query_variables;
use crate::chart::ChartItem;
use crate::diagnostic::ParseError;
use crate::explain::DerivationSet;
use crate::input::ClauseMode;
use crate::metrics::ParseScore;
use lexflex_lingua::LinguaExpression;
use lexflex_model::{SemanticType, VariableId};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CompleteCandidateKind {
    Assertion,
    Goal,
}

#[derive(Debug, Clone)]
pub struct CompleteCandidate {
    pub kind: CompleteCandidateKind,
    pub expression: LinguaExpression,
    pub variables: BTreeMap<VariableId, SemanticType>,
    pub projection: Vec<VariableId>,
    pub derivations: DerivationSet,
    pub score: ParseScore,
}

pub enum CompleteCandidateOutcome {
    Valid(CompleteCandidate),
    Rejected(ParseError),
}

pub fn classify_complete_item(
    item: ChartItem,
    mode: ClauseMode,
) -> Result<CompleteCandidateOutcome, ParseError> {
    match mode {
        ClauseMode::Declarative => {
            if !item.meaning.query_variables.is_empty() {
                return Ok(CompleteCandidateOutcome::Rejected(
                    ParseError::UnexpectedQueryVariable,
                ));
            }
            Ok(CompleteCandidateOutcome::Valid(CompleteCandidate {
                kind: CompleteCandidateKind::Assertion,
                expression: item.meaning.expression,
                variables: BTreeMap::new(),
                projection: Vec::new(),
                derivations: item.derivations,
                score: item.score,
            }))
        }
        ClauseMode::Interrogative => {
            if item.meaning.query_variables.is_empty() {
                return Ok(CompleteCandidateOutcome::Rejected(
                    ParseError::QuestionWithoutProjection,
                ));
            }
            let projection = item.meaning.query_variables.keys().cloned().collect();
            let variables =
                match resolve_query_variables(&item.meaning.query_variables, &item.substitution) {
                    Ok(value) => value,
                    Err(ParseError::UnresolvedQueryCategoryType) => {
                        return Ok(CompleteCandidateOutcome::Rejected(
                            ParseError::UnresolvedQueryCategoryType,
                        ));
                    }
                    Err(error) => return Err(error),
                };
            Ok(CompleteCandidateOutcome::Valid(CompleteCandidate {
                kind: CompleteCandidateKind::Goal,
                expression: item.meaning.expression,
                variables,
                projection,
                derivations: item.derivations,
                score: item.score,
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::CategorySubstitution;
    use crate::explain::{DerivationNode, DerivationSet};
    use crate::id::TokenId;
    use crate::meaning::MeaningInstance;
    use crate::token::{Token, TokenKind};
    use lexflex_language::{CategoryType, CategoryTypeVariableId, SyntacticCategory};
    use lexflex_model::{SemanticType, SourceSpan, VariableId};

    fn item(query_variables: BTreeMap<VariableId, CategoryType>) -> ChartItem {
        let derivation = DerivationSet::singleton(DerivationNode::Lexical {
            token: Token {
                id: TokenId::new_unchecked("test-token"),
                surface: "test".into(),
                normalized: "test".into(),
                span: SourceSpan::new(0, 4).expect("span"),
                kind: TokenKind::Word,
            },
            sense: lexflex_language::LexicalSenseId::new_unchecked("test-sense"),
        })
        .expect("derivation");
        ChartItem::new(
            0,
            1,
            SyntacticCategory::sentence(),
            CategorySubstitution::default(),
            MeaningInstance {
                expression: lexflex_lingua::LinguaExpression::Value(true.into()),
                query_variables,
                semantic_nodes: 1,
                boolean_operator: None,
                boolean_scope_violations: 0,
            },
            ParseScore::lexical(0),
            derivation,
        )
        .expect("chart item")
    }

    #[test]
    fn valid_assertion_candidate_survives() {
        let result = classify_complete_item(item(BTreeMap::new()), ClauseMode::Declarative)
            .expect("classification");
        assert!(matches!(result, CompleteCandidateOutcome::Valid(candidate)
            if candidate.kind == CompleteCandidateKind::Assertion));
    }

    #[test]
    fn declarative_query_candidate_is_rejected() {
        let variables = BTreeMap::from([(
            VariableId::new_unchecked("query"),
            CategoryType::Concrete(SemanticType::Entity),
        )]);
        let result = classify_complete_item(item(variables), ClauseMode::Declarative)
            .expect("classification");
        assert!(matches!(
            result,
            CompleteCandidateOutcome::Rejected(ParseError::UnexpectedQueryVariable)
        ));
    }

    #[test]
    fn unresolved_goal_candidate_is_rejected() {
        let variables = BTreeMap::from([(
            VariableId::new_unchecked("query"),
            CategoryType::Variable(CategoryTypeVariableId::new_unchecked("missing")),
        )]);
        let result = classify_complete_item(item(variables), ClauseMode::Interrogative)
            .expect("classification");
        assert!(matches!(
            result,
            CompleteCandidateOutcome::Rejected(ParseError::UnresolvedQueryCategoryType)
        ));
    }
}
