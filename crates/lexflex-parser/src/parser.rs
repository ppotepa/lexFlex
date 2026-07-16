use crate::budget::ParseBudget;
use crate::diagnostic::{ParseBudgetLimit, ParseError};
use crate::input::ParseInput;
use crate::output::{AssertionDraft, GoalDraft, ParseAlternative, ParseOutput};
use crate::token::{tokenize, Token, TokenKind};
use lexflex_language::LanguageModel;
use std::collections::BTreeMap;
use std::sync::Arc;

#[path = "parser/chart.rs"]
mod chart;
#[path = "parser/meaning.rs"]
mod meaning;

pub use chart::DerivationNode;
use chart::{compose, insert_item, ChartItem};
use meaning::{
    candidate_order, meaning_instance, score_candidate, Candidate, CategoryFeatureMerge,
};

pub struct LexicalCompositionParser {
    language: Arc<LanguageModel>,
    budget: ParseBudget,
}

impl LexicalCompositionParser {
    pub fn new(language: Arc<LanguageModel>) -> Self {
        Self {
            language,
            budget: ParseBudget::default(),
        }
    }

    pub fn with_budget(language: Arc<LanguageModel>, budget: ParseBudget) -> Self {
        Self { language, budget }
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
        let words = tokenization
            .tokens
            .iter()
            .filter(|token| token.kind == TokenKind::Word)
            .cloned()
            .collect::<Vec<_>>();
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
        let Some(best_score) = complete.iter().map(|item| item.score).min() else {
            return Err(ParseError::NoParse);
        };
        let best = complete
            .into_iter()
            .filter(|item| item.score == best_score)
            .collect::<Vec<_>>();
        if best.len() > 1 {
            let alternatives = best
                .into_iter()
                .map(|item| ParseAlternative {
                    expression: item.meaning.expression,
                    derivation: item.derivation,
                    score: item.score,
                })
                .collect();
            return Ok(ParseOutput::Ambiguous { alternatives });
        }

        let item = best.into_iter().next().ok_or(ParseError::NoParse)?;
        match tokenization.mode {
            crate::input::ClauseMode::Declarative => {
                if !item.meaning.query_variables.is_empty() {
                    return Err(ParseError::UnexpectedQueryVariable);
                }
                Ok(ParseOutput::Assertion(AssertionDraft {
                    source_id: input.source_id,
                    language: input.language,
                    expression: item.meaning.expression,
                    derivation: item.derivation,
                }))
            }
            crate::input::ClauseMode::Interrogative => {
                if item.meaning.query_variables.is_empty() {
                    return Err(ParseError::QuestionWithoutProjection);
                }
                let projection = item.meaning.query_variables.keys().cloned().collect();
                Ok(ParseOutput::Goal(GoalDraft {
                    source_id: input.source_id,
                    language: input.language,
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
            for (form_index, form_id) in form_ids.iter().enumerate() {
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
                    candidates.push(Candidate {
                        token: token.clone(),
                        sense: sense.clone(),
                        category,
                        meaning: meaning_instance(sense),
                        score: score_candidate(form_index, sense),
                    });
                }
            }
            candidates.sort_by(candidate_order);
            candidates.truncate(self.budget.max_lexical_candidates_per_token);
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
                            if let Some(item) = compose(left, right) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::ParseInput;
    use crate::output::ParseOutput;
    use crate::token::normalize_surface;
    use lexflex_language::{
        FeatureStructure, Form, FormId, FormIndex, LanguageModel, LanguagePackageManifest, Lexeme,
        LexemeId, LexicalSense, LexicalSenseId, MeaningTemplate, MeaningTemplateId, SenseIndex,
        SyntacticCategory,
    };
    use lexflex_lingua::LinguaExpression;
    use lexflex_model::LanguageId;
    use std::collections::BTreeMap;
    use std::sync::Arc;

    fn ambiguous_language_model() -> Arc<LanguageModel> {
        let language = LanguageId::new("en").expect("language");
        let lexeme_id = LexemeId::new("lexeme:test:foo");
        let form_id = FormId::new("form:test:foo");
        let lexeme = Lexeme {
            id: lexeme_id.clone(),
            language: LanguageId::new("en").expect("language"),
            lemma: "foo".into(),
            normalized_lemma: "foo".into(),
        };
        let sense_left = LexicalSense {
            id: LexicalSenseId::new("sense:test:foo:left"),
            lexeme_id: lexeme_id.clone(),
            anchor: None,
            base_category: SyntacticCategory::sentence(),
            meaning: MeaningTemplate::new(
                MeaningTemplateId::new("meaning:test:foo:left"),
                LinguaExpression::Entity(lexflex_model::EntityId::new_unchecked("LEFT")),
                BTreeMap::new(),
            ),
            features: FeatureStructure::default(),
            valency: Vec::new(),
            priority: 0,
        };
        let sense_right = LexicalSense {
            id: LexicalSenseId::new("sense:test:foo:right"),
            lexeme_id: lexeme_id.clone(),
            anchor: None,
            base_category: SyntacticCategory::sentence(),
            meaning: MeaningTemplate::new(
                MeaningTemplateId::new("meaning:test:foo:right"),
                LinguaExpression::Entity(lexflex_model::EntityId::new_unchecked("RIGHT")),
                BTreeMap::new(),
            ),
            features: FeatureStructure::default(),
            valency: Vec::new(),
            priority: 0,
        };
        let forms = vec![Form {
            id: form_id,
            lexeme_id: lexeme_id.clone(),
            surface: "foo".into(),
            normalized: normalize_surface("foo"),
            features: FeatureStructure::default(),
            priority: 0,
        }];
        let senses = BTreeMap::from([
            (
                LexicalSenseId::new("sense:test:foo:left"),
                sense_left.clone(),
            ),
            (
                LexicalSenseId::new("sense:test:foo:right"),
                sense_right.clone(),
            ),
        ]);
        Arc::new(LanguageModel {
            manifest: LanguagePackageManifest {
                schema: 1,
                package_id: "lexflex:language:test:ambiguous".into(),
                language,
                lexemes: "lexemes.ron".into(),
                senses: "senses.ron".into(),
                forms: "forms.ron".into(),
                paradigms: "paradigms.ron".into(),
            },
            lexemes: BTreeMap::from([(lexeme.id.clone(), lexeme)]),
            senses: BTreeMap::from([
                (LexicalSenseId::new("sense:test:foo:left"), sense_left),
                (LexicalSenseId::new("sense:test:foo:right"), sense_right),
            ]),
            compiled_senses: BTreeMap::from([
                (
                    LexicalSenseId::new("sense:test:foo:left"),
                    lexflex_language::CompiledLexicalSense {
                        id: LexicalSenseId::new("sense:test:foo:left"),
                        lexeme_id: lexeme_id.clone(),
                        anchor: None,
                        category: SyntacticCategory::sentence(),
                        meaning: lexflex_language::MeaningTemplate::new(
                            lexflex_language::MeaningTemplateId::new("meaning:test:foo:left"),
                            LinguaExpression::Entity(lexflex_model::EntityId::new_unchecked(
                                "LEFT",
                            )),
                            BTreeMap::new(),
                        ),
                        features: FeatureStructure::default(),
                        priority: 0,
                    },
                ),
                (
                    LexicalSenseId::new("sense:test:foo:right"),
                    lexflex_language::CompiledLexicalSense {
                        id: LexicalSenseId::new("sense:test:foo:right"),
                        lexeme_id: lexeme_id.clone(),
                        anchor: None,
                        category: SyntacticCategory::sentence(),
                        meaning: lexflex_language::MeaningTemplate::new(
                            lexflex_language::MeaningTemplateId::new("meaning:test:foo:right"),
                            LinguaExpression::Entity(lexflex_model::EntityId::new_unchecked(
                                "RIGHT",
                            )),
                            BTreeMap::new(),
                        ),
                        features: FeatureStructure::default(),
                        priority: 0,
                    },
                ),
            ]),
            forms: BTreeMap::from([(forms[0].id.clone(), forms[0].clone())]),
            paradigms: BTreeMap::new(),
            form_index: FormIndex::build(forms.clone()),
            sense_index: SenseIndex::build(senses.clone().into_values()),
            model_hash: String::new(),
        })
    }

    #[test]
    fn ambiguous_parse_carries_derivations() {
        let parser = LexicalCompositionParser::with_budget(
            ambiguous_language_model(),
            ParseBudget::default(),
        );
        let output = parser
            .parse(ParseInput {
                source_id: "test:ambiguous".into(),
                language: LanguageId::new("en").expect("language"),
                text: "foo".into(),
            })
            .expect("parse");

        match output {
            ParseOutput::Ambiguous { alternatives } => {
                assert_eq!(alternatives.len(), 2);
                assert!(alternatives
                    .iter()
                    .all(|alt| matches!(alt.derivation, DerivationNode::Lexical { .. })));
            }
            other => panic!("unexpected output: {other:?}"),
        }
    }
}
