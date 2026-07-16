#[path = "support/mod.rs"]
mod support;

use lexflex_lingua::{
    DeclarationId, FunctionDeclaration, FunctionId, LambdaParameter, LinguaDeclaration,
    LinguaExpression, LinguaProgram, ProgramId, SemanticType, SymbolName,
};
use lexflex_model::{EntityId, ParameterId, SemanticExpression};
use std::collections::BTreeMap;

#[test]
fn declared_function_is_callable_by_id() {
    let function = FunctionDeclaration {
        declaration_id: DeclarationId::new("function:identity"),
        function_id: FunctionId::new("identity"),
        name: SymbolName::new("identity"),
        parameters: vec![LambdaParameter {
            name: SymbolName::new("value"),
            parameter_id: ParameterId::new_unchecked("value"),
            value_type: SemanticType::Entity,
        }],
        result_type: SemanticType::Entity,
        body: LinguaExpression::Variable(SymbolName::new("value")),
    };

    let program = LinguaProgram {
        id: ProgramId::new("test:function"),
        declarations: vec![LinguaDeclaration::Function(function)],
        entry: LinguaExpression::Call {
            callee: Box::new(LinguaExpression::Function(FunctionId::new("identity"))),
            arguments: BTreeMap::from([(
                ParameterId::new_unchecked("value"),
                LinguaExpression::Entity(EntityId::new_unchecked("PARIS")),
            )]),
        },
    };

    let result = support::compile_and_execute(program).expect("execute");
    assert_eq!(
        result.value,
        SemanticExpression::Entity(EntityId::new_unchecked("PARIS"))
    );
}
