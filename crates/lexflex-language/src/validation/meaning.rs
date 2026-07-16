use crate::validation::LanguageValidationIssue;
use crate::MeaningTemplate;
use lexflex_lingua::LinguaExpression;
use lexflex_model::{ConceptCatalog, VariableId};
use std::collections::BTreeSet;

pub(super) fn validate_meaning_template(
    meaning: &MeaningTemplate,
    catalog: &ConceptCatalog,
    sense_id: &str,
) -> Result<(), LanguageValidationIssue> {
    let mut query_variables = BTreeSet::new();
    collect_query_variables(&meaning.expression, &mut query_variables);
    let declared: BTreeSet<_> = meaning.query_variables.keys().cloned().collect();
    if query_variables != declared {
        return Err(LanguageValidationIssue::SenseMeaning {
            sense_id: sense_id.to_owned(),
            message: "query variable set mismatch".to_owned(),
        });
    }
    validate_meaning_expression(&meaning.expression, catalog, &declared, sense_id)
}

fn collect_query_variables(expression: &LinguaExpression, output: &mut BTreeSet<VariableId>) {
    match expression {
        LinguaExpression::QueryVariable(variable) => {
            output.insert(variable.clone());
        }
        LinguaExpression::Lambda { body, .. }
        | LinguaExpression::Not(body)
        | LinguaExpression::Exists { body, .. }
        | LinguaExpression::ForAll { body, .. }
        | LinguaExpression::Let { body, .. } => collect_query_variables(body, output),
        LinguaExpression::Call { callee, arguments } => {
            collect_query_variables(callee, output);
            for value in arguments.values() {
                collect_query_variables(value, output);
            }
        }
        LinguaExpression::ApplyConcept { bindings, .. } => {
            for value in bindings.values() {
                collect_query_variables(value, output);
            }
        }
        LinguaExpression::Satisfies { subject, concept }
        | LinguaExpression::Equals {
            left: subject,
            right: concept,
        } => {
            collect_query_variables(subject, output);
            collect_query_variables(concept, output);
        }
        LinguaExpression::And(items) | LinguaExpression::Or(items) => {
            for item in items {
                collect_query_variables(item, output);
            }
        }
        LinguaExpression::Concept(_)
        | LinguaExpression::Entity(_)
        | LinguaExpression::Value(_)
        | LinguaExpression::Variable(_)
        | LinguaExpression::Function(_) => {}
    }
}

fn validate_meaning_expression(
    expression: &LinguaExpression,
    catalog: &ConceptCatalog,
    allowed_query_variables: &BTreeSet<VariableId>,
    sense_id: &str,
) -> Result<(), LanguageValidationIssue> {
    match expression {
        LinguaExpression::Concept(concept) => {
            crate::validation::ensure_known_concept(concept, catalog, sense_id)
        }
        LinguaExpression::Entity(entity) => {
            if catalog.entity(entity).is_some() {
                Ok(())
            } else {
                Err(LanguageValidationIssue::UnknownConceptReference {
                    sense_id: sense_id.to_owned(),
                    concept_id: entity.to_string(),
                })
            }
        }
        LinguaExpression::Lambda { body, .. }
        | LinguaExpression::Not(body)
        | LinguaExpression::Exists { body, .. }
        | LinguaExpression::ForAll { body, .. }
        | LinguaExpression::Let { body, .. } => {
            validate_meaning_expression(body, catalog, allowed_query_variables, sense_id)
        }
        LinguaExpression::Call { callee, arguments } => {
            validate_meaning_expression(callee, catalog, allowed_query_variables, sense_id)?;
            for value in arguments.values() {
                validate_meaning_expression(value, catalog, allowed_query_variables, sense_id)?;
            }
            Ok(())
        }
        LinguaExpression::ApplyConcept { concept, bindings } => {
            crate::validation::ensure_known_concept(concept, catalog, sense_id)?;
            for value in bindings.values() {
                validate_meaning_expression(value, catalog, allowed_query_variables, sense_id)?;
            }
            Ok(())
        }
        LinguaExpression::Satisfies { subject, concept }
        | LinguaExpression::Equals {
            left: subject,
            right: concept,
        } => {
            validate_meaning_expression(subject, catalog, allowed_query_variables, sense_id)?;
            validate_meaning_expression(concept, catalog, allowed_query_variables, sense_id)
        }
        LinguaExpression::And(items) | LinguaExpression::Or(items) => {
            for item in items {
                validate_meaning_expression(item, catalog, allowed_query_variables, sense_id)?;
            }
            Ok(())
        }
        LinguaExpression::QueryVariable(variable) => {
            if allowed_query_variables.contains(variable) {
                Ok(())
            } else {
                Err(LanguageValidationIssue::QueryVariableNotAllowed {
                    sense_id: sense_id.to_owned(),
                    variable: variable.to_string(),
                })
            }
        }
        LinguaExpression::Value(_)
        | LinguaExpression::Variable(_)
        | LinguaExpression::Function(_) => Ok(()),
    }
}
