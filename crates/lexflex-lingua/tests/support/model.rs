use lexflex_lingua::{
    ConceptDeclaration, ConceptSemantics, DeclarationId, ExpansionPolicy, LambdaParameter,
    LinguaExpression, ProgramId, SemanticType, SymbolName, ValueType,
};
use lexflex_model::{
    ConceptCatalog, ConceptId, ConceptKind, ConceptParameterSchema, ConceptSchema,
    EntityDefinition, EntityId, ParameterId,
};
use std::collections::{BTreeMap, BTreeSet};

pub fn kernel_catalog() -> ConceptCatalog {
    let city = ConceptId::new_unchecked("CITY");
    let polity = ConceptId::new_unchecked("POLITY");
    let country = ConceptId::new_unchecked("COUNTRY");
    let capital = ConceptId::new_unchecked("CAPITAL");
    let capital_role = ConceptId::new_unchecked("CAPITAL_ROLE");
    let fills_role = ConceptId::new_unchecked("FILLS_ROLE");
    let entity = ConceptId::new_unchecked("ENTITY");

    ConceptCatalog {
        concepts: BTreeMap::from([
            (
                entity.clone(),
                ConceptSchema {
                    id: entity.clone(),
                    kind: ConceptKind::EntityType,
                    parameters: BTreeMap::new(),
                    result_type: SemanticType::Predicate(Box::new(SemanticType::Entity)),
                },
            ),
            (
                city.clone(),
                ConceptSchema {
                    id: city.clone(),
                    kind: ConceptKind::EntityType,
                    parameters: BTreeMap::new(),
                    result_type: SemanticType::Predicate(Box::new(SemanticType::EntityOf(
                        city.clone(),
                    ))),
                },
            ),
            (
                polity.clone(),
                ConceptSchema {
                    id: polity.clone(),
                    kind: ConceptKind::EntityType,
                    parameters: BTreeMap::new(),
                    result_type: SemanticType::Predicate(Box::new(SemanticType::EntityOf(
                        polity.clone(),
                    ))),
                },
            ),
            (
                country.clone(),
                ConceptSchema {
                    id: country.clone(),
                    kind: ConceptKind::EntityType,
                    parameters: BTreeMap::new(),
                    result_type: SemanticType::Predicate(Box::new(SemanticType::EntityOf(
                        country.clone(),
                    ))),
                },
            ),
            (
                capital_role.clone(),
                ConceptSchema {
                    id: capital_role.clone(),
                    kind: ConceptKind::RoleType,
                    parameters: BTreeMap::new(),
                    result_type: SemanticType::ConceptOf(ConceptKind::RoleType),
                },
            ),
            (
                fills_role.clone(),
                ConceptSchema {
                    id: fills_role.clone(),
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
                capital.clone(),
                ConceptSchema {
                    id: capital.clone(),
                    kind: ConceptKind::Predicate,
                    parameters: BTreeMap::from([(
                        ParameterId::new_unchecked("scope"),
                        ConceptParameterSchema {
                            id: ParameterId::new_unchecked("scope"),
                            value_type: SemanticType::EntityOf(polity.clone()),
                            required: true,
                        },
                    )]),
                    result_type: SemanticType::Predicate(Box::new(SemanticType::EntityOf(
                        city.clone(),
                    ))),
                },
            ),
        ]),
        entities: BTreeMap::from([
            (
                EntityId::new_unchecked("PARIS"),
                EntityDefinition {
                    id: EntityId::new_unchecked("PARIS"),
                    primary_type: city,
                    additional_types: BTreeSet::new(),
                },
            ),
            (
                EntityId::new_unchecked("FRANCE"),
                EntityDefinition {
                    id: EntityId::new_unchecked("FRANCE"),
                    primary_type: country,
                    additional_types: BTreeSet::new(),
                },
            ),
            (
                EntityId::new_unchecked("WARSAW"),
                EntityDefinition {
                    id: EntityId::new_unchecked("WARSAW"),
                    primary_type: ConceptId::new_unchecked("CITY"),
                    additional_types: BTreeSet::new(),
                },
            ),
            (
                EntityId::new_unchecked("POLAND"),
                EntityDefinition {
                    id: EntityId::new_unchecked("POLAND"),
                    primary_type: ConceptId::new_unchecked("COUNTRY"),
                    additional_types: BTreeSet::new(),
                },
            ),
        ]),
        parents: BTreeMap::from([(
            ConceptId::new_unchecked("COUNTRY"),
            BTreeSet::from([polity]),
        )]),
    }
}

pub fn capital_declaration() -> ConceptDeclaration {
    let scope = ParameterId::new_unchecked("scope");
    ConceptDeclaration {
        declaration_id: DeclarationId::new("concept:CAPITAL"),
        concept_id: ConceptId::new_unchecked("CAPITAL"),
        self_parameter: Some(LambdaParameter {
            name: SymbolName::new("self"),
            parameter_id: ParameterId::new_unchecked("self"),
            value_type: SemanticType::EntityOf(ConceptId::new_unchecked("CITY")),
        }),
        parameters: vec![LambdaParameter {
            name: SymbolName::new("scope"),
            parameter_id: scope,
            value_type: SemanticType::EntityOf(ConceptId::new_unchecked("POLITY")),
        }],
        semantics: ConceptSemantics::Primitive,
        expansion: ExpansionPolicy::Transparent,
    }
}

pub fn city_declaration() -> ConceptDeclaration {
    ConceptDeclaration {
        declaration_id: DeclarationId::new("concept:CITY"),
        concept_id: ConceptId::new_unchecked("CITY"),
        self_parameter: Some(LambdaParameter {
            name: SymbolName::new("self"),
            parameter_id: ParameterId::new_unchecked("self"),
            value_type: SemanticType::EntityOf(ConceptId::new_unchecked("CITY")),
        }),
        parameters: Vec::new(),
        semantics: ConceptSemantics::Primitive,
        expansion: ExpansionPolicy::Transparent,
    }
}

pub fn population_declaration() -> ConceptDeclaration {
    let subject = ParameterId::new_unchecked("subject");
    let time = ParameterId::new_unchecked("time");
    ConceptDeclaration {
        declaration_id: DeclarationId::new("concept:POPULATION"),
        concept_id: ConceptId::new_unchecked("POPULATION"),
        self_parameter: None,
        parameters: vec![
            LambdaParameter {
                name: SymbolName::new("subject"),
                parameter_id: subject,
                value_type: SemanticType::Entity,
            },
            LambdaParameter {
                name: SymbolName::new("time"),
                parameter_id: time,
                value_type: SemanticType::Value(ValueType::Date),
            },
        ],
        semantics: ConceptSemantics::Primitive,
        expansion: ExpansionPolicy::Opaque,
    }
}

pub fn recursive_capital_declaration() -> ConceptDeclaration {
    let mut declaration = capital_declaration();
    declaration.semantics = ConceptSemantics::Defined {
        body: LinguaExpression::ApplyConcept {
            concept: ConceptId::new_unchecked("CAPITAL"),
            bindings: BTreeMap::from([(
                ParameterId::new_unchecked("scope"),
                LinguaExpression::Entity(EntityId::new_unchecked("FRANCE")),
            )]),
        },
    };
    declaration
}

pub fn capital_program() -> lexflex_lingua::LinguaProgram {
    use lexflex_lingua::{LinguaDeclaration, LinguaExpression};

    let scope = ParameterId::new_unchecked("scope");
    lexflex_lingua::LinguaProgram {
        id: ProgramId::new("test:capital"),
        declarations: vec![LinguaDeclaration::Concept(capital_declaration())],
        entry: LinguaExpression::Satisfies {
            subject: Box::new(LinguaExpression::Entity(EntityId::new_unchecked("PARIS"))),
            concept: Box::new(LinguaExpression::ApplyConcept {
                concept: ConceptId::new_unchecked("CAPITAL"),
                bindings: BTreeMap::from([(
                    scope,
                    LinguaExpression::Entity(EntityId::new_unchecked("FRANCE")),
                )]),
            }),
        },
    }
}

pub fn identity_program() -> lexflex_lingua::LinguaProgram {
    use lexflex_lingua::{LambdaParameter, LinguaExpression};
    let parameter = ParameterId::new_unchecked("value");
    let name = lexflex_lingua::SymbolName::new("value");
    lexflex_lingua::LinguaProgram {
        id: ProgramId::new("test:identity"),
        declarations: Vec::new(),
        entry: LinguaExpression::Call {
            callee: Box::new(LinguaExpression::Lambda {
                parameters: vec![LambdaParameter {
                    name,
                    parameter_id: parameter.clone(),
                    value_type: SemanticType::Entity,
                }],
                body: Box::new(LinguaExpression::Variable(lexflex_lingua::SymbolName::new(
                    "value",
                ))),
            }),
            arguments: BTreeMap::from([(
                parameter,
                LinguaExpression::Entity(EntityId::new_unchecked("PARIS")),
            )]),
        },
    }
}

pub fn let_program() -> lexflex_lingua::LinguaProgram {
    use lexflex_lingua::LinguaExpression;
    lexflex_lingua::LinguaProgram {
        id: ProgramId::new("test:let"),
        declarations: Vec::new(),
        entry: LinguaExpression::Let {
            name: lexflex_lingua::SymbolName::new("x"),
            value: Box::new(LinguaExpression::Entity(EntityId::new_unchecked("PARIS"))),
            body: Box::new(LinguaExpression::Variable(lexflex_lingua::SymbolName::new(
                "x",
            ))),
        },
    }
}

pub fn deeply_nested_not_program(depth: usize) -> lexflex_lingua::LinguaProgram {
    use lexflex_lingua::LinguaExpression;
    let mut expression = LinguaExpression::Equals {
        left: Box::new(LinguaExpression::Entity(EntityId::new_unchecked("PARIS"))),
        right: Box::new(LinguaExpression::Entity(EntityId::new_unchecked("PARIS"))),
    };
    for _ in 0..depth {
        expression = LinguaExpression::Not(Box::new(expression));
    }
    lexflex_lingua::LinguaProgram {
        id: ProgramId::new("test:deep-not"),
        declarations: Vec::new(),
        entry: expression,
    }
}
