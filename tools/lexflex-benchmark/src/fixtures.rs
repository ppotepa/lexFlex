use lexflex_lingua::solve::EvidencePolicy;
use lexflex_lingua::{
    ConceptDeclaration, ConceptSemantics, ExpansionPolicy, FunctionDeclaration, LinguaDeclaration,
    LinguaExpression, LinguaGoal, LinguaProgram, ProgramId, SemanticType, SymbolName,
};
use lexflex_model::{
    canonical_hash, ConceptCatalog, ConceptId, ConceptKind, ConceptParameterSchema, ConceptSchema,
    EntityDefinition, EntityId, Evidence, EvidenceId, ParameterId, SemanticAssertion,
    SemanticExpression, SourceSpan, VariableId, WorldId,
};
use std::collections::{BTreeMap, BTreeSet};

pub fn kernel_catalog() -> ConceptCatalog {
    let primary = ConceptId::new_unchecked("PRIMARY_TYPE");
    let secondary = ConceptId::new_unchecked("SECONDARY_TYPE");
    let target = ConceptId::new_unchecked("TARGET_PREDICATE");
    let target_role = ConceptId::new_unchecked("TARGET_ROLE");
    let binds_role = ConceptId::new_unchecked("BINDS_ROLE");

    ConceptCatalog {
        concepts: BTreeMap::from([
            (
                primary.clone(),
                ConceptSchema {
                    id: primary.clone(),
                    kind: ConceptKind::EntityType,
                    parameters: BTreeMap::new(),
                    result_type: SemanticType::Predicate(Box::new(SemanticType::EntityOf(
                        primary.clone(),
                    ))),
                },
            ),
            (
                secondary.clone(),
                ConceptSchema {
                    id: secondary.clone(),
                    kind: ConceptKind::EntityType,
                    parameters: BTreeMap::new(),
                    result_type: SemanticType::Predicate(Box::new(SemanticType::EntityOf(
                        secondary.clone(),
                    ))),
                },
            ),
            (
                target_role.clone(),
                ConceptSchema {
                    id: target_role.clone(),
                    kind: ConceptKind::RoleType,
                    parameters: BTreeMap::new(),
                    result_type: SemanticType::ConceptOf(ConceptKind::RoleType),
                },
            ),
            (
                binds_role.clone(),
                ConceptSchema {
                    id: binds_role.clone(),
                    kind: ConceptKind::RelationType,
                    parameters: BTreeMap::from([
                        (
                            ParameterId::new_unchecked("holder"),
                            ConceptParameterSchema {
                                id: ParameterId::new_unchecked("holder"),
                                value_type: SemanticType::Entity,
                                required: true,
                            },
                        ),
                        (
                            ParameterId::new_unchecked("role"),
                            ConceptParameterSchema {
                                id: ParameterId::new_unchecked("role"),
                                value_type: SemanticType::ConceptOf(ConceptKind::RoleType),
                                required: true,
                            },
                        ),
                        (
                            ParameterId::new_unchecked("scope"),
                            ConceptParameterSchema {
                                id: ParameterId::new_unchecked("scope"),
                                value_type: SemanticType::Entity,
                                required: true,
                            },
                        ),
                    ]),
                    result_type: SemanticType::Boolean,
                },
            ),
            (
                target.clone(),
                ConceptSchema {
                    id: target.clone(),
                    kind: ConceptKind::Predicate,
                    parameters: BTreeMap::from([(
                        ParameterId::new_unchecked("scope"),
                        ConceptParameterSchema {
                            id: ParameterId::new_unchecked("scope"),
                            value_type: SemanticType::EntityOf(secondary.clone()),
                            required: true,
                        },
                    )]),
                    result_type: SemanticType::Predicate(Box::new(SemanticType::EntityOf(
                        primary.clone(),
                    ))),
                },
            ),
        ]),
        entities: BTreeMap::from([
            (
                EntityId::new_unchecked("ENTITY_A"),
                EntityDefinition {
                    id: EntityId::new_unchecked("ENTITY_A"),
                    primary_type: primary.clone(),
                    additional_types: BTreeSet::new(),
                },
            ),
            (
                EntityId::new_unchecked("ENTITY_B"),
                EntityDefinition {
                    id: EntityId::new_unchecked("ENTITY_B"),
                    primary_type: secondary.clone(),
                    additional_types: BTreeSet::new(),
                },
            ),
        ]),
        parents: BTreeMap::from([(secondary, BTreeSet::from([primary]))]),
    }
}

pub fn entity_program() -> LinguaProgram {
    LinguaProgram {
        id: ProgramId::new("benchmark:entity"),
        declarations: Vec::new(),
        entry: LinguaExpression::Entity(EntityId::new_unchecked("ENTITY_A")),
    }
}

pub fn lambda_program() -> LinguaProgram {
    let parameter = ParameterId::new_unchecked("value");
    LinguaProgram {
        id: ProgramId::new("benchmark:lambda"),
        declarations: Vec::new(),
        entry: LinguaExpression::Call {
            callee: Box::new(LinguaExpression::Lambda {
                parameters: vec![lexflex_lingua::LambdaParameter {
                    name: SymbolName::new("value"),
                    parameter_id: parameter.clone(),
                    value_type: SemanticType::Entity,
                }],
                body: Box::new(LinguaExpression::Variable(SymbolName::new("value"))),
            }),
            arguments: BTreeMap::from([(
                parameter,
                LinguaExpression::Entity(EntityId::new_unchecked("ENTITY_A")),
            )]),
        },
    }
}

pub fn concept_application_program(defined: bool) -> LinguaProgram {
    let target = ConceptDeclaration {
        declaration_id: lexflex_lingua::DeclarationId::new("concept:TARGET_PREDICATE"),
        concept_id: ConceptId::new_unchecked("TARGET_PREDICATE"),
        self_parameter: Some(lexflex_lingua::LambdaParameter {
            name: SymbolName::new("self"),
            parameter_id: ParameterId::new_unchecked("self"),
            value_type: SemanticType::EntityOf(ConceptId::new_unchecked("PRIMARY_TYPE")),
        }),
        parameters: vec![lexflex_lingua::LambdaParameter {
            name: SymbolName::new("scope"),
            parameter_id: ParameterId::new_unchecked("scope"),
            value_type: SemanticType::EntityOf(ConceptId::new_unchecked("SECONDARY_TYPE")),
        }],
        semantics: if defined {
            ConceptSemantics::Defined {
                body: LinguaExpression::ApplyConcept {
                    concept: ConceptId::new_unchecked("PRIMARY_TYPE"),
                    bindings: BTreeMap::new(),
                },
            }
        } else {
            ConceptSemantics::Primitive
        },
        expansion: if defined {
            ExpansionPolicy::Transparent
        } else {
            ExpansionPolicy::Opaque
        },
    };

    LinguaProgram {
        id: ProgramId::new(if defined {
            "benchmark:defined-concept-expansion"
        } else {
            "benchmark:concept-application"
        }),
        declarations: vec![LinguaDeclaration::Concept(target)],
        entry: LinguaExpression::Satisfies {
            subject: Box::new(LinguaExpression::Entity(EntityId::new_unchecked(
                "ENTITY_A",
            ))),
            concept: Box::new(LinguaExpression::ApplyConcept {
                concept: ConceptId::new_unchecked("TARGET_PREDICATE"),
                bindings: BTreeMap::from([(
                    ParameterId::new_unchecked("scope"),
                    LinguaExpression::Entity(EntityId::new_unchecked("ENTITY_B")),
                )]),
            }),
        },
    }
}

pub fn global_function_program() -> LinguaProgram {
    let parameter = ParameterId::new_unchecked("value");
    let function_id = lexflex_lingua::FunctionId::new("function:identity");
    LinguaProgram {
        id: ProgramId::new("benchmark:global-function"),
        declarations: vec![LinguaDeclaration::Function(FunctionDeclaration {
            declaration_id: lexflex_lingua::DeclarationId::new("function:identity"),
            function_id: function_id.clone(),
            name: SymbolName::new("identity"),
            parameters: vec![lexflex_lingua::LambdaParameter {
                name: SymbolName::new("value"),
                parameter_id: parameter.clone(),
                value_type: SemanticType::Entity,
            }],
            result_type: SemanticType::Entity,
            body: LinguaExpression::Variable(SymbolName::new("value")),
        })],
        entry: LinguaExpression::Call {
            callee: Box::new(LinguaExpression::Function(function_id)),
            arguments: BTreeMap::from([(
                parameter,
                LinguaExpression::Entity(EntityId::new_unchecked("ENTITY_A")),
            )]),
        },
    }
}

pub fn solver_single_goal() -> LinguaGoal {
    let answer = VariableId::new_unchecked("answer");
    let scope = ParameterId::new_unchecked("scope");
    LinguaGoal {
        expression: SemanticExpression::Satisfies {
            subject: Box::new(SemanticExpression::Variable(answer.clone())),
            predicate: Box::new(SemanticExpression::Apply {
                concept: ConceptId::new_unchecked("TARGET_PREDICATE"),
                bindings: BTreeMap::from([(
                    scope,
                    SemanticExpression::Entity(EntityId::new_unchecked("ENTITY_B")),
                )]),
            }),
        },
        variables: BTreeMap::from([(
            answer,
            SemanticType::EntityOf(ConceptId::new_unchecked("PRIMARY_TYPE")),
        )]),
        projection: vec![VariableId::new_unchecked("answer")],
        evidence_policy: EvidencePolicy::Required,
        world: Some(WorldId::new_unchecked("actual")),
        limit: Some(1),
    }
}

pub fn solver_single_assertions() -> Vec<SemanticAssertion> {
    vec![generic_assertion("ENTITY_A")]
}

pub fn solver_thousand_assertions() -> Vec<SemanticAssertion> {
    std::iter::repeat_with(|| generic_assertion("ENTITY_A"))
        .take(1_000)
        .collect()
}

fn generic_assertion(subject: &str) -> SemanticAssertion {
    let scope = ParameterId::new_unchecked("scope");
    let source_hash = canonical_hash(&(subject, "ENTITY_B"));
    SemanticAssertion::create(
        SemanticExpression::Satisfies {
            subject: Box::new(SemanticExpression::Entity(EntityId::new_unchecked(subject))),
            predicate: Box::new(SemanticExpression::Apply {
                concept: ConceptId::new_unchecked("TARGET_PREDICATE"),
                bindings: BTreeMap::from([(
                    scope,
                    SemanticExpression::Entity(EntityId::new_unchecked("ENTITY_B")),
                )]),
            }),
        },
        vec![Evidence {
            id: EvidenceId::new_unchecked(format!("evidence:{source_hash}")),
            source_id: "benchmark".into(),
            span: Some(SourceSpan { start: 0, end: 1 }),
            source_hash: Some(source_hash),
        }],
        WorldId::new_unchecked("actual"),
    )
}
