use lexflex_engine::catalog::{
    model_loader::validate_concept_programs, ModelLoadError, ModelPackageLoader,
};
use lexflex_lingua::{
    ConceptDeclaration, ConceptSemantics, DeclarationId, ExpansionPolicy, LinguaDeclaration,
    LinguaExpression, LinguaProgram,
};
use lexflex_model::{ConceptCatalog, ConceptId, EntityId, SemanticValue};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_root() -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should move forward")
        .as_nanos();
    std::env::temp_dir().join(format!("lexflex-model-loader-{stamp}"))
}

fn repo_model_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/model")
}

#[test]
fn unknown_concept_program_target_is_rejected() {
    let root = temp_root();
    std::fs::create_dir_all(&root).expect("create root");
    std::fs::write(
        root.join("manifest.ron"),
        r#"(
    schema: 1,
    package_id: "lexflex:model:test",
    concepts: "concepts.ron",
    entities: "entities.ron",
    programs: "programs.ron",
)"#,
    )
    .expect("write manifest");
    std::fs::copy(
        repo_model_root().join("concepts.ron"),
        root.join("concepts.ron"),
    )
    .expect("copy concepts");
    std::fs::copy(
        repo_model_root().join("entities.ron"),
        root.join("entities.ron"),
    )
    .expect("copy entities");
    let program = LinguaProgram {
        id: lexflex_lingua::ProgramId::new_unchecked("program:test:invalid"),
        declarations: vec![LinguaDeclaration::Concept(ConceptDeclaration {
            declaration_id: DeclarationId::new_unchecked("decl:test:invalid"),
            concept_id: ConceptId::new_unchecked("MISSING"),
            self_parameter: None,
            parameters: Vec::new(),
            semantics: ConceptSemantics::Defined {
                body: LinguaExpression::Entity(EntityId::new_unchecked("PARIS")),
            },
            expansion: ExpansionPolicy::Opaque,
        })],
        entry: LinguaExpression::Entity(EntityId::new_unchecked("PARIS")),
    };
    let programs = ron::ser::to_string(&vec![program]).expect("serialize programs");
    std::fs::write(root.join("programs.ron"), programs).expect("write programs");

    let err = ModelPackageLoader
        .load(&root)
        .expect_err("unknown concept must fail");
    assert!(
        matches!(err, ModelLoadError::Validation(_)),
        "unexpected error: {err:?}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn defined_concept_body_result_mismatch_is_rejected() {
    let catalog = ConceptCatalog::default();
    let program = LinguaProgram {
        id: lexflex_lingua::ProgramId::new_unchecked("program:test:mismatch"),
        declarations: vec![LinguaDeclaration::Concept(ConceptDeclaration {
            declaration_id: DeclarationId::new_unchecked("decl:test:mismatch"),
            concept_id: ConceptId::new_unchecked("BROKEN"),
            self_parameter: None,
            parameters: Vec::new(),
            semantics: ConceptSemantics::Defined {
                body: LinguaExpression::Value(SemanticValue::from(1_i64)),
            },
            expansion: ExpansionPolicy::Opaque,
        })],
        entry: LinguaExpression::Value(SemanticValue::from(1_i64)),
    };

    let err = validate_concept_programs(&catalog, &[program])
        .expect_err("mismatched defined body must fail");
    assert!(matches!(err, ModelLoadError::Validation(_)));
}

#[test]
fn entity_type_must_return_predicate_shape() {
    let root = temp_root();
    std::fs::create_dir_all(&root).expect("create root");
    std::fs::write(
        root.join("manifest.ron"),
        r#"(
    schema: 1,
    package_id: "lexflex:model:test:shape",
    concepts: "concepts.ron",
    entities: "entities.ron",
    programs: "programs.ron",
)"#,
    )
    .expect("write manifest");
    std::fs::write(
        root.join("concepts.ron"),
        r#"(
    concepts: {
        "BROKEN": (
            id: "BROKEN",
            kind: EntityType,
            parameters: {},
            result_type: Boolean,
        ),
    },
    entities: {},
    parents: {},
)"#,
    )
    .expect("write concepts");
    std::fs::write(root.join("entities.ron"), r#"( entities: {}, )"#).expect("write entities");
    std::fs::write(root.join("programs.ron"), "[]").expect("write programs");

    let err = ModelPackageLoader
        .load(&root)
        .expect_err("invalid concept shape must fail");
    assert!(
        matches!(err, ModelLoadError::Validation(_)),
        "unexpected error: {err:?}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn unsafe_package_paths_are_rejected() {
    for (field, unsafe_path) in [
        ("concepts", "../concepts.ron"),
        ("entities", "../entities.ron"),
        ("programs", "../programs.ron"),
        ("concepts", "/concepts.ron"),
        ("entities", "/entities.ron"),
        ("programs", "/programs.ron"),
    ] {
        let root = temp_root();
        std::fs::create_dir_all(&root).expect("create root");
        for file in ["concepts.ron", "entities.ron", "concept_programs.ron"] {
            std::fs::copy(repo_model_root().join(file), root.join(file)).expect("copy baseline");
        }
        std::fs::write(
            root.join("manifest.ron"),
            format!(
                r#"(
    schema: 1,
    package_id: "lexflex:model:unsafe",
    concepts: "{concepts}",
    entities: "{entities}",
    programs: "{programs}",
)"#,
                concepts = if field == "concepts" {
                    unsafe_path
                } else {
                    "concepts.ron"
                },
                entities = if field == "entities" {
                    unsafe_path
                } else {
                    "entities.ron"
                },
                programs = if field == "programs" {
                    unsafe_path
                } else {
                    "programs.ron"
                }
            ),
        )
        .expect("write manifest");

        let err = ModelPackageLoader
            .load(&root)
            .expect_err("unsafe paths must fail");
        assert!(
            matches!(err, ModelLoadError::UnsafePath { .. }),
            "unexpected error: {err:?}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }
}
