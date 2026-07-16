mod error;
mod meaning;

pub use error::{LanguageValidationError, LanguageValidationIssue};

use crate::loader::LanguageModel;
use crate::SemanticAnchor;
use lexflex_model::{ConceptCatalog, ConceptId, SemanticType, TypeRelation, ValueType};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Default)]
pub struct LanguageModelValidator;

impl LanguageModelValidator {
    pub fn validate(
        &self,
        model: &LanguageModel,
        catalog: &ConceptCatalog,
    ) -> Result<(), LanguageValidationError> {
        let mut lexemes = BTreeSet::new();
        for lexeme in model.lexemes.values() {
            if !lexemes.insert(lexeme.id.as_str().to_owned()) {
                return Err(LanguageValidationIssue::DuplicateLexemeId {
                    lexeme_id: lexeme.id.to_string(),
                }
                .into());
            }
            if lexeme.language != model.manifest.language {
                return Err(LanguageValidationIssue::LexemeLanguageMismatch {
                    lexeme_id: lexeme.id.to_string(),
                    lexeme_language: lexeme.language.to_string(),
                    package_language: model.manifest.language.to_string(),
                }
                .into());
            }
            let expected_normalized = crate::Form::normalize_surface(&lexeme.lemma);
            if !lexeme.normalized_lemma.is_empty()
                && lexeme.normalized_lemma != expected_normalized
            {
                return Err(
                    LanguageValidationIssue::LexemeNormalizedLemmaMismatch {
                        lexeme_id: lexeme.id.to_string(),
                        expected: expected_normalized,
                        found: lexeme.normalized_lemma.clone(),
                    }
                    .into(),
                );
            }
        }

        let mut senses = BTreeSet::new();
        for sense in model.senses.values() {
            if !senses.insert(sense.id.as_str().to_owned()) {
                return Err(LanguageValidationIssue::DuplicateSenseId {
                    sense_id: sense.id.to_string(),
                }
                .into());
            }
            if !lexemes.contains(sense.lexeme_id.as_str()) {
                return Err(LanguageValidationIssue::UnknownLexemeIdInSense {
                    sense_id: sense.id.to_string(),
                    lexeme_id: sense.lexeme_id.to_string(),
                }
                .into());
            }
            validate_anchor(&sense.anchor, catalog, sense.id.as_str())?;
            meaning::validate_meaning_template(&sense.meaning, catalog, sense.id.as_str())?;
            validate_category(&sense.base_category, catalog, sense.id.as_str())?;
            validate_valency(
                &sense.valency,
                sense.anchor.as_ref(),
                catalog,
                sense.id.as_str(),
            )?;
        }

        let mut forms = BTreeSet::new();
        let mut forms_by_id: BTreeMap<_, _> = BTreeMap::new();
        let mut forms_by_lexeme: BTreeMap<_, Vec<_>> = BTreeMap::new();
        for form in model.forms.values() {
            if !forms.insert(form.id.as_str().to_owned()) {
                return Err(LanguageValidationIssue::DuplicateFormId {
                    form_id: form.id.to_string(),
                }
                .into());
            }
            validate_form(form, &lexemes)?;
            forms_by_id.insert(form.id.clone(), form);
            forms_by_lexeme
                .entry(form.lexeme_id.clone())
                .or_default()
                .push(form);
        }

        for sense in model.senses.values() {
            if let Some(forms) = forms_by_lexeme.get(&sense.lexeme_id) {
                for form in forms {
                    sense.features.unify(&form.features).map_err(|conflict| {
                        LanguageValidationIssue::SenseCategory {
                            sense_id: sense.id.to_string(),
                            message: format!(
                                "feature conflict with form {}: {}",
                                form.id, conflict.name
                            ),
                        }
                    })?;
                }
            }
        }

        let mut paradigms = BTreeSet::new();
        for paradigm in model.paradigms.values() {
            if !paradigms.insert(paradigm.id.as_str().to_owned()) {
                return Err(LanguageValidationIssue::DuplicateParadigmId {
                    paradigm_id: paradigm.id.to_string(),
                }
                .into());
            }
            if paradigm.language != model.manifest.language {
                return Err(LanguageValidationIssue::ParadigmLanguageMismatch {
                    paradigm_id: paradigm.id.to_string(),
                    paradigm_language: paradigm.language.to_string(),
                    package_language: model.manifest.language.to_string(),
                }
                .into());
            }
            for form in &paradigm.forms {
                validate_form(form, &lexemes).map_err(|issue| {
                    LanguageValidationIssue::SenseValency {
                        sense_id: paradigm.id.to_string(),
                        message: issue.to_string(),
                    }
                })?;
                let canonical = forms_by_id.get(&form.id).ok_or_else(|| {
                    LanguageValidationIssue::ParadigmUnknownForm {
                        paradigm_id: paradigm.id.to_string(),
                        form_id: form.id.to_string(),
                    }
                })?;
                if *canonical != form {
                    return Err(LanguageValidationIssue::ParadigmCanonicalMismatch {
                        paradigm_id: paradigm.id.to_string(),
                        form_id: form.id.to_string(),
                    }
                    .into());
                }
            }
        }

        Ok(())
    }
}

fn validate_form(
    form: &crate::Form,
    lexemes: &BTreeSet<String>,
) -> Result<(), LanguageValidationIssue> {
    if !lexemes.contains(form.lexeme_id.as_str()) {
        return Err(LanguageValidationIssue::UnknownLexemeIdInForm {
            form_id: form.id.to_string(),
            lexeme_id: form.lexeme_id.to_string(),
        });
    }
    let expected_normalized = crate::Form::normalize_surface(&form.surface);
    if form.normalized != expected_normalized {
        return Err(LanguageValidationIssue::FormNormalizedMismatch {
            form_id: form.id.to_string(),
            expected: expected_normalized,
            found: form.normalized.clone(),
        });
    }
    Ok(())
}

fn validate_semantic_type(
    semantic_type: &SemanticType,
    catalog: &ConceptCatalog,
    sense_id: &str,
) -> Result<(), LanguageValidationIssue> {
    match semantic_type {
        SemanticType::Boolean | SemanticType::Concept | SemanticType::Entity => Ok(()),
        SemanticType::ConceptOf(_) => Ok(()),
        SemanticType::EntityOf(concept) => ensure_known_concept(concept, catalog, sense_id),
        SemanticType::Value(value_type) => validate_value_type(value_type, catalog, sense_id),
        SemanticType::Predicate(inner)
        | SemanticType::Set(inner)
        | SemanticType::Optional(inner) => validate_semantic_type(inner, catalog, sense_id),
        SemanticType::Record(entries) => {
            for value in entries.values() {
                validate_semantic_type(value, catalog, sense_id)?;
            }
            Ok(())
        }
        SemanticType::Function(function) => {
            for value in function.parameters.values() {
                validate_semantic_type(value, catalog, sense_id)?;
            }
            validate_semantic_type(&function.result, catalog, sense_id)
        }
    }
}

fn validate_value_type(
    value_type: &ValueType,
    catalog: &ConceptCatalog,
    sense_id: &str,
) -> Result<(), LanguageValidationIssue> {
    match value_type {
        ValueType::Quantity { dimension } => ensure_known_concept(dimension, catalog, sense_id),
        _ => Ok(()),
    }
}

fn validate_anchor(
    anchor: &Option<SemanticAnchor>,
    catalog: &ConceptCatalog,
    sense_id: &str,
) -> Result<(), LanguageValidationIssue> {
    match anchor {
        None => Ok(()),
        Some(SemanticAnchor::Concept(concept)) => ensure_known_concept(concept, catalog, sense_id),
        Some(SemanticAnchor::Entity(entity)) => {
            if catalog.entity(entity).is_some() {
                Ok(())
            } else {
                Err(LanguageValidationIssue::UnknownEntityAnchor {
                    sense_id: sense_id.to_owned(),
                    entity_id: entity.to_string(),
                })
            }
        }
    }
}

fn validate_valency(
    valency: &[crate::ValencySlot],
    anchor: Option<&SemanticAnchor>,
    catalog: &ConceptCatalog,
    sense_id: &str,
) -> Result<(), LanguageValidationIssue> {
    if valency.is_empty() {
        return Ok(());
    }

    let concept = match anchor {
        Some(SemanticAnchor::Concept(concept)) => concept,
        Some(SemanticAnchor::Entity(entity)) => {
            return Err(LanguageValidationIssue::ValencyRequiresConceptAnchor {
                sense_id: sense_id.to_owned(),
                entity_id: entity.to_string(),
            });
        }
        None => {
            return Err(LanguageValidationIssue::UnknownAnchorConcept {
                sense_id: sense_id.to_owned(),
                concept_id: "missing concept anchor".to_owned(),
            });
        }
    };

    let schema =
        catalog
            .concept(concept)
            .ok_or_else(|| LanguageValidationIssue::UnknownAnchorConcept {
                sense_id: sense_id.to_owned(),
                concept_id: concept.to_string(),
            })?;

    let mut ranks = BTreeSet::new();
    for slot in valency {
        if !ranks.insert(slot.application_rank) {
            return Err(LanguageValidationIssue::DuplicateValencyApplicationRank {
                sense_id: sense_id.to_owned(),
                rank: slot.application_rank,
            });
        }

        let parameter = schema.parameters.get(&slot.parameter).ok_or_else(|| {
            LanguageValidationIssue::UnknownValencyParameter {
                sense_id: sense_id.to_owned(),
                concept_id: concept.to_string(),
                parameter_id: slot.parameter.to_string(),
            }
        })?;

        let actual = semantic_type_of_category(&slot.argument_category).ok_or_else(|| {
            LanguageValidationIssue::SenseValency {
                sense_id: sense_id.to_owned(),
                message: format!("invalid argument category for slot {}", slot.id),
            }
        })?;

        if !TypeRelation::new(catalog).accepts(&parameter.value_type, &actual) {
            return Err(LanguageValidationIssue::ValencyTypeMismatch {
                sense_id: sense_id.to_owned(),
                parameter_id: slot.parameter.to_string(),
                expected: format!("{:?}", parameter.value_type),
                actual: format!("{:?}", actual),
            });
        }
    }

    for (parameter_id, parameter) in &schema.parameters {
        if parameter.required && !valency.iter().any(|slot| &slot.parameter == parameter_id) {
            return Err(LanguageValidationIssue::MissingRequiredValency {
                sense_id: sense_id.to_owned(),
                concept_id: concept.to_string(),
                parameter_id: parameter_id.to_string(),
            });
        }
    }

    Ok(())
}

fn validate_category(
    category: &crate::SyntacticCategory,
    catalog: &ConceptCatalog,
    sense_id: &str,
) -> Result<(), LanguageValidationIssue> {
    match category {
        crate::SyntacticCategory::Atom { semantic_type, .. } => {
            validate_category_type(semantic_type, catalog, sense_id)?;
            Ok(())
        }
        crate::SyntacticCategory::Function {
            result,
            argument,
            ..
        } => {
            validate_category(result, catalog, sense_id)?;
            validate_category(argument, catalog, sense_id)
        }
    }
}

fn validate_category_type(
    category_type: &crate::CategoryType,
    catalog: &ConceptCatalog,
    sense_id: &str,
) -> Result<(), LanguageValidationIssue> {
    match category_type {
        crate::CategoryType::Concrete(semantic_type) => validate_semantic_type(semantic_type, catalog, sense_id),
        crate::CategoryType::Variable(_) => Ok(()),
    }
}

fn semantic_type_of_category(category: &crate::SyntacticCategory) -> Option<SemanticType> {
    match category {
        crate::SyntacticCategory::Atom {
            kind: crate::AtomicCategoryKind::Sentence,
            ..
        } => Some(SemanticType::Boolean),
        crate::SyntacticCategory::Atom {
            kind: crate::AtomicCategoryKind::NounPhrase,
            semantic_type,
            ..
        } => Some(extract_category_type(semantic_type)),
        crate::SyntacticCategory::Atom {
            kind: crate::AtomicCategoryKind::Predicate,
            semantic_type,
            ..
        } => Some(SemanticType::Predicate(Box::new(extract_category_type(
            semantic_type,
        )))),
        crate::SyntacticCategory::Atom {
            kind: crate::AtomicCategoryKind::MarkedArgument { .. },
            semantic_type,
            ..
        } => Some(extract_category_type(semantic_type)),
        crate::SyntacticCategory::Function { result, .. } => semantic_type_of_category(result),
    }
}

fn extract_category_type(category_type: &crate::CategoryType) -> SemanticType {
    match category_type {
        crate::CategoryType::Concrete(semantic_type) => semantic_type.clone(),
        crate::CategoryType::Variable(_) => SemanticType::Entity,
    }
}

fn ensure_known_concept(
    concept: &ConceptId,
    catalog: &ConceptCatalog,
    sense_id: &str,
) -> Result<(), LanguageValidationIssue> {
    if catalog.concepts.contains_key(concept) {
        Ok(())
    } else {
        Err(LanguageValidationIssue::UnknownConceptReference {
            sense_id: sense_id.to_owned(),
            concept_id: concept.to_string(),
        })
    }
}
