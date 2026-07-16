#[path = "support/mod.rs"]
mod support;

use lexflex_lingua::{
    compiler::CompileError, LinguaDeclaration, LinguaExpression, LinguaProgram, ProgramId,
    TypeError,
};
use lexflex_model::{ConceptId, EntityId, ParameterId, SemanticValue};
use std::collections::BTreeMap;

#[test]
fn missing_required_scope_is_rejected() {
    let program = LinguaProgram {
        id: ProgramId::new("test:missing-scope"),
        declarations: vec![LinguaDeclaration::Concept(
            support::model::capital_declaration(),
        )],
        entry: LinguaExpression::ApplyConcept {
            concept: ConceptId::new_unchecked("CAPITAL"),
            bindings: BTreeMap::new(),
        },
    };

    let error = support::test_compiler()
        .compile(&program)
        .expect_err("missing binding");
    assert!(matches!(
        error,
        CompileError::Type(TypeError::MissingParameter { .. })
    ));
}

#[test]
fn unknown_parameter_is_rejected() {
    let program = LinguaProgram {
        id: ProgramId::new("test:unknown-param"),
        declarations: vec![LinguaDeclaration::Concept(
            support::model::capital_declaration(),
        )],
        entry: LinguaExpression::ApplyConcept {
            concept: ConceptId::new_unchecked("CAPITAL"),
            bindings: BTreeMap::from([
                (
                    ParameterId::new_unchecked("scope"),
                    LinguaExpression::Entity(EntityId::new_unchecked("FRANCE")),
                ),
                (
                    ParameterId::new_unchecked("bogus"),
                    LinguaExpression::Entity(EntityId::new_unchecked("FRANCE")),
                ),
            ]),
        },
    };

    let error = support::test_compiler()
        .compile(&program)
        .expect_err("unknown parameter");
    assert!(matches!(
        error,
        CompileError::Type(TypeError::UnknownParameter { .. })
    ));
}

#[test]
fn wrong_scope_type_is_rejected() {
    let program = LinguaProgram {
        id: ProgramId::new("test:wrong-scope"),
        declarations: vec![LinguaDeclaration::Concept(
            support::model::capital_declaration(),
        )],
        entry: LinguaExpression::ApplyConcept {
            concept: ConceptId::new_unchecked("CAPITAL"),
            bindings: BTreeMap::from([(
                ParameterId::new_unchecked("scope"),
                LinguaExpression::Value(SemanticValue::from(42_i64)),
            )]),
        },
    };

    let error = support::test_compiler()
        .compile(&program)
        .expect_err("wrong type");
    assert!(matches!(
        error,
        CompileError::Type(TypeError::ParameterTypeMismatch { .. })
    ));
}

#[test]
fn satisfies_requires_boolean_predicate_subject() {
    let program = LinguaProgram {
        id: ProgramId::new("test:bad-satisfies"),
        declarations: vec![LinguaDeclaration::Concept(
            support::model::capital_declaration(),
        )],
        entry: LinguaExpression::Satisfies {
            subject: Box::new(LinguaExpression::Value(SemanticValue::from(1_i64))),
            concept: Box::new(LinguaExpression::ApplyConcept {
                concept: ConceptId::new_unchecked("CAPITAL"),
                bindings: BTreeMap::from([(
                    ParameterId::new_unchecked("scope"),
                    LinguaExpression::Entity(EntityId::new_unchecked("FRANCE")),
                )]),
            }),
        },
    };

    let error = support::test_compiler()
        .compile(&program)
        .expect_err("bad subject");
    assert!(matches!(
        error,
        CompileError::Type(TypeError::ExpectedPredicate { .. })
    ));
}

#[test]
fn equals_rejects_incompatible_operands() {
    let program = LinguaProgram {
        id: ProgramId::new("test:bad-equals"),
        declarations: Vec::new(),
        entry: LinguaExpression::Equals {
            left: Box::new(LinguaExpression::Entity(EntityId::new_unchecked("PARIS"))),
            right: Box::new(LinguaExpression::Value(SemanticValue::from(1_i64))),
        },
    };

    let error = support::test_compiler()
        .compile(&program)
        .expect_err("bad equals");
    assert!(matches!(
        error,
        CompileError::Type(TypeError::EqualityTypeMismatch { .. })
    ));
}
