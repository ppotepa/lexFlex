use lexflex_engine::catalog::{ModelProgramRegistry, ProgramRegistryError};
use lexflex_lingua::{
    ConceptDeclaration, ConceptSemantics, DeclarationId, ExpansionPolicy, LinguaDeclaration,
    LinguaExpression, LinguaProgram, ProgramId,
};
use lexflex_model::{ConceptCatalog, ConceptId, EntityId};

fn catalog() -> ConceptCatalog {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/model");
    let loaded = lexflex_engine::catalog::ModelPackageLoader
        .load(&root)
        .expect("load model");
    loaded.catalog
}

fn concept_program(program_id: &str, declaration_id: &str, concept_id: &str) -> LinguaProgram {
    LinguaProgram {
        id: ProgramId::new_unchecked(program_id),
        declarations: vec![LinguaDeclaration::Concept(ConceptDeclaration {
            declaration_id: DeclarationId::new_unchecked(declaration_id),
            concept_id: ConceptId::new_unchecked(concept_id),
            self_parameter: None,
            parameters: Vec::new(),
            semantics: ConceptSemantics::Defined {
                body: LinguaExpression::Entity(EntityId::new_unchecked("PARIS")),
            },
            expansion: ExpansionPolicy::Opaque,
        })],
        entry: LinguaExpression::Entity(EntityId::new_unchecked("PARIS")),
    }
}

#[test]
fn registry_hash_is_stable_across_input_order() {
    let catalog = catalog();
    let first = concept_program("program:test:1", "decl:test:1", "CITY");
    let second = concept_program("program:test:2", "decl:test:2", "COUNTRY");

    let left = ModelProgramRegistry::build(vec![first.clone(), second.clone()], &catalog)
        .expect("left registry");
    let right = ModelProgramRegistry::build(vec![second, first], &catalog).expect("right registry");

    assert_eq!(left.declarations(), right.declarations());
    assert_eq!(left.declaration_hash(), right.declaration_hash());
}

#[test]
fn duplicate_program_ids_are_rejected() {
    let catalog = catalog();
    let first = concept_program("program:test:dup", "decl:test:1", "CITY");
    let second = concept_program("program:test:dup", "decl:test:2", "COUNTRY");

    let error = ModelProgramRegistry::build(vec![first, second], &catalog).expect_err("duplicate");
    assert!(matches!(error, ProgramRegistryError::DuplicateProgram(_)));
}

#[test]
fn unknown_concept_declaration_is_rejected() {
    let catalog = catalog();
    let error = ModelProgramRegistry::build(
        vec![concept_program("program:test", "decl:test", "MISSING")],
        &catalog,
    )
    .expect_err("unknown concept");
    assert!(matches!(error, ProgramRegistryError::UnknownConcept(_)));
}
