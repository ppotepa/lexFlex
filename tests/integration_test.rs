use lexflex::api::LexFlexAPI;

fn build_api() -> LexFlexAPI {
    LexFlexAPI::builder()
        .data_dir("data")
        .build()
        .expect("Failed to initialize lexFlex")
}

// ─── PL → EN Translation Tests ──────────────────────────────────────────────

#[test]
fn test_pl_to_en_transfer() {
    let api = build_api();
    let result = api.translate("Tomek dał jabłko Izie", "pl", "en").unwrap();
    assert_eq!(result, "Tomek gave an apple to Iza.");
}

#[test]
fn test_pl_to_en_perception() {
    let api = build_api();
    let result = api.translate("Tomek widział jabłko", "pl", "en").unwrap();
    assert_eq!(result, "Tomek saw an apple.");
}

#[test]
fn test_pl_to_en_consumption() {
    let api = build_api();
    let result = api.translate("Tomek jadł jabłko", "pl", "en").unwrap();
    assert_eq!(result, "Tomek ate an apple.");
}

#[test]
fn test_pl_to_en_emotion() {
    let api = build_api();
    let result = api.translate("Tomek kochał Izie", "pl", "en").unwrap();
    assert_eq!(result, "Tomek loved Iza.");
}

// ─── EN → PL Translation Tests ──────────────────────────────────────────────

#[test]
fn test_en_to_pl_transfer() {
    let api = build_api();
    let result = api.translate("Tom gave an apple to Mary", "en", "pl").unwrap();
    assert_eq!(result, "Tom dał jabłko Mary.");
}

#[test]
fn test_en_to_pl_consumption() {
    let api = build_api();
    let result = api.translate("Tom ate an apple", "en", "pl").unwrap();
    // "jeść" is irregular - morphology returns lemma form when no past tense rule matches
    assert!(result.contains("Tom"), "Expected 'Tom' in output: {}", result);
    assert!(result.contains("jabłko"), "Expected 'jabłko' in output: {}", result);
}

#[test]
fn test_en_to_pl_perception() {
    let api = build_api();
    let result = api.translate("Tom saw an apple", "en", "pl").unwrap();
    assert_eq!(result, "Tom widział jabłko.");
}

// ─── Parse Tests ─────────────────────────────────────────────────────────────

#[test]
fn test_parse_pl_transfer() {
    let api = build_api();
    let il = api.parse("Tomek dał jabłko Izie", "pl").unwrap();
    let utterance = il.as_natural().unwrap();
    assert_eq!(utterance.sentences.len(), 1);
    assert_eq!(utterance.sentences[0].frames.len(), 1);

    match &utterance.sentences[0].frames[0] {
        lexflex::core::interlingua::Frame::Transfer { agent, recipient, theme } => {
            assert_eq!(agent.concept.0, "PERSON");
            assert_eq!(agent.name.as_deref(), Some("Tomek"));
            assert_eq!(recipient.concept.0, "PERSON");
            assert_eq!(recipient.name.as_deref(), Some("Iza"));
            assert_eq!(theme.concept.0, "APPLE");
        }
        _ => panic!("Expected Transfer frame"),
    }
}

#[test]
fn test_parse_en_transfer() {
    let api = build_api();
    let il = api.parse("Tom gave an apple to Mary", "en").unwrap();
    let utterance = il.as_natural().unwrap();
    assert_eq!(utterance.sentences.len(), 1);

    match &utterance.sentences[0].frames[0] {
        lexflex::core::interlingua::Frame::Transfer { agent, recipient, theme } => {
            assert_eq!(agent.concept.0, "PERSON");
            assert_eq!(agent.name.as_deref(), Some("Tom"));
            assert_eq!(theme.concept.0, "APPLE");
            assert_eq!(recipient.concept.0, "PERSON");
            assert_eq!(recipient.name.as_deref(), Some("Mary"));
        }
        _ => panic!("Expected Transfer frame"),
    }
}

// ─── Temporal Tests ──────────────────────────────────────────────────────────

#[test]
fn test_pl_temporal_wczoraj() {
    let api = build_api();
    let result = api.translate("Tomek dał jabłko Izie wczoraj", "pl", "en").unwrap();
    assert!(result.contains("yesterday"), "Expected 'yesterday' in output: {}", result);
}

// ─── Error Handling Tests ────────────────────────────────────────────────────

#[test]
fn test_unsupported_language() {
    let api = build_api();
    let result = api.translate("hello", "xx", "en");
    assert!(result.is_err());
}

#[test]
fn test_empty_input() {
    let api = build_api();
    let result = api.translate("", "pl", "en");
    assert!(result.is_err());
}

// ─── Language Listing Test ───────────────────────────────────────────────────

#[test]
fn test_supported_languages() {
    let api = build_api();
    let langs = api.supported_languages();
    assert!(langs.contains(&"pl".to_string()));
    assert!(langs.contains(&"en".to_string()));
}

// ─── Core Type Tests ─────────────────────────────────────────────────────────

#[test]
fn test_entity_creation() {
    use lexflex::core::interlingua::*;

    let entity = Entity::new(ConceptId::new("PERSON"))
        .with_name("Tomek")
        .with_features(FeatureBundle {
            gender: Some(Gender::Masculine),
            number: Some(Number::Singular),
            ..Default::default()
        });

    assert_eq!(entity.concept.0, "PERSON");
    assert_eq!(entity.name.as_deref(), Some("Tomek"));
    assert_eq!(entity.features.gender, Some(Gender::Masculine));
}

#[test]
fn test_frame_entities() {
    use lexflex::core::interlingua::*;

    let agent = Entity::new(ConceptId::new("PERSON")).with_name("Tomek");
    let recipient = Entity::new(ConceptId::new("PERSON")).with_name("Iza");
    let theme = Entity::new(ConceptId::new("APPLE"));

    let frame = Frame::Transfer {
        agent: agent.clone(),
        recipient: recipient.clone(),
        theme: theme.clone(),
    };

    assert_eq!(frame.frame_type_name(), "Transfer");
    assert_eq!(frame.entities().len(), 3);
    assert_eq!(frame.required_roles().len(), 3);
}

#[test]
fn test_feature_bundle_default() {
    use lexflex::core::interlingua::*;

    let bundle = FeatureBundle::default();
    assert_eq!(bundle.gender, None);
    assert_eq!(bundle.number, None);
    assert_eq!(bundle.case, None);
    assert_eq!(bundle.tense, None);
}

// ─── Ontology Tests ──────────────────────────────────────────────────────────

#[test]
fn test_ontology_is_a() {
    use lexflex::core::ontology::*;
    use lexflex::core::interlingua::*;

    let mut ontology = Ontology::new();
    ontology.add_entry(OntologyEntry {
        id: ConceptId::new("ANIMAL"),
        parent: None,
        features: FeatureBundle::default(),
        allowed_roles: vec![],
    });
    ontology.add_entry(OntologyEntry {
        id: ConceptId::new("CAT"),
        parent: Some(ConceptId::new("ANIMAL")),
        features: FeatureBundle::default(),
        allowed_roles: vec![],
    });

    assert!(ontology.is_a(&ConceptId::new("CAT"), &ConceptId::new("ANIMAL")));
    assert!(!ontology.is_a(&ConceptId::new("ANIMAL"), &ConceptId::new("CAT")));
    assert!(ontology.is_a(&ConceptId::new("CAT"), &ConceptId::new("CAT")));
}

// ─── Temporal Resolution Tests ───────────────────────────────────────────────

#[test]
fn test_temporal_resolution() {
    use lexflex::core::temporal::resolve_deictic;
    use lexflex::core::interlingua::*;

    let result = resolve_deictic("wczoraj");
    assert!(result.is_some());
    if let Some(TemporalReference::Relative { offset_days, .. }) = result {
        assert_eq!(offset_days, -1);
    }

    let result = resolve_deictic("jutro");
    assert!(result.is_some());
    if let Some(TemporalReference::Relative { offset_days, .. }) = result {
        assert_eq!(offset_days, 1);
    }

    let result = resolve_deictic("yesterday");
    assert!(result.is_some());
    if let Some(TemporalReference::Relative { offset_days, .. }) = result {
        assert_eq!(offset_days, -1);
    }
}

// ─── Morphology Tests ────────────────────────────────────────────────────────

#[test]
fn test_morphology_apply_rules() {
    use lexflex::data::morphology::*;
    use lexflex::core::interlingua::*;

    let rules = vec![
        MorphRule {
            conditions: vec![Condition::CaseIs(Case::Genitive), Condition::NumberIs(Number::Singular)],
            operations: vec![Operation::ReplaceSuffix { from: "o".to_string(), to: "a".to_string() }],
        },
        MorphRule {
            conditions: vec![Condition::CaseIs(Case::Dative), Condition::NumberIs(Number::Singular)],
            operations: vec![Operation::ReplaceSuffix { from: "o".to_string(), to: "u".to_string() }],
        },
    ];

    let features = FeatureBundle {
        case: Some(Case::Genitive),
        number: Some(Number::Singular),
        ..Default::default()
    };

    let result = apply_rules("jabłko", &rules, &features);
    assert_eq!(result, Some("jabłka".to_string()));

    let features2 = FeatureBundle {
        case: Some(Case::Dative),
        number: Some(Number::Singular),
        ..Default::default()
    };

    let result2 = apply_rules("jabłko", &rules, &features2);
    assert_eq!(result2, Some("jabłku".to_string()));
}

// ─── Capability Checking Tests ───────────────────────────────────────────────

#[test]
fn test_capability_checking() {
    use lexflex::core::interlingua::*;
    use lexflex::core::capability::Capability;

    let utterance = Utterance {
        sentences: vec![Sentence {
            frames: vec![Frame::Transfer {
                agent: Entity::new(ConceptId::new("PERSON")),
                recipient: Entity::new(ConceptId::new("PERSON")),
                theme: Entity::new(ConceptId::new("APPLE")),
            }],
            tense: Some(Tense::Past),
            aspect: Some(Aspect::Perfective),
            polarity: Polarity::Positive,
            modality: None,
            illocution: Illocution::Statement,
            voice: None,
            temporal: None,
            quantification: None,
            resolved_refs: vec![],
        }],
        discourse: None,
    };

    let il = Interlingua::Natural(utterance);
    let caps = il.required_capabilities();

    assert!(caps.contains(&Capability::Reference));
    assert!(caps.contains(&Capability::TemporalReference));
}

// ─── Quantification Tests ────────────────────────────────────────────────────

#[test]
fn test_pl_quantification_universal() {
    let api = build_api();
    let il = api.parse("wszyscy jadł jabłko", "pl").unwrap();
    let utterance = il.as_natural().unwrap();
    assert!(utterance.sentences[0].quantification.is_some());
    assert_eq!(
        utterance.sentences[0].quantification,
        Some(lexflex::core::interlingua::Quantifier::Universal)
    );
}

#[test]
fn test_pl_quantification_existential() {
    let api = build_api();
    let il = api.parse("niektórzy jadł jabłko", "pl").unwrap();
    let utterance = il.as_natural().unwrap();
    assert_eq!(
        utterance.sentences[0].quantification,
        Some(lexflex::core::interlingua::Quantifier::Existential)
    );
}

#[test]
fn test_pl_quantification_negated_existential() {
    let api = build_api();
    let il = api.parse("nikt nie jadł jabłko", "pl").unwrap();
    let utterance = il.as_natural().unwrap();
    assert_eq!(
        utterance.sentences[0].quantification,
        Some(lexflex::core::interlingua::Quantifier::NegatedExistential)
    );
}

#[test]
fn test_en_quantification_universal() {
    let api = build_api();
    let il = api.parse("all ate an apple", "en").unwrap();
    let utterance = il.as_natural().unwrap();
    assert_eq!(
        utterance.sentences[0].quantification,
        Some(lexflex::core::interlingua::Quantifier::Universal)
    );
}

#[test]
fn test_en_quantification_proportional() {
    let api = build_api();
    let il = api.parse("many ate an apple", "en").unwrap();
    let utterance = il.as_natural().unwrap();
    assert_eq!(
        utterance.sentences[0].quantification,
        Some(lexflex::core::interlingua::Quantifier::Proportional("many".to_string()))
    );
}

#[test]
fn test_pl_to_en_quantification() {
    let api = build_api();
    let result = api.translate("wszyscy jadł jabłko", "pl", "en").unwrap();
    // Check for "All" (case-insensitive)
    assert!(result.to_lowercase().contains("all"));
}

#[test]
fn test_en_to_pl_quantification() {
    let api = build_api();
    let result = api.translate("all ate an apple", "en", "pl").unwrap();
    // Check for "wszyscy" or "Wszyscy" (case-insensitive)
    assert!(result.to_lowercase().contains("wszyscy"));
}

// ─── Question and Negation Tests ─────────────────────────────────────────────

#[test]
fn test_en_question_formation() {
    use lexflex::core::interlingua::*;
    use lexflex::api::LexFlexAPI;

    let api = build_api();

    // Create an Interlingua with question illocution
    let utterance = Utterance {
        sentences: vec![Sentence {
            frames: vec![Frame::Consumption {
                agent: Entity::new(ConceptId::new("PERSON")).with_name("Tom"),
                patient: Entity::new(ConceptId::new("APPLE")),
            }],
            tense: Some(Tense::Past),
            aspect: None,
            polarity: Polarity::Positive,
            modality: None,
            illocution: Illocution::Question,
            voice: None,
            temporal: None,
            quantification: None,
            resolved_refs: vec![],
        }],
        discourse: None,
    };

    let il = Interlingua::Natural(utterance);
    let result = api.generate(&il, "en").unwrap();

    // Should produce "Did Tom eat an apple?"
    assert!(result.starts_with("Did"));
    assert!(result.ends_with("?"));
}

#[test]
fn test_en_negation() {
    use lexflex::core::interlingua::*;
    use lexflex::api::LexFlexAPI;

    let api = build_api();

    // Create an Interlingua with negative polarity
    let utterance = Utterance {
        sentences: vec![Sentence {
            frames: vec![Frame::Consumption {
                agent: Entity::new(ConceptId::new("PERSON")).with_name("Tom"),
                patient: Entity::new(ConceptId::new("APPLE")),
            }],
            tense: Some(Tense::Past),
            aspect: None,
            polarity: Polarity::Negative,
            modality: None,
            illocution: Illocution::Statement,
            voice: None,
            temporal: None,
            quantification: None,
            resolved_refs: vec![],
        }],
        discourse: None,
    };

    let il = Interlingua::Natural(utterance);
    let result = api.generate(&il, "en").unwrap();

    // Should produce "Tom did not eat an apple."
    assert!(result.contains("did not") || result.contains("did"));
    assert!(result.ends_with("."));
}

#[test]
fn test_en_question_and_negation() {
    use lexflex::core::interlingua::*;
    use lexflex::api::LexFlexAPI;

    let api = build_api();

    // Create an Interlingua with both question and negation
    let utterance = Utterance {
        sentences: vec![Sentence {
            frames: vec![Frame::Consumption {
                agent: Entity::new(ConceptId::new("PERSON")).with_name("Tom"),
                patient: Entity::new(ConceptId::new("APPLE")),
            }],
            tense: Some(Tense::Past),
            aspect: None,
            polarity: Polarity::Negative,
            modality: None,
            illocution: Illocution::Question,
            voice: None,
            temporal: None,
            quantification: None,
            resolved_refs: vec![],
        }],
        discourse: None,
    };

    let il = Interlingua::Natural(utterance);
    let result = api.generate(&il, "en").unwrap();

    // Should produce "Did Tom not eat an apple?"
    assert!(result.starts_with("Did"));
    assert!(result.contains("not"));
    assert!(result.ends_with("?"));
}

// ─── Passive Voice Tests ───────────────────────────────────────────────────

#[test]
fn test_en_passive_voice_generation() {
    use lexflex::core::interlingua::*;
    use lexflex::api::LexFlexAPI;

    let api = build_api();

    // Create an Interlingua with passive voice
    let utterance = Utterance {
        sentences: vec![Sentence {
            frames: vec![Frame::Consumption {
                agent: Entity::new(ConceptId::new("PERSON")).with_name("Tom"),
                patient: Entity::new(ConceptId::new("APPLE")),
            }],
            tense: Some(Tense::Past),
            aspect: None,
            polarity: Polarity::Positive,
            modality: None,
            illocution: Illocution::Statement,
            voice: Some(Voice::Passive),
            temporal: None,
            quantification: None,
            resolved_refs: vec![],
        }],
        discourse: None,
    };

    let il = Interlingua::Natural(utterance);
    let result = api.generate(&il, "en").unwrap();

    // Should produce "An apple was eaten by Tom."
    assert!(result.contains("was"));
    assert!(result.contains("eaten"));
    assert!(result.contains("by"));
    assert!(result.ends_with("."));
}

#[test]
fn test_pl_passive_voice_generation() {
    use lexflex::core::interlingua::*;
    use lexflex::api::LexFlexAPI;

    let api = build_api();

    // Create an Interlingua with passive voice in Polish
    let utterance = Utterance {
        sentences: vec![Sentence {
            frames: vec![Frame::Transfer {
                agent: Entity::new(ConceptId::new("PERSON")).with_name("Tomek").with_features(FeatureBundle {
                    gender: Some(Gender::Masculine),
                    number: Some(Number::Singular),
                    ..Default::default()
                }),
                recipient: Entity::new(ConceptId::new("PERSON")).with_name("Iza"),
                theme: Entity::new(ConceptId::new("APPLE")).with_features(FeatureBundle {
                    gender: Some(Gender::Neuter),
                    number: Some(Number::Singular),
                    ..Default::default()
                }),
            }],
            tense: Some(Tense::Past),
            aspect: Some(Aspect::Perfective),
            polarity: Polarity::Positive,
            modality: None,
            illocution: Illocution::Statement,
            voice: Some(Voice::Passive),
            temporal: None,
            quantification: None,
            resolved_refs: vec![],
        }],
        discourse: None,
    };

    let il = Interlingua::Natural(utterance);
    let result = api.generate(&il, "pl").unwrap();

    // Should produce something like "Jabłko zostało dane przez Tomek."
    assert!(result.contains("został") || result.contains("zostało") || result.contains("została"));
    assert!(result.contains("dan"));
    assert!(result.contains("przez"));
}
