use lexflex::api::LexFlexAPI;
use lexflex::core::interlingua::*;

fn main() {
    let api = LexFlexAPI::builder()
        .data_dir("data")
        .build()
        .expect("Failed to initialize lexFlex");

    // Test English passive
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
    println!("English passive: {}", result);

    // Test Polish passive
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
    println!("Polish passive: {}", result);
}
