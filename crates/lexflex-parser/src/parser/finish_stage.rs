use crate::chart::Chart;
use crate::diagnostic::{ParseBudgetLimit, ParseError};
use crate::output::{AssertionDraft, GoalDraft, ParseAlternative, ParseOutput};
use crate::parser::complete_candidate::{
    classify_complete_item, CompleteCandidate, CompleteCandidateKind, CompleteCandidateOutcome,
};
use crate::parser::context::ParseContext;
use lexflex_model::{canonical_hash, CanonicalDigest, SourceSpan};
use std::collections::BTreeMap;

pub fn finish_stage(mut ctx: ParseContext, chart: Chart) -> Result<ParseOutput, ParseError> {
    let complete = chart
        .cell(0, ctx.words.len())
        .map(|cell| cell.values().cloned().collect::<Vec<_>>())
        .unwrap_or_default()
        .into_iter()
        .filter(|item| item.category.is_sentence())
        .collect::<Vec<_>>();

    ctx.metrics.complete_candidate_count = complete.len();

    if complete.is_empty() {
        return Err(ParseError::NoParse);
    }

    let mut valid = Vec::new();
    let mut rejected = Vec::new();
    for item in complete {
        match classify_complete_item(item, ctx.tokenization.mode)? {
            CompleteCandidateOutcome::Valid(candidate) => valid.push(candidate),
            CompleteCandidateOutcome::Rejected(error) => {
                ctx.metrics.complete_rejection_count += 1;
                rejected.push(error);
            }
        }
    }

    if valid.is_empty() {
        return Err(select_complete_rejection(rejected).unwrap_or(ParseError::NoParse));
    }

    let grouped = group_complete_candidates(valid, ctx.budget.max_derivations_per_item)?;
    ctx.metrics.complete_semantic_count = grouped.len();

    if grouped.len() > ctx.budget.max_complete_parses {
        return Err(ParseError::BudgetExceeded(
            ParseBudgetLimit::CompleteParseLimit,
        ));
    }

    let Some(best_score) = grouped.iter().map(|(_, item)| item.score).min() else {
        return Err(ParseError::NoParse);
    };

    let best = grouped
        .into_iter()
        .filter(|(_, item)| item.score == best_score)
        .collect::<Vec<_>>();

    if best.len() > 1 {
        return Ok(ParseOutput::Ambiguous {
            mode: ctx.tokenization.mode,
            alternatives: best
                .into_iter()
                .map(|(_, item)| ParseAlternative {
                    expression: item.expression,
                    query_variables: item.variables,
                    projection: item.projection,
                    derivations: item.derivations,
                    score: item.score,
                    metrics: ctx.metrics.clone(),
                })
                .collect(),
            metrics: ctx.metrics,
        });
    }

    let (_, item) = best.into_iter().next().ok_or(ParseError::NoParse)?;

    let span =
        SourceSpan::new(0, ctx.input.text.len() as u64).map_err(|_| ParseError::InvalidSpan {
            start_byte: 0,
            end_byte: ctx.input.text.len() as u64,
        })?;

    match item.kind {
        CompleteCandidateKind::Assertion => Ok(ParseOutput::Assertion(AssertionDraft {
            source_id: ctx.input.source_id,
            language: ctx.input.language,
            span,
            expression: item.expression,
            derivations: item.derivations,
            metrics: ctx.metrics,
        })),
        CompleteCandidateKind::Goal => Ok(ParseOutput::Goal(GoalDraft {
            source_id: ctx.input.source_id,
            language: ctx.input.language,
            span,
            expression: item.expression,
            variables: item.variables,
            projection: item.projection,
            mode: ctx.tokenization.mode,
            derivations: item.derivations,
            metrics: ctx.metrics,
        })),
    }
}

fn group_complete_candidates(
    complete: Vec<CompleteCandidate>,
    derivation_limit: usize,
) -> Result<Vec<(CompleteSemanticKey, CompleteCandidate)>, ParseError> {
    let mut grouped: BTreeMap<CompleteSemanticKey, CompleteCandidate> = BTreeMap::new();
    for item in complete {
        let key = CompleteSemanticKey {
            kind: item.kind,
            expression_hash: canonical_hash(&item.expression)?,
            variable_hash: canonical_hash(&item.variables)?,
            projection_hash: canonical_hash(&item.projection)?,
        };
        match grouped.get_mut(&key) {
            None => {
                grouped.insert(key, item);
            }
            Some(existing) if item.score < existing.score => {
                *existing = item;
            }
            Some(existing) if item.score == existing.score => {
                existing
                    .derivations
                    .try_merge(&item.derivations, derivation_limit)?;
            }
            Some(_) => {}
        }
    }
    let mut output = grouped.into_iter().collect::<Vec<_>>();
    output.sort_by(|left, right| {
        left.1
            .score
            .cmp(&right.1.score)
            .then_with(|| left.0.cmp(&right.0))
    });
    Ok(output)
}

fn select_complete_rejection(mut rejected: Vec<ParseError>) -> Option<ParseError> {
    rejected.sort_by(|left, right| {
        complete_rejection_rank(left)
            .cmp(&complete_rejection_rank(right))
            .then_with(|| left.to_string().cmp(&right.to_string()))
    });
    rejected.into_iter().next()
}

fn complete_rejection_rank(error: &ParseError) -> u8 {
    match error {
        ParseError::UnresolvedQueryCategoryType => 0,
        ParseError::UnexpectedQueryVariable => 1,
        ParseError::QuestionWithoutProjection => 2,
        _ => 3,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CompleteSemanticKey {
    kind: CompleteCandidateKind,
    expression_hash: CanonicalDigest,
    variable_hash: CanonicalDigest,
    projection_hash: CanonicalDigest,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::explain::{DerivationNode, DerivationSet};
    use crate::id::TokenId;
    use crate::metrics::ParseScore;
    use crate::token::{Token, TokenKind};
    use lexflex_language::LexicalSenseId;
    use lexflex_model::SemanticValue;

    fn derivation(name: &str) -> DerivationSet {
        DerivationSet::singleton(DerivationNode::Lexical {
            token: Token {
                id: TokenId::new_unchecked(format!("token-{name}")),
                surface: name.into(),
                normalized: name.into(),
                span: SourceSpan::new(0, 1).expect("span"),
                kind: TokenKind::Word,
            },
            sense: LexicalSenseId::new_unchecked(format!("sense-{name}")),
        })
        .expect("derivation")
    }

    fn candidate(name: &str, score: ParseScore) -> CompleteCandidate {
        CompleteCandidate {
            kind: CompleteCandidateKind::Assertion,
            expression: lexflex_lingua::LinguaExpression::Value(SemanticValue::Boolean(true)),
            variables: BTreeMap::new(),
            projection: Vec::new(),
            derivations: derivation(name),
            score,
        }
    }

    #[test]
    fn equal_semantics_merge_derivations() {
        let grouped = group_complete_candidates(
            vec![
                candidate("one", ParseScore::lexical(0)),
                candidate("two", ParseScore::lexical(0)),
            ],
            4,
        )
        .expect("grouping");
        assert_eq!(grouped.len(), 1);
        assert_eq!(grouped[0].1.derivations.len(), 2);
    }

    #[test]
    fn rejection_ranking_is_deterministic_and_prefers_unresolved() {
        let selected = select_complete_rejection(vec![
            ParseError::QuestionWithoutProjection,
            ParseError::UnexpectedQueryVariable,
            ParseError::UnresolvedQueryCategoryType,
        ])
        .expect("rejection");
        assert_eq!(selected, ParseError::UnresolvedQueryCategoryType);
    }

    #[test]
    fn equal_semantics_respect_derivation_limit() {
        let error = group_complete_candidates(
            vec![
                candidate("one", ParseScore::lexical(0)),
                candidate("two", ParseScore::lexical(0)),
            ],
            1,
        )
        .expect_err("derivation overflow must fail");
        assert!(matches!(error, ParseError::BudgetExceeded(_)));
    }
}
