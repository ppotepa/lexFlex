mod support;

use lexflex_lingua::compiler::{CompileError, CompileTypeReferenceLocation};
use lexflex_lingua::{
    FunctionDeclaration, FunctionId, LambdaParameter, LinguaDeclaration, LinguaExpression,
    LinguaProgram, ProgramId, SemanticType, SymbolName,
};
use lexflex_model::{
    ConceptId, ParameterId, SemanticTypeReferenceError, SemanticValue, VariableId,
};
use support::compiler::test_compiler;

fn missing_type() -> SemanticType {
    SemanticType::EntityOf(ConceptId::new_unchecked("MISSING"))
}

fn program(entry: LinguaExpression) -> LinguaProgram {
    LinguaProgram {
        id: ProgramId::new_unchecked("program:type-reference"),
        declarations: Vec::new(),
        entry,
    }
}

fn assert_unknown_concept(error: CompileError) {
    let CompileError::TypeReference(error) = error else {
        panic!("expected typed reference error, found {error:?}");
    };
    assert_eq!(
        error.source,
        SemanticTypeReferenceError::UnknownConcept(ConceptId::new_unchecked("MISSING"))
    );
}

#[test]
fn entry_lambda_parameter_reference_is_validated() {
    let error = test_compiler()
        .compile(&program(LinguaExpression::Lambda {
            parameters: vec![LambdaParameter {
                name: SymbolName::new_unchecked("x"),
                parameter_id: ParameterId::new_unchecked("x"),
                value_type: missing_type(),
            }],
            body: Box::new(LinguaExpression::Value(SemanticValue::Boolean(true))),
        }))
        .expect_err("missing lambda type reference");

    let CompileError::TypeReference(error) = error else {
        panic!("expected typed reference error, found {error:?}");
    };
    assert_eq!(
        error.location,
        CompileTypeReferenceLocation::LambdaParameter {
            parameter: ParameterId::new_unchecked("x")
        }
    );
    assert_eq!(
        error.source,
        SemanticTypeReferenceError::UnknownConcept(ConceptId::new_unchecked("MISSING"))
    );
}

#[test]
fn nested_lambda_parameter_reference_is_validated() {
    let error = test_compiler()
        .compile(&program(LinguaExpression::Not(Box::new(
            LinguaExpression::Lambda {
                parameters: vec![LambdaParameter {
                    name: SymbolName::new_unchecked("x"),
                    parameter_id: ParameterId::new_unchecked("x"),
                    value_type: missing_type(),
                }],
                body: Box::new(LinguaExpression::Value(SemanticValue::Boolean(true))),
            },
        ))))
        .expect_err("missing nested lambda type reference");

    assert_unknown_concept(error);
}

#[test]
fn exists_type_reference_is_validated() {
    let variable = VariableId::new_unchecked("x");
    let error = test_compiler()
        .compile(&program(LinguaExpression::Exists {
            variable: variable.clone(),
            value_type: missing_type(),
            body: Box::new(LinguaExpression::Value(SemanticValue::Boolean(true))),
        }))
        .expect_err("missing exists type reference");

    let CompileError::TypeReference(error) = error else {
        panic!("expected typed reference error, found {error:?}");
    };
    assert_eq!(
        error.location,
        CompileTypeReferenceLocation::Quantifier { variable }
    );
    assert_eq!(
        error.source,
        SemanticTypeReferenceError::UnknownConcept(ConceptId::new_unchecked("MISSING"))
    );
}

#[test]
fn forall_type_reference_is_validated() {
    let error = test_compiler()
        .compile(&program(LinguaExpression::ForAll {
            variable: VariableId::new_unchecked("x"),
            value_type: missing_type(),
            body: Box::new(LinguaExpression::Value(SemanticValue::Boolean(true))),
        }))
        .expect_err("missing forall type reference");

    assert_unknown_concept(error);
}

#[test]
fn function_body_annotation_reference_is_validated() {
    let error = test_compiler()
        .compile(&LinguaProgram {
            id: ProgramId::new_unchecked("program:function-body-type"),
            declarations: vec![LinguaDeclaration::Function(FunctionDeclaration {
                declaration_id: lexflex_lingua::DeclarationId::new_unchecked("declaration:f"),
                function_id: FunctionId::new_unchecked("f"),
                name: SymbolName::new_unchecked("f"),
                parameters: Vec::new(),
                result_type: SemanticType::Boolean,
                body: LinguaExpression::Exists {
                    variable: VariableId::new_unchecked("x"),
                    value_type: missing_type(),
                    body: Box::new(LinguaExpression::Value(SemanticValue::Boolean(true))),
                },
            })],
            entry: LinguaExpression::Value(SemanticValue::Boolean(true)),
        })
        .expect_err("missing function body type reference");

    assert_unknown_concept(error);
}
