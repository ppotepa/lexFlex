use crate::compiler::{CompileTypeReferenceError, CompileTypeReferenceLocation};
use crate::syntax::LinguaExpression;
use lexflex_model::{validate_semantic_type_references, ConceptCatalog};

pub(crate) fn validate_expression_type_references(
    expression: &LinguaExpression,
    catalog: &ConceptCatalog,
) -> Result<(), CompileTypeReferenceError> {
    match expression {
        LinguaExpression::Concept(_)
        | LinguaExpression::Entity(_)
        | LinguaExpression::Value(_)
        | LinguaExpression::Variable(_)
        | LinguaExpression::QueryVariable(_)
        | LinguaExpression::Function(_) => Ok(()),

        LinguaExpression::Lambda { parameters, body } => {
            for parameter in parameters {
                validate_semantic_type_references(&parameter.value_type, catalog).map_err(
                    |source| CompileTypeReferenceError {
                        location: CompileTypeReferenceLocation::LambdaParameter {
                            parameter: parameter.parameter_id.clone(),
                        },
                        source,
                    },
                )?;
            }
            validate_expression_type_references(body, catalog)
        }

        LinguaExpression::Call { callee, arguments } => {
            validate_expression_type_references(callee, catalog)?;
            for argument in arguments.values() {
                validate_expression_type_references(argument, catalog)?;
            }
            Ok(())
        }

        LinguaExpression::ApplyConcept { bindings, .. } => {
            for binding in bindings.values() {
                validate_expression_type_references(binding, catalog)?;
            }
            Ok(())
        }

        LinguaExpression::Satisfies { subject, concept } => {
            validate_expression_type_references(subject, catalog)?;
            validate_expression_type_references(concept, catalog)
        }

        LinguaExpression::Equals { left, right } => {
            validate_expression_type_references(left, catalog)?;
            validate_expression_type_references(right, catalog)
        }

        LinguaExpression::And(expressions) | LinguaExpression::Or(expressions) => {
            for expression in expressions {
                validate_expression_type_references(expression, catalog)?;
            }
            Ok(())
        }

        LinguaExpression::Not(inner) => validate_expression_type_references(inner, catalog),

        LinguaExpression::Exists {
            variable,
            value_type,
            body,
        }
        | LinguaExpression::ForAll {
            variable,
            value_type,
            body,
        } => {
            validate_semantic_type_references(value_type, catalog).map_err(|source| {
                CompileTypeReferenceError {
                    location: CompileTypeReferenceLocation::Quantifier {
                        variable: variable.clone(),
                    },
                    source,
                }
            })?;
            validate_expression_type_references(body, catalog)
        }

        LinguaExpression::Let { value, body, .. } => {
            validate_expression_type_references(value, catalog)?;
            validate_expression_type_references(body, catalog)
        }
    }
}
