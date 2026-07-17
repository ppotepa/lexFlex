use crate::{ConceptCatalog, SemanticType};
use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub struct TypeRelation<'a> {
    catalog: &'a ConceptCatalog,
}

impl<'a> TypeRelation<'a> {
    pub fn new(catalog: &'a ConceptCatalog) -> Self {
        Self { catalog }
    }

    pub fn accepts(&self, expected: &SemanticType, actual: &SemanticType) -> bool {
        match (expected, actual) {
            (left, right) if left == right => true,
            (SemanticType::Entity, SemanticType::EntityOf(_)) => true,
            (SemanticType::EntityOf(parent), SemanticType::EntityOf(child)) => {
                self.catalog.is_subtype(child, parent)
            }
            (SemanticType::Optional(expected), SemanticType::Optional(actual)) => {
                self.accepts(expected, actual)
            }
            (SemanticType::Optional(expected), actual) => self.accepts(expected, actual),
            (SemanticType::Predicate(expected_arg), SemanticType::Predicate(actual_arg)) => {
                self.accepts(expected_arg, actual_arg)
            }
            (SemanticType::Set(expected), SemanticType::Set(actual)) => {
                self.accepts(expected, actual)
            }
            (SemanticType::Function(expected), SemanticType::Function(actual)) => {
                if expected.parameters.keys().ne(actual.parameters.keys()) {
                    return false;
                }
                for parameter in expected.parameters.keys() {
                    let expected_parameter = &expected.parameters[parameter];
                    let actual_parameter = &actual.parameters[parameter];
                    if !self.accepts(actual_parameter, expected_parameter) {
                        return false;
                    }
                }
                self.accepts(&expected.result, &actual.result)
            }
            (SemanticType::Record(expected), SemanticType::Record(actual)) => expected
                .iter()
                .all(|(field, expected_type)| {
                    actual
                        .get(field)
                        .is_some_and(|actual_type| self.accepts(expected_type, actual_type))
                }),
            (SemanticType::Value(expected), SemanticType::Value(actual)) => expected == actual,
            (SemanticType::Concept, SemanticType::ConceptOf(_)) => true,
            (SemanticType::ConceptOf(expected), SemanticType::ConceptOf(actual)) => {
                expected == actual
            }
            _ => false,
        }
    }

    pub fn equivalent(&self, left: &SemanticType, right: &SemanticType) -> bool {
        self.accepts(left, right) && self.accepts(right, left)
    }

    pub fn narrower(
        &self,
        left: &SemanticType,
        right: &SemanticType,
    ) -> Result<SemanticType, TypeRelationError> {
        let left_accepts_right = self.accepts(left, right);
        let right_accepts_left = self.accepts(right, left);

        match (left_accepts_right, right_accepts_left) {
            (true, false) => Ok(right.clone()),
            (false, true) => Ok(left.clone()),
            (true, true) => Ok(std::cmp::min(left.clone(), right.clone())),
            (false, false) => Err(TypeRelationError::Incompatible {
                left: left.clone(),
                right: right.clone(),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TypeRelationError {
    #[error("incompatible semantic types: left={left:?}, right={right:?}")]
    Incompatible {
        left: SemanticType,
        right: SemanticType,
    },
}
