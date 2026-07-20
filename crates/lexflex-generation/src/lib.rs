#![forbid(unsafe_code)]

mod agreement;
mod category;
mod morphology;
mod plan;
mod valency;

use lexflex_language::{FeatureStructure, LanguageModel, SemanticAnchor};
use lexflex_model::{
    ConceptId, EntityId, LanguageId, ResourceBudget, SemanticExpression, SemanticValue, VariableId,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub use plan::{build_generation_plan, GenerationNode, GenerationPlan};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenerationRequest {
    pub expression: SemanticExpression,
    pub include_trace: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneratedText {
    pub text: String,
    pub trace: Option<Vec<GenerationTrace>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GenerationResult {
    Generated(GeneratedText),
    Ambiguous { anchor: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenerationTrace {
    pub anchor: String,
    pub lexeme_id: String,
    #[serde(default)]
    pub events: Vec<GenerationTraceEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GenerationTraceEvent {
    SelectSense {
        sense_id: String,
    },
    SelectForm {
        form_id: String,
    },
    BindValency {
        parameter: String,
        relation: Option<String>,
    },
    ApplyAgreement {
        features: FeatureStructure,
    },
    Linearize {
        separator: String,
    },
    Emit {
        text: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticRealizationRequest {
    pub expression: SemanticExpression,
    pub kind: SemanticRealizationKind,
    pub projection: Vec<VariableId>,
    pub include_trace: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextTranslationRequest {
    pub input: String,
    pub source_language: LanguageId,
    pub target_language: LanguageId,
    pub ambiguity_policy: TranslationAmbiguityPolicy,
    pub include_trace: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TranslationAmbiguityPolicy {
    Reject,
    ReturnAlternatives,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SemanticRealizationKind {
    Assertion,
    Goal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenerationBudget {
    pub max_depth: usize,
    pub max_nodes: usize,
    pub max_output_bytes: usize,
}

impl Default for GenerationBudget {
    fn default() -> Self {
        Self {
            max_depth: 128,
            max_nodes: 10_000,
            max_output_bytes: ResourceBudget::default().max_generation_bytes,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenerationStyle {
    pub conjunction: String,
    pub disjunction: String,
    pub negation_prefix: String,
    pub existential_prefix: String,
    pub universal_prefix: String,
}

impl Default for GenerationStyle {
    fn default() -> Self {
        Self {
            conjunction: " and ".into(),
            disjunction: " or ".into(),
            negation_prefix: "not ".into(),
            existential_prefix: "there exists ".into(),
            universal_prefix: "for all ".into(),
        }
    }
}

impl GenerationStyle {
    pub fn from_language(language: &LanguageModel) -> Self {
        Self {
            conjunction: language.realizations.conjunction.clone(),
            disjunction: language.realizations.disjunction.clone(),
            negation_prefix: language.realizations.negation_prefix.clone(),
            existential_prefix: language.realizations.existential_prefix.clone(),
            universal_prefix: language.realizations.universal_prefix.clone(),
        }
    }

    fn equality_separator<'a>(&self, language: &'a LanguageModel) -> &'a str {
        &language.realizations.equality_separator
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum GenerationError {
    #[error("semantic expression canonicalization failed")]
    Canonicalization,
    #[error("no lexical realization for semantic expression")]
    NoRealization,
    #[error("unsupported semantic expression for lexical realization")]
    UnsupportedExpression,
    #[error("requested generation features conflict with the lexical sense")]
    IncompatibleFeatures,
    #[error("generation resource budget exceeded: {0}")]
    BudgetExceeded(&'static str),
    #[error("ambiguous lexical realization for {anchor}")]
    AmbiguousRealization { anchor: String },
    #[error("generation projection contains duplicate variable: {0}")]
    DuplicateProjection(VariableId),
    #[error("unknown valency binding: {0}")]
    UnknownValencyBinding(lexflex_model::ParameterId),
    #[error("required valency binding is missing: {0}")]
    MissingValencyBinding(lexflex_model::ParameterId),
}

fn structural_join(
    left: &SemanticExpression,
    right: &SemanticExpression,
    separator: &str,
    include_trace: bool,
    language: &LanguageModel,
    style: &GenerationStyle,
) -> Result<GeneratedText, GenerationError> {
    let left = generate_with_style_unchecked(
        &GenerationRequest {
            expression: left.clone(),
            include_trace,
        },
        language,
        style,
    )?;
    let right = generate_with_style_unchecked(
        &GenerationRequest {
            expression: right.clone(),
            include_trace,
        },
        language,
        style,
    )?;
    let trace = merge_traces(left.trace, right.trace);
    let trace = trace.map(|mut events| {
        events.push(GenerationTrace {
            anchor: "structure".into(),
            lexeme_id: String::new(),
            events: vec![GenerationTraceEvent::Linearize {
                separator: separator.into(),
            }],
        });
        events
    });
    Ok(GeneratedText {
        text: format!("{}{}{}", left.text, separator, right.text),
        trace,
    })
}

fn merge_traces(
    left: Option<Vec<GenerationTrace>>,
    right: Option<Vec<GenerationTrace>>,
) -> Option<Vec<GenerationTrace>> {
    match (left, right) {
        (None, None) => None,
        (Some(mut left), Some(right)) => {
            left.extend(right);
            Some(left)
        }
        (Some(trace), None) | (None, Some(trace)) => Some(trace),
    }
}

pub fn generate_anchor_with_features(
    anchor: &SemanticAnchor,
    required_features: &FeatureStructure,
    include_trace: bool,
    language: &LanguageModel,
) -> Result<GeneratedText, GenerationError> {
    let mut candidates = language
        .compiled_senses
        .values()
        .filter(|sense| sense.anchor.as_ref() == Some(anchor))
        .filter_map(|sense| {
            let effective = match agreement::merge_subject_predicate_features(
                &sense.lexical_features,
                required_features,
            ) {
                Ok(effective) => effective,
                Err(_) => return Some(Err(GenerationError::IncompatibleFeatures)),
            };
            let has_form = language.forms.values().any(|form| {
                form.lexeme_id == sense.lexeme_id && form.features.contains_all(&effective)
            });
            if effective.values.is_empty() || has_form {
                Some(Ok((sense, effective)))
            } else {
                None
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    candidates.sort_by_key(|(sense, _)| (sense.priority, sense.id.clone()));
    let (sense, effective_features) = candidates
        .first()
        .cloned()
        .ok_or(GenerationError::NoRealization)?;
    let best_priority = sense.priority;
    let best_lexemes = candidates
        .iter()
        .filter(|(candidate, _)| candidate.priority == best_priority)
        .map(|(candidate, _)| candidate.lexeme_id.clone())
        .collect::<std::collections::BTreeSet<_>>();
    if best_lexemes.len() > 1 {
        return Err(GenerationError::AmbiguousRealization {
            anchor: format_anchor(anchor),
        });
    }
    if let Some(form) = morphology::select_form(sense, &effective_features, language) {
        return Ok(GeneratedText {
            text: form.surface.clone(),
            trace: include_trace.then(|| {
                vec![GenerationTrace {
                    anchor: format_anchor(anchor),
                    lexeme_id: sense.lexeme_id.to_string(),
                    events: vec![
                        GenerationTraceEvent::SelectSense {
                            sense_id: sense.id.to_string(),
                        },
                        GenerationTraceEvent::SelectForm {
                            form_id: form.id.to_string(),
                        },
                        GenerationTraceEvent::ApplyAgreement {
                            features: effective_features.clone(),
                        },
                        GenerationTraceEvent::Emit {
                            text: form.surface.clone(),
                        },
                    ],
                }]
            }),
        });
    }
    if effective_features.values.is_empty() {
        let lexeme = language
            .lexeme(&sense.lexeme_id)
            .ok_or(GenerationError::NoRealization)?;
        return Ok(GeneratedText {
            text: lexeme.lemma.clone(),
            trace: include_trace.then(|| {
                vec![GenerationTrace {
                    anchor: format_anchor(anchor),
                    lexeme_id: sense.lexeme_id.to_string(),
                    events: vec![
                        GenerationTraceEvent::SelectSense {
                            sense_id: sense.id.to_string(),
                        },
                        GenerationTraceEvent::Emit {
                            text: lexeme.lemma.clone(),
                        },
                    ],
                }]
            }),
        });
    }
    Err(GenerationError::NoRealization)
}

fn generate_basic(
    request: &GenerationRequest,
    language: &LanguageModel,
) -> Result<GeneratedText, GenerationError> {
    if let SemanticExpression::Value(value) = &request.expression {
        return Ok(GeneratedText {
            text: realize_value(value),
            trace: None,
        });
    }
    let anchor = match &request.expression {
        SemanticExpression::Entity(id) => SemanticAnchor::Entity(id.clone()),
        SemanticExpression::Concept(id) => SemanticAnchor::Concept(id.clone()),
        SemanticExpression::Variable(id) => {
            return Ok(GeneratedText {
                text: id.to_string(),
                trace: None,
            });
        }
        _ => return Err(GenerationError::UnsupportedExpression),
    };
    generate_anchor_with_features(
        &anchor,
        &FeatureStructure::default(),
        request.include_trace,
        language,
    )
}

pub fn generate(
    request: &GenerationRequest,
    language: &LanguageModel,
) -> Result<GeneratedText, GenerationError> {
    generate_with_budget(
        request,
        language,
        &GenerationStyle::from_language(language),
        GenerationBudget::default(),
    )
}

pub fn generate_result(
    request: &GenerationRequest,
    language: &LanguageModel,
) -> Result<GenerationResult, GenerationError> {
    match generate(request, language) {
        Ok(text) => Ok(GenerationResult::Generated(text)),
        Err(GenerationError::AmbiguousRealization { anchor }) => {
            Ok(GenerationResult::Ambiguous { anchor })
        }
        Err(error) => Err(error),
    }
}

pub fn generate_with_budget(
    request: &GenerationRequest,
    language: &LanguageModel,
    style: &GenerationStyle,
    budget: GenerationBudget,
) -> Result<GeneratedText, GenerationError> {
    if budget.max_depth == 0 || budget.max_nodes == 0 || budget.max_output_bytes == 0 {
        return Err(GenerationError::BudgetExceeded("invalid generation budget"));
    }
    let _plan = build_generation_plan(&request.expression, language)?;
    let mut nodes = 0;
    check_expression_budget(&request.expression, 0, &mut nodes, &budget)?;
    let generated = generate_with_style_unchecked(request, language, style)?;
    if generated.text.len() > budget.max_output_bytes {
        return Err(GenerationError::BudgetExceeded("output bytes"));
    }
    Ok(generated)
}

fn check_expression_budget(
    expression: &SemanticExpression,
    depth: usize,
    nodes: &mut usize,
    budget: &GenerationBudget,
) -> Result<(), GenerationError> {
    *nodes = nodes.saturating_add(1);
    if *nodes > budget.max_nodes {
        return Err(GenerationError::BudgetExceeded("expression nodes"));
    }
    if depth > budget.max_depth {
        return Err(GenerationError::BudgetExceeded("expression depth"));
    }
    let mut child = |expression: &SemanticExpression| {
        check_expression_budget(expression, depth + 1, nodes, budget)
    };
    match expression {
        SemanticExpression::Apply { bindings, .. } => {
            for expression in bindings.values() {
                child(expression)?;
            }
        }
        SemanticExpression::Satisfies { subject, predicate }
        | SemanticExpression::Equals {
            left: subject,
            right: predicate,
        } => {
            child(subject)?;
            child(predicate)?;
        }
        SemanticExpression::And(expressions) | SemanticExpression::Or(expressions) => {
            for expression in expressions {
                child(expression)?;
            }
        }
        SemanticExpression::Not(expression)
        | SemanticExpression::Exists {
            body: expression, ..
        }
        | SemanticExpression::ForAll {
            body: expression, ..
        } => child(expression)?,
        SemanticExpression::Qualified {
            expression,
            qualifiers,
        } => {
            child(expression)?;
            for expression in qualifiers.values() {
                child(expression)?;
            }
        }
        SemanticExpression::Concept(_)
        | SemanticExpression::Entity(_)
        | SemanticExpression::Value(_)
        | SemanticExpression::Variable(_) => {}
    }
    Ok(())
}

pub fn generate_with_style(
    request: &GenerationRequest,
    language: &LanguageModel,
    style: &GenerationStyle,
) -> Result<GeneratedText, GenerationError> {
    generate_with_budget(request, language, style, GenerationBudget::default())
}

fn generate_with_style_unchecked(
    request: &GenerationRequest,
    language: &LanguageModel,
    style: &GenerationStyle,
) -> Result<GeneratedText, GenerationError> {
    match &request.expression {
        SemanticExpression::And(expressions) => combine(
            expressions,
            &style.conjunction,
            request.include_trace,
            language,
            style,
        ),
        SemanticExpression::Or(expressions) => combine(
            expressions,
            &style.disjunction,
            request.include_trace,
            language,
            style,
        ),
        SemanticExpression::Not(expression) => {
            let nested = generate_with_style_unchecked(
                &GenerationRequest {
                    expression: (**expression).clone(),
                    include_trace: request.include_trace,
                },
                language,
                style,
            )?;
            Ok(GeneratedText {
                text: format!("{}{}", style.negation_prefix, nested.text),
                trace: nested.trace,
            })
        }
        expression @ SemanticExpression::Satisfies { .. } => {
            let clause = category::plan_satisfies(expression)
                .ok_or(GenerationError::UnsupportedExpression)?;
            let subject = generate_with_style_unchecked(
                &GenerationRequest {
                    expression: clause.subject.clone(),
                    include_trace: request.include_trace,
                },
                language,
                style,
            )?;
            let predicate =
                generate_predicate(clause.predicate, request.include_trace, language, style)?;
            Ok(GeneratedText {
                text: format!(
                    "{}{}{}{}",
                    subject.text,
                    language.realizations.equality_separator,
                    language.realizations.predicate_prefix,
                    predicate.text
                ),
                trace: merge_traces(subject.trace, predicate.trace),
            })
        }
        SemanticExpression::Equals { left, right } => structural_join(
            left,
            right,
            style.equality_separator(language),
            request.include_trace,
            language,
            style,
        ),
        SemanticExpression::Apply { concept, bindings } => {
            valency::validate_apply(concept, bindings, language)?;
            let head = generate_with_style_unchecked(
                &GenerationRequest {
                    expression: SemanticExpression::Concept(concept.clone()),
                    include_trace: request.include_trace,
                },
                language,
                style,
            )?;
            let mut arguments = Vec::with_capacity(bindings.len());
            let mut trace = head.trace;
            for binding in valency::ordered_bindings(concept, bindings, language) {
                let generated = generate_argument(
                    binding.expression,
                    binding.features,
                    request.include_trace,
                    language,
                    style,
                )?;
                trace = merge_traces(trace, generated.trace);
                let separator = binding
                    .relation
                    .and_then(|relation| language.realizations.surface_relations.get(relation))
                    .map(String::as_str)
                    .unwrap_or(" ");
                arguments.push(format!("{}{}", separator, generated.text));
                if request.include_trace {
                    if let Some(ref mut entries) = trace {
                        entries.push(GenerationTrace {
                            anchor: "valency".into(),
                            lexeme_id: binding.parameter.to_string(),
                            events: vec![GenerationTraceEvent::BindValency {
                                parameter: binding.parameter.to_string(),
                                relation: binding.relation.map(str::to_owned),
                            }],
                        });
                    }
                }
            }
            Ok(GeneratedText {
                text: if arguments.is_empty() {
                    head.text
                } else {
                    realize_apply_text(
                        &head.text,
                        arguments,
                        language
                            .realizations
                            .head_positions
                            .get(concept.as_str())
                            .copied(),
                    )
                },
                trace,
            })
        }
        SemanticExpression::Qualified {
            expression,
            qualifiers,
        } => {
            let mut parts = vec![(*expression.clone()).clone()];
            parts.extend(qualifiers.values().cloned());
            generate_with_style_unchecked(
                &GenerationRequest {
                    expression: SemanticExpression::And(parts),
                    include_trace: request.include_trace,
                },
                language,
                style,
            )
        }
        SemanticExpression::Exists { variable, body, .. } => {
            let nested = generate_with_style_unchecked(
                &GenerationRequest {
                    expression: (**body).clone(),
                    include_trace: request.include_trace,
                },
                language,
                style,
            )?;
            Ok(GeneratedText {
                text: format!("{}{}: {}", style.existential_prefix, variable, nested.text),
                trace: nested.trace,
            })
        }
        SemanticExpression::ForAll { variable, body, .. } => {
            let nested = generate_with_style_unchecked(
                &GenerationRequest {
                    expression: (**body).clone(),
                    include_trace: request.include_trace,
                },
                language,
                style,
            )?;
            Ok(GeneratedText {
                text: format!("{}{}: {}", style.universal_prefix, variable, nested.text),
                trace: nested.trace,
            })
        }
        _ => generate_basic(request, language),
    }
}

fn combine(
    expressions: &[SemanticExpression],
    separator: &str,
    include_trace: bool,
    language: &LanguageModel,
    style: &GenerationStyle,
) -> Result<GeneratedText, GenerationError> {
    if expressions.is_empty() || separator.trim().is_empty() {
        return Err(GenerationError::UnsupportedExpression);
    }
    let mut texts = Vec::with_capacity(expressions.len());
    let mut trace = None;
    for expression in expressions {
        let generated = generate_with_style_unchecked(
            &GenerationRequest {
                expression: expression.clone(),
                include_trace,
            },
            language,
            style,
        )?;
        trace = merge_traces(trace, generated.trace);
        texts.push(generated.text);
    }
    Ok(GeneratedText {
        text: texts.join(separator),
        trace,
    })
}

fn realize_value(value: &SemanticValue) -> String {
    match value {
        SemanticValue::Integer(value) => value.to_string(),
        SemanticValue::Decimal(value) => value.canonical.clone(),
        SemanticValue::Text(value) => value.clone(),
        SemanticValue::Boolean(value) => value.to_string(),
        SemanticValue::Date(value) => value.iso8601.clone(),
        SemanticValue::Quantity(value) => format!("{} {}", value.amount.canonical, value.unit),
    }
}

fn format_anchor(anchor: &SemanticAnchor) -> String {
    match anchor {
        SemanticAnchor::Concept(id) => format!("concept:{id}"),
        SemanticAnchor::Entity(id) => format!("entity:{id}"),
    }
}

pub fn generate_entity(
    entity: &EntityId,
    language: &LanguageModel,
) -> Result<GeneratedText, GenerationError> {
    generate(
        &GenerationRequest {
            expression: SemanticExpression::Entity(entity.clone()),
            include_trace: false,
        },
        language,
    )
}

pub fn generate_entity_with_features(
    entity: &EntityId,
    required_features: &FeatureStructure,
    include_trace: bool,
    language: &LanguageModel,
) -> Result<GeneratedText, GenerationError> {
    generate_anchor_with_features(
        &SemanticAnchor::Entity(entity.clone()),
        required_features,
        include_trace,
        language,
    )
}

pub fn realize_semantic(
    request: &SemanticRealizationRequest,
    target_language: &LanguageModel,
) -> Result<GeneratedText, GenerationError> {
    validate_realization_request(request)?;
    let result = generate(
        &GenerationRequest {
            expression: request.expression.clone(),
            include_trace: request.include_trace,
        },
        target_language,
    )?;
    Ok(apply_question_realization(request, target_language, result))
}

pub fn realize_semantic_with_style(
    request: &SemanticRealizationRequest,
    target_language: &LanguageModel,
    style: &GenerationStyle,
) -> Result<GeneratedText, GenerationError> {
    validate_realization_request(request)?;
    let result = generate_with_budget(
        &GenerationRequest {
            expression: request.expression.clone(),
            include_trace: request.include_trace,
        },
        target_language,
        style,
        GenerationBudget::default(),
    )?;
    Ok(apply_question_realization(request, target_language, result))
}

pub fn realize_semantic_with_budget(
    request: &SemanticRealizationRequest,
    target_language: &LanguageModel,
    style: &GenerationStyle,
    budget: GenerationBudget,
) -> Result<GeneratedText, GenerationError> {
    validate_realization_request(request)?;
    let result = generate_with_budget(
        &GenerationRequest {
            expression: request.expression.clone(),
            include_trace: request.include_trace,
        },
        target_language,
        style,
        budget,
    )?;
    let result = apply_question_realization(request, target_language, result);
    if result.text.len() > budget.max_output_bytes {
        return Err(GenerationError::BudgetExceeded("output bytes"));
    }
    Ok(result)
}

fn validate_realization_request(
    request: &SemanticRealizationRequest,
) -> Result<(), GenerationError> {
    if request.kind == SemanticRealizationKind::Goal && request.projection.is_empty() {
        return Err(GenerationError::UnsupportedExpression);
    }
    let mut seen = std::collections::BTreeSet::new();
    for variable in &request.projection {
        if !seen.insert(variable) {
            return Err(GenerationError::DuplicateProjection(variable.clone()));
        }
    }
    Ok(())
}

fn generate_predicate(
    expression: &SemanticExpression,
    include_trace: bool,
    language: &LanguageModel,
    style: &GenerationStyle,
) -> Result<GeneratedText, GenerationError> {
    generate_predicate_internal(
        expression,
        include_trace,
        language,
        style,
        None,
        &language.realizations.predicate_features,
        None,
    )
}

fn generate_question_predicate(
    expression: &SemanticExpression,
    query_variable: &VariableId,
    include_trace: bool,
    language: &LanguageModel,
    style: &GenerationStyle,
) -> Result<GeneratedText, GenerationError> {
    generate_predicate_internal(
        expression,
        include_trace,
        language,
        style,
        Some(query_variable),
        &FeatureStructure::default(),
        Some(0),
    )
}

fn generate_postverbal_question_predicate(
    expression: &SemanticExpression,
    query_variable: &VariableId,
    include_trace: bool,
    language: &LanguageModel,
    style: &GenerationStyle,
) -> Result<GeneratedText, GenerationError> {
    generate_predicate_internal(
        expression,
        include_trace,
        language,
        style,
        Some(query_variable),
        &FeatureStructure::default(),
        None,
    )
}

fn generate_predicate_internal(
    expression: &SemanticExpression,
    include_trace: bool,
    language: &LanguageModel,
    style: &GenerationStyle,
    omitted_variable: Option<&VariableId>,
    predicate_features: &FeatureStructure,
    head_position_override: Option<usize>,
) -> Result<GeneratedText, GenerationError> {
    let SemanticExpression::Apply { concept, bindings } = expression else {
        return generate_with_style_unchecked(
            &GenerationRequest {
                expression: expression.clone(),
                include_trace,
            },
            language,
            style,
        );
    };
    valency::validate_apply(concept, bindings, language)?;
    let head = generate_anchor_with_features(
        &SemanticAnchor::Concept(concept.clone()),
        predicate_features,
        include_trace,
        language,
    )?;
    let mut trace = head.trace;
    let mut arguments = Vec::with_capacity(bindings.len());
    for binding in valency::ordered_bindings(concept, bindings, language) {
        if omitted_variable.is_some_and(|variable| {
            matches!(
                binding.expression,
                SemanticExpression::Variable(candidate) if candidate == variable
            )
        }) {
            continue;
        }
        let generated = generate_argument(
            binding.expression,
            binding.features,
            include_trace,
            language,
            style,
        )?;
        let separator = binding
            .relation
            .and_then(|relation| language.realizations.surface_relations.get(relation))
            .map(String::as_str)
            .unwrap_or(" ");
        arguments.push(format!("{}{}", separator, generated.text));
        trace = merge_traces(trace, generated.trace);
    }
    Ok(GeneratedText {
        text: realize_apply_text(
            &head.text,
            arguments,
            head_position_override.or_else(|| {
                language
                    .realizations
                    .head_positions
                    .get(concept.as_str())
                    .copied()
            }),
        ),
        trace,
    })
}

fn realize_apply_text(head: &str, arguments: Vec<String>, head_position: Option<usize>) -> String {
    let Some(position) = head_position else {
        return format!("{}{}", head, arguments.join(""));
    };
    if position == 0 || arguments.is_empty() {
        return format!("{}{}", head, arguments.join(""));
    }
    let position = position.min(arguments.len());
    let before = arguments[..position]
        .iter()
        .map(|argument| argument.trim_start())
        .collect::<Vec<_>>()
        .join(" ");
    let after = arguments[position..].join("");
    format!("{} {}{}", before, head, after)
}

fn generate_argument(
    expression: &SemanticExpression,
    features: Option<&FeatureStructure>,
    include_trace: bool,
    language: &LanguageModel,
    style: &GenerationStyle,
) -> Result<GeneratedText, GenerationError> {
    let required = features.cloned().unwrap_or_default();
    let anchor = match expression {
        SemanticExpression::Entity(id) => Some(SemanticAnchor::Entity(id.clone())),
        SemanticExpression::Concept(id) => Some(SemanticAnchor::Concept(id.clone())),
        _ => None,
    };
    match anchor {
        Some(anchor) => generate_anchor_with_features(&anchor, &required, include_trace, language),
        None => generate_with_style_unchecked(
            &GenerationRequest {
                expression: expression.clone(),
                include_trace,
            },
            language,
            style,
        ),
    }
}

fn apply_question_realization(
    request: &SemanticRealizationRequest,
    language: &LanguageModel,
    mut result: GeneratedText,
) -> GeneratedText {
    if request.kind != SemanticRealizationKind::Goal {
        return result;
    }

    if let SemanticExpression::Equals { left, right } = &request.expression {
        if matches!(left.as_ref(), SemanticExpression::Variable(_)) {
            if let Ok(value) = generate_basic(
                &GenerationRequest {
                    expression: right.as_ref().clone(),
                    include_trace: request.include_trace,
                },
                language,
            ) {
                result.text = format!("{}{}?", language.realizations.question_prefix, value.text);
                result.trace = value.trace;
                return result;
            }
        }
    }

    if let SemanticExpression::Satisfies { subject, predicate } = &request.expression {
        if let Some(query) = request.projection.first() {
            if matches!(subject.as_ref(), SemanticExpression::Variable(variable) if variable == query)
            {
                if let SemanticExpression::Apply { .. } = predicate.as_ref() {
                    let prefix = if predicate_is_sentence(predicate, language) {
                        &language.realizations.question_subject_prefix
                    } else {
                        &language.realizations.question_prefix
                    };
                    if let Ok(value) = generate_question_predicate(
                        predicate,
                        query,
                        request.include_trace,
                        language,
                        &GenerationStyle::from_language(language),
                    ) {
                        result.text = format!(
                            "{}{}{}?",
                            prefix, language.realizations.predicate_prefix, value.text
                        );
                        result.trace = value.trace;
                        return result;
                    }
                }
            }
            if let SemanticExpression::Apply { bindings, .. } = predicate.as_ref() {
                let queried_binding = bindings.values().any(|expression| {
                    matches!(expression, SemanticExpression::Variable(variable) if variable == query)
                });
                if queried_binding {
                    if let Ok(value) = generate_question_predicate(
                        predicate,
                        query,
                        request.include_trace,
                        language,
                        &GenerationStyle::from_language(language),
                    ) {
                        result.text = format!(
                            "{}{}?",
                            language.realizations.question_object_prefix, value.text
                        );
                        result.trace = value.trace;
                        return result;
                    }
                }
            }
        }
    }

    if let SemanticExpression::Apply { bindings, .. } = &request.expression {
        if let Some(query) = request.projection.first() {
            if let Some(parameter) = bindings.iter().find_map(|(parameter, expression)| {
                matches!(expression, SemanticExpression::Variable(variable) if variable == query)
                    .then_some(parameter)
            }) {
                if let Some(suffix) = language
                    .realizations
                    .question_argument_suffixes
                    .get(parameter.as_str())
                {
                    if let Ok(value) = generate_postverbal_question_predicate(
                        &request.expression,
                        query,
                        request.include_trace,
                        language,
                        &GenerationStyle::from_language(language),
                    ) {
                        result.text = format!("{} {}?", value.text, suffix);
                        result.trace = value.trace;
                        return result;
                    }
                }
                if let Ok(value) = generate_question_predicate(
                    &request.expression,
                    query,
                    request.include_trace,
                    language,
                    &GenerationStyle::from_language(language),
                ) {
                    let prefix = language
                        .realizations
                        .question_argument_prefixes
                        .get(parameter.as_str())
                        .unwrap_or(&language.realizations.question_subject_prefix);
                    result.text = format!("{}{}?", prefix, value.text);
                    result.trace = value.trace;
                    return result;
                }
            }
        }
    }

    result.text = format!("{}{}?", language.realizations.question_prefix, result.text);
    result
}

fn predicate_is_sentence(expression: &SemanticExpression, language: &LanguageModel) -> bool {
    let SemanticExpression::Apply { concept, .. } = expression else {
        return false;
    };
    language
        .compiled_senses
        .values()
        .filter(|sense| sense.anchor == Some(SemanticAnchor::Concept(concept.clone())))
        .min_by_key(|sense| (sense.priority, sense.id.clone()))
        .is_some_and(|sense| sense.category.is_sentence())
}

pub fn generate_concept(
    concept: &ConceptId,
    language: &LanguageModel,
) -> Result<GeneratedText, GenerationError> {
    generate(
        &GenerationRequest {
            expression: SemanticExpression::Concept(concept.clone()),
            include_trace: false,
        },
        language,
    )
}
