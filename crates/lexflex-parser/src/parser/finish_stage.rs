use crate::category::resolve_query_variables;
use crate::chart::Chart;
use crate::diagnostic::{ParseBudgetLimit, ParseError};
use crate::input::ClauseMode;
use crate::output::{AssertionDraft, GoalDraft, ParseAlternative, ParseOutput};
use crate::parser::context::ParseContext;
use lexflex_model::{canonical_hash, CanonicalDigest, SourceSpan};
use std::collections::BTreeMap;

pub fn finish_stage(
    mut ctx: ParseContext,
    chart: Chart,
) -> Result<ParseOutput, ParseError> {
    let complete = chart
        .cell(0, ctx.words.len())
        .map(|cell| cell.values().cloned().collect::<Vec<_>>())
        .unwrap_or_default()
        .into_iter()
        .filter(|item| item.category.is_sentence())
        .collect::<Vec<_>>();

    if complete.is_empty() {
        return Err(ParseError::NoParse);
    }

    let grouped = group_complete_items(complete)?;
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
            alternatives: best
                .into_iter()
                .map(|(_, item)| {
                    let query_variables = resolve_query_variables(
                        &item.meaning.query_variables,
                        &item.substitution,
                    )?;
                    Ok(ParseAlternative {
                        expression: item.meaning.expression,
                        query_variables,
                        projection: item.meaning.query_variables.keys().cloned().collect(),
                        derivation: item.derivations.primary().clone(),
                        score: item.score,
                        metrics: ctx.metrics.clone(),
                    })
                })
                .collect::<Result<Vec<_>, ParseError>>()?,
            metrics: ctx.metrics,
        });
    }

    let (_, item) = best.into_iter().next().ok_or(ParseError::NoParse)?;

    let span = SourceSpan::new(0, ctx.input.text.len() as u64)
        .map_err(|_| ParseError::InvalidSpan {
            start_byte: 0,
            end_byte: ctx.input.text.len() as u64,
        })?;

    match ctx.tokenization.mode {
        ClauseMode::Declarative => {
            if !item.meaning.query_variables.is_empty() {
                return Err(ParseError::UnexpectedQueryVariable);
            }
            Ok(ParseOutput::Assertion(AssertionDraft {
                source_id: ctx.input.source_id,
                language: ctx.input.language,
                span,
                expression: item.meaning.expression,
                derivation: item.derivations.primary().clone(),
                metrics: ctx.metrics,
            }))
        }
        ClauseMode::Interrogative => {
            if item.meaning.query_variables.is_empty() {
                return Err(ParseError::QuestionWithoutProjection);
            }
            let projection = item.meaning.query_variables.keys().cloned().collect();
            let variables = resolve_query_variables(&item.meaning.query_variables, &item.substitution)?;
            Ok(ParseOutput::Goal(GoalDraft {
                source_id: ctx.input.source_id,
                language: ctx.input.language,
                span,
                expression: item.meaning.expression,
                variables,
                projection,
                mode: ctx.tokenization.mode,
                derivation: item.derivations.primary().clone(),
                metrics: ctx.metrics,
            }))
        }
    }
}

fn group_complete_items(
    complete: Vec<crate::chart::ChartItem>,
) -> Result<Vec<(CompleteSemanticKey, crate::chart::ChartItem)>, ParseError> {
    let mut grouped: BTreeMap<CompleteSemanticKey, crate::chart::ChartItem> = BTreeMap::new();
    for item in complete {
        let query_variables = resolve_query_variables(&item.meaning.query_variables, &item.substitution)?;
        let projection = item
            .meaning
            .query_variables
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        let key = CompleteSemanticKey {
            expression_hash: canonical_hash(&item.meaning.expression)?,
            variable_hash: canonical_hash(&query_variables)?,
            projection_hash: canonical_hash(&projection)?,
        };
        match grouped.get(&key) {
            None => {
                grouped.insert(key, item);
            }
            Some(existing) if item.score < existing.score => {
                grouped.insert(key, item);
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CompleteSemanticKey {
    expression_hash: CanonicalDigest,
    variable_hash: CanonicalDigest,
    projection_hash: CanonicalDigest,
}