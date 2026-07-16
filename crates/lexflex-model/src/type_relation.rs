use crate::{ConceptCatalog, SemanticType};

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
            (SemanticType::Optional(expected), actual) => self.accepts(expected, actual),
            (SemanticType::Predicate(expected), SemanticType::Predicate(actual)) => {
                self.accepts(expected, actual)
            }
            (SemanticType::Set(expected), SemanticType::Set(actual)) => {
                self.accepts(expected, actual)
            }
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
}
