use crate::budget::ParseBudget;
use crate::chart::{compose, insert_item, ChartItem};
use crate::diagnostic::{ParseBudgetLimit, ParseError};
use crate::explain::DerivationNode;
use crate::input::{ClauseMode, ParseInput};
use crate::lexical::{candidate_order, score_candidate, Candidate, CategoryFeatureMerge};
use crate::meaning::instantiate_meaning;
use crate::output::{AssertionDraft, GoalDraft, ParseAlternative, ParseOutput};
use crate::token::{tokenize, Token, TokenKind};
use lexflex_language::LanguageModel;
use lexflex_model::{canonical_hash, ConceptCatalog, SourceSpan};
use std::collections::BTreeMap;
use std::sync::Arc;

pub struct LexicalCompositionParser {
    catalog: Arc<ConceptCatalog>,
    language: Arc<LanguageModel>,
    budget: ParseBudget,
}

impl LexicalCompositionParser {
    pub fn new(catalog: Arc<ConceptCatalog>, language: Arc<LanguageModel>) -> Self {
        Self {
            catalog,
            language,
            budget: ParseBudget::default(),
        }
    }

    pub fn with_budget(
        catalog: Arc<ConceptCatalog>,
        language: Arc<LanguageModel>,
        budget: ParseBudget,
    ) -> Self {
        Self {
            catalog,
            language,
            budget,
        }
    }

    pub fn parse(&self, input: ParseInput) -> Result<ParseOutput, ParseError> {
        if input.text.is_empty() {
            return Err(ParseError::EmptyInput);
        }
        if self.language.manifest.language.as_str() != input.language.as_str() {
            return Err(ParseError::UnsupportedLanguage(input.language));
        }
        let tokenization = tokenize(&input)?;
        if tokenization.tokens.len() > self.budget.max_tokens {
            return Err(ParseError::BudgetExceeded(ParseBudgetLimit::TokenLimit));
        }
        let words = collect_words(&tokenization.tokens)?;
        let candidates = self.lexical_candidates(&words)?;
        let chart = self.build_chart(&candidates)?;
        let complete = chart
            .get(&(0, words.len()))
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter(|item| item.category.is_sentence())
            .collect::<Vec<_>>();
        if complete.is_empty() {
            return Err(ParseError::NoParse);
        }
        if complete.len() > self.budget.max_complete_parses {
            return Err(ParseError::BudgetExceeded(
                ParseBudgetLimit::CompleteParseLimit,
            ));
        }
        let Some(best_score) = complete.iter().map(|item| item.score).min() else {
            return Err(ParseError::NoParse);
        };
        let best = complete
            .into_iter()
            .filter(|item| item.score == best_score)
            .collect::<Vec<_>>();
        if best.len() > 1 {
            return Ok(ParseOutput::Ambiguous {
                alternatives: best
                    .into_iter()
                    .map(|item| ParseAlternative {
                        expression: item.meaning.expression,
                        derivation: item.derivation,
                        score: item.score,
                    })
                    .collect(),
            });
        }

        let item = best.into_iter().next().ok_or(ParseError::NoParse)?;
        let span =
            SourceSpan::new(0, input.text.len() as u64).map_err(|_| ParseError::InvalidSpan {
                start_byte: 0,
                end_byte: input.text.len() as u64,
            })?;
        match tokenization.mode {
            ClauseMode::Declarative => {
                if !item.meaning.query_variables.is_empty() {
                    return Err(ParseError::UnexpectedQueryVariable);
                }
                Ok(ParseOutput::Assertion(AssertionDraft {
                    source_id: input.source_id,
                    language: input.language,
                    span,
                    expression: item.meaning.expression,
                    derivation: item.derivation,
                }))
            }
            ClauseMode::Interrogative => {
                if item.meaning.query_variables.is_empty() {
                    return Err(ParseError::QuestionWithoutProjection);
                }
                let projection = item.meaning.query_variables.keys().cloned().collect();
                Ok(ParseOutput::Goal(GoalDraft {
                    source_id: input.source_id,
                    language: input.language,
                    span,
                    expression: item.meaning.expression,
                    variables: item.meaning.query_variables,
                    projection,
                    mode: tokenization.mode,
                    derivation: item.derivation,
                }))
            }
        }
    }

    fn lexical_candidates(&self, words: &[Token]) -> Result<Vec<Vec<Candidate>>, ParseError> {
        let mut output = Vec::with_capacity(words.len());
        for token in words {
            let form_ids = self.language.form_index.lookup(&token.normalized);
            if form_ids.is_empty() {
                return Err(ParseError::UnknownSurface {
                    token: token.surface.clone(),
                });
            }
            let mut candidates = Vec::new();
            for form_id in form_ids {
                let Some(form) = self.language.form(form_id) else {
                    continue;
                };
                let sense_ids = self.language.sense_index.lookup(&form.lexeme_id);
                for sense_id in sense_ids {
                    let Some(sense) = self.language.compiled_sense(sense_id) else {
                        continue;
                    };
                    let Some(category) = sense.category.unify_features(&form.features) else {
                        continue;
                    };
                    let seed =
                        canonical_hash(&(token.id.as_str(), form.id.as_str(), sense.id.as_str()));
                    let (category, meaning) = instantiate_meaning(&seed, &category, sense)?;
                    if meaning.semantic_nodes > self.budget.max_semantic_nodes {
                        return Err(ParseError::BudgetExceeded(
                            ParseBudgetLimit::SemanticNodeLimit,
                        ));
                    }
                    candidates.push(Candidate {
                        token: token.clone(),
                        sense: sense.clone(),
                        category,
                        meaning,
                        score: score_candidate(form, sense),
                    });
                }
            }
            candidates.sort_by(candidate_order);
            if candidates.len() > self.budget.max_lexical_candidates_per_token {
                return Err(ParseError::BudgetExceeded(
                    ParseBudgetLimit::LexicalCandidateLimit,
                ));
            }
            output.push(candidates);
        }
        Ok(output)
    }

    fn build_chart(
        &self,
        candidates: &[Vec<Candidate>],
    ) -> Result<BTreeMap<(usize, usize), Vec<ChartItem>>, ParseError> {
        let token_count = candidates.len();
        let mut chart: BTreeMap<(usize, usize), Vec<ChartItem>> = BTreeMap::new();
        let mut total_items = 0usize;
        for (index, bucket) in candidates.iter().enumerate() {
            let cell = chart.entry((index, index + 1)).or_default();
            for candidate in bucket {
                insert_item(
                    cell,
                    ChartItem {
                        start: index,
                        end: index + 1,
                        category: candidate.category.clone(),
                        meaning: candidate.meaning.clone(),
                        score: candidate.score,
                        derivation: DerivationNode::Lexical {
                            token: candidate.token.clone(),
                            sense: candidate.sense.id.clone(),
                        },
                    },
                    self.budget.max_items_per_cell,
                )?;
                total_items += 1;
            }
        }
        for span in 2..=token_count {
            for start in 0..=token_count - span {
                let end = start + span;
                let mut cell_items = Vec::new();
                for split in start + 1..end {
                    let left_items = chart.get(&(start, split)).cloned().unwrap_or_default();
                    let right_items = chart.get(&(split, end)).cloned().unwrap_or_default();
                    for left in &left_items {
                        for right in &right_items {
                            if let Some(item) = compose(
                                left,
                                right,
                                self.catalog.as_ref(),
                                self.budget.max_semantic_nodes,
                            )? {
                                if item.derivation.depth() > self.budget.max_derivation_depth {
                                    return Err(ParseError::BudgetExceeded(
                                        ParseBudgetLimit::DerivationDepthLimit,
                                    ));
                                }
                                cell_items.push(item);
                            }
                        }
                    }
                }
                let cell = chart.entry((start, end)).or_default();
                for item in cell_items {
                    insert_item(cell, item, self.budget.max_items_per_cell)?;
                    total_items += 1;
                    if total_items > self.budget.max_total_items {
                        return Err(ParseError::BudgetExceeded(ParseBudgetLimit::TotalItemLimit));
                    }
                }
            }
        }
        Ok(chart)
    }
}

fn collect_words(tokens: &[Token]) -> Result<Vec<Token>, ParseError> {
    let punctuation = tokens
        .iter()
        .filter(|token| token.kind == TokenKind::Punctuation)
        .cloned()
        .collect::<Vec<_>>();
    let words = tokens
        .iter()
        .filter(|token| token.kind == TokenKind::Word)
        .cloned()
        .collect::<Vec<_>>();
    if words.is_empty() {
        return Err(ParseError::EmptyInput);
    }

    match punctuation.as_slice() {
        [] => Ok(words),
        [token] if token.surface == "." || token.surface == "?" || token.surface == "!" => {
            if Some(token.id.clone()) == tokens.last().map(|value| value.id.clone()) {
                Ok(words)
            } else {
                Err(ParseError::UnsupportedPunctuationLayout {
                    token: token.clone(),
                })
            }
        }
        [token, ..] => Err(ParseError::UnsupportedPunctuationLayout {
            token: token.clone(),
        }),
    }
}
