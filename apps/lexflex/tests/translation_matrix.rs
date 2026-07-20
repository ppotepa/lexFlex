#[path = "command_support.rs"]
#[allow(dead_code)]
mod command_support;

#[test]
fn a2_assertion_translation_preserves_semantic_hash() {
    let cases = [
        (
            "en",
            "pl",
            "Paris is the capital of France.",
            "Paryż jest stolicą Francji",
        ),
        (
            "pl",
            "en",
            "Paryż jest stolicą Francji.",
            "Paris is the capital of France",
        ),
        (
            "en",
            "pl",
            "Warsaw is the capital of Poland.",
            "Warszawa jest stolicą Polski",
        ),
        (
            "pl",
            "en",
            "Warszawa jest stolicą Polski.",
            "Warsaw is the capital of Poland",
        ),
        ("en", "pl", "Tom sees Iza.", "Tomek widzi Izę"),
        ("pl", "en", "Tomek widzi Izę.", "Tom sees Iza"),
        ("en", "pl", "Iza sees Tom.", "Iza widzi Tomka"),
        ("pl", "en", "Iza widzi Tomka.", "Iza sees Tom"),
        (
            "en",
            "pl",
            "What is the capital of Poland?",
            "Jaka jest stolica Polski?",
        ),
        (
            "pl",
            "en",
            "Jaka jest stolica Polski?",
            "What is the capital of Poland?",
        ),
        ("en", "pl", "Who sees Tom?", "Kto widzi Tomka?"),
        ("pl", "en", "Kto widzi Tomka?", "Who sees Tom?"),
        ("en", "pl", "Tom sees who?", "Tomek widzi Kogo?"),
        ("pl", "en", "Tomek widzi Kogo?", "Tom sees who?"),
    ];

    let results = cases
        .iter()
        .enumerate()
        .map(|(index, (source, target, input, expected))| {
            assert_translation(index, source, target, input, expected)
        })
        .collect::<Vec<_>>();
    command_support::write_level_report("a2-seed", &results);
    command_support::assert_level_passed("a2-seed", &results);
}

#[derive(Clone, Copy)]
struct PersonCase {
    en: &'static str,
    pl_subject: &'static str,
    pl_object: &'static str,
}

fn assert_translation(
    index: usize,
    source_language: &str,
    target_language: &str,
    source: &str,
    expected: &str,
) -> command_support::LevelCaseResult {
    let output = command_support::run(&[
        "text-translate-text",
        "--language",
        source_language,
        "--target-language",
        target_language,
        "--text",
        source,
    ]);
    let parsed = serde_json::from_slice::<serde_json::Value>(&output.stdout);
    let payload = parsed.as_ref().ok();
    let actual = payload
        .and_then(|value| value["text"].as_str())
        .unwrap_or_default()
        .to_owned();
    let semantic_hash_equal = payload
        .map(|value| value["source_semantic_hash"] == value["target_semantic_hash"])
        .unwrap_or(false);
    let passed = output.code == Some(0) && actual == expected && semantic_hash_equal;
    let error = (!passed).then(|| {
        format!(
            "code={:?}, json={}, stderr={}",
            output.code,
            parsed.is_ok(),
            String::from_utf8_lossy(&output.stderr)
        )
    });
    command_support::LevelCaseResult {
        id: format!("A2-{}-{index:03}", source_language.to_uppercase()),
        source_language: source_language.to_owned(),
        target_language: target_language.to_owned(),
        source: source.to_owned(),
        expected: expected.to_owned(),
        actual,
        semantic_hash_equal,
        passed,
        error,
    }
}

fn assert_b1_translation(
    index: usize,
    source_language: &str,
    target_language: &str,
    source: &str,
    expected: &str,
) -> command_support::LevelCaseResult {
    let mut result = assert_translation(index, source_language, target_language, source, expected);
    result.id = format!("B1-{}-{index:03}", source_language.to_uppercase());
    result
}

#[test]
fn a2_reaches_fifty_english_and_polish_source_sentences() {
    let people = [
        PersonCase {
            en: "Tom",
            pl_subject: "Tomek",
            pl_object: "Tomka",
        },
        PersonCase {
            en: "Iza",
            pl_subject: "Iza",
            pl_object: "Izę",
        },
        PersonCase {
            en: "Thomas",
            pl_subject: "Tomasz",
            pl_object: "Tomasza",
        },
        PersonCase {
            en: "Tommy",
            pl_subject: "Tomcio",
            pl_object: "Tomcia",
        },
        PersonCase {
            en: "Isabelle",
            pl_subject: "Izabela",
            pl_object: "Izabelę",
        },
        PersonCase {
            en: "Izzy",
            pl_subject: "Izka",
            pl_object: "Izkę",
        },
    ];
    let canonical_en = ["Tom", "Iza", "Tom", "Tom", "Iza", "Iza"];
    let canonical_pl_subject = ["Tomek", "Iza", "Tomek", "Tomek", "Iza", "Iza"];
    let canonical_pl_object = ["Tomka", "Izę", "Tomka", "Tomka", "Izę", "Izę"];

    let mut english_cases = 0;
    let mut polish_cases = 0;
    let mut results = Vec::new();
    for (agent_index, agent) in people.iter().enumerate() {
        for (patient_index, patient) in people.iter().enumerate() {
            let expected_pl = format!(
                "{} widzi {}",
                canonical_pl_subject[agent_index], canonical_pl_object[patient_index]
            );
            let expected_en = format!(
                "{} sees {}",
                canonical_en[agent_index], canonical_en[patient_index]
            );
            results.push(assert_translation(
                english_cases,
                "en",
                "pl",
                &format!("{} sees {}.", agent.en, patient.en),
                &expected_pl,
            ));
            english_cases += 1;
            results.push(assert_translation(
                polish_cases,
                "pl",
                "en",
                &format!("{} widzi {}.", agent.pl_subject, patient.pl_object),
                &expected_en,
            ));
            polish_cases += 1;
        }
    }

    for (index, patient) in people.iter().take(6).enumerate() {
        let expected_pl = format!("Kto widzi {}?", canonical_pl_object[index]);
        let expected_en = format!("Who sees {}?", canonical_en[index]);
        results.push(assert_translation(
            english_cases,
            "en",
            "pl",
            &format!("Who sees {}?", patient.en),
            &expected_pl,
        ));
        english_cases += 1;
        results.push(assert_translation(
            polish_cases,
            "pl",
            "en",
            &format!("Kto widzi {}?", patient.pl_object),
            &expected_en,
        ));
        polish_cases += 1;
    }

    for (index, agent) in people.iter().take(6).enumerate() {
        let expected_pl = format!("{} widzi Kogo?", canonical_pl_subject[index]);
        let expected_en = format!("{} sees who?", canonical_en[index]);
        results.push(assert_translation(
            english_cases,
            "en",
            "pl",
            &format!("{} sees who?", agent.en),
            &expected_pl,
        ));
        english_cases += 1;
        results.push(assert_translation(
            polish_cases,
            "pl",
            "en",
            &format!("{} widzi Kogo?", agent.pl_subject),
            &expected_en,
        ));
        polish_cases += 1;
    }

    results.push(assert_translation(
        english_cases,
        "en",
        "pl",
        "Paris is the capital of France.",
        "Paryż jest stolicą Francji",
    ));
    english_cases += 1;
    results.push(assert_translation(
        english_cases,
        "en",
        "pl",
        "Warsaw is the capital of Poland.",
        "Warszawa jest stolicą Polski",
    ));
    english_cases += 1;
    results.push(assert_translation(
        polish_cases,
        "pl",
        "en",
        "Paryż jest stolicą Francji.",
        "Paris is the capital of France",
    ));
    polish_cases += 1;
    results.push(assert_translation(
        polish_cases,
        "pl",
        "en",
        "Warszawa jest stolicą Polski.",
        "Warsaw is the capital of Poland",
    ));
    polish_cases += 1;

    assert_eq!(english_cases, 50);
    assert_eq!(polish_cases, 50);
    assert_eq!(results.len(), 100);
    command_support::write_level_report("a2", &results);
    command_support::assert_level_passed("a2", &results);
}

#[test]
fn b1_reaches_fifty_english_and_polish_varied_sentences() {
    let people = [
        ("Tom", "Tomek", "Tomka", "Tom"),
        ("Iza", "Iza", "Izę", "Iza"),
        ("Thomas", "Tomasz", "Tomasza", "Tom"),
        ("Tommy", "Tomcio", "Tomcia", "Tom"),
        ("Isabelle", "Izabela", "Izabelę", "Iza"),
        ("Izzy", "Izka", "Izkę", "Iza"),
    ];
    let mut english_cases = 0;
    let mut polish_cases = 0;
    let mut results = Vec::new();

    for (agent, agent_pl, _, agent_canonical) in people {
        for (patient, _, patient_pl_object, patient_canonical) in people {
            results.push(assert_b1_translation(
                english_cases,
                "en",
                "pl",
                &format!("{agent} sees {patient}!"),
                &format!(
                    "{} widzi {}",
                    if agent_canonical == "Tom" {
                        "Tomek"
                    } else {
                        "Iza"
                    },
                    if patient_canonical == "Tom" {
                        "Tomka"
                    } else {
                        "Izę"
                    }
                ),
            ));
            english_cases += 1;
            results.push(assert_b1_translation(
                polish_cases,
                "pl",
                "en",
                &format!("{agent_pl} widzi {patient_pl_object}!"),
                &format!("{agent_canonical} sees {patient_canonical}"),
            ));
            polish_cases += 1;
        }
    }

    for (index, (person, _, object, canonical)) in people.iter().enumerate() {
        results.push(assert_b1_translation(
            english_cases,
            "en",
            "pl",
            &format!("Who sees {person}?"),
            &format!(
                "Kto widzi {}?",
                if *canonical == "Tom" { "Tomka" } else { "Izę" }
            ),
        ));
        english_cases += 1;
        results.push(assert_b1_translation(
            polish_cases,
            "pl",
            "en",
            &format!("Kto widzi {object}?"),
            &format!("Who sees {canonical}?"),
        ));
        polish_cases += 1;
        let _ = index;
    }

    for (index, (person, person_pl, _, canonical)) in people.iter().enumerate() {
        results.push(assert_b1_translation(
            english_cases,
            "en",
            "pl",
            &format!("{person} sees who?"),
            &format!(
                "{} widzi Kogo?",
                if *canonical == "Tom" { "Tomek" } else { "Iza" }
            ),
        ));
        english_cases += 1;
        results.push(assert_b1_translation(
            polish_cases,
            "pl",
            "en",
            &format!("{person_pl} widzi Kogo?"),
            &format!("{canonical} sees who?"),
        ));
        polish_cases += 1;
        let _ = index;
    }

    results.push(assert_b1_translation(
        english_cases,
        "en",
        "pl",
        "Paris is the capital of France!",
        "Paryż jest stolicą Francji",
    ));
    english_cases += 1;
    results.push(assert_b1_translation(
        english_cases,
        "en",
        "pl",
        "Warsaw is the capital of Poland!",
        "Warszawa jest stolicą Polski",
    ));
    english_cases += 1;
    results.push(assert_b1_translation(
        polish_cases,
        "pl",
        "en",
        "Paryż jest stolicą Francji!",
        "Paris is the capital of France",
    ));
    polish_cases += 1;
    results.push(assert_b1_translation(
        polish_cases,
        "pl",
        "en",
        "Warszawa jest stolicą Polski!",
        "Warsaw is the capital of Poland",
    ));
    polish_cases += 1;

    assert_eq!(english_cases, 50);
    assert_eq!(polish_cases, 50);
    command_support::write_level_report("b1", &results);
    command_support::assert_level_passed("b1", &results);
}

#[test]
fn fronted_object_questions_preserve_semantics_in_both_languages() {
    let people = [
        ("Tom", "Tomek"),
        ("Iza", "Iza"),
        ("Thomas", "Tomasz"),
        ("Tommy", "Tomcio"),
        ("Isabelle", "Izabela"),
        ("Izzy", "Izka"),
    ];
    let mut results = Vec::new();
    for (index, (english, polish)) in people.iter().enumerate() {
        results.push(assert_b1_translation(
            index,
            "en",
            "pl",
            &format!("Who does {english} see?"),
            &format!("{} widzi Kogo?", canonical_polish_subject(polish)),
        ));
        results.push(assert_b1_translation(
            index,
            "pl",
            "en",
            &format!("Kogo widzi {polish}?"),
            &format!("{} sees who?", canonical_english_subject(english)),
        ));
    }
    assert_eq!(results.len(), 12);
    command_support::write_level_report("fronted-questions", &results);
    command_support::assert_level_passed("fronted-questions", &results);
}

#[test]
fn coordination_and_preserves_canonical_order_in_both_languages() {
    let results = vec![
        assert_b1_translation(
            0,
            "en",
            "pl",
            "Tom sees Iza and Iza sees Tom.",
            "Iza widzi Tomka i Tomek widzi Izę",
        ),
        assert_b1_translation(
            0,
            "pl",
            "en",
            "Tomek widzi Izę i Iza widzi Tomka.",
            "Iza sees Tom and Tom sees Iza",
        ),
    ];
    command_support::write_level_report("coordination-and", &results);
    command_support::assert_level_passed("coordination-and", &results);
}

#[test]
fn coordination_or_preserves_canonical_order_in_both_languages() {
    let results = vec![
        assert_b1_translation(
            0,
            "en",
            "pl",
            "Tom sees Iza or Iza sees Tom.",
            "Iza widzi Tomka albo Tomek widzi Izę",
        ),
        assert_b1_translation(
            0,
            "pl",
            "en",
            "Tomek widzi Izę albo Iza widzi Tomka.",
            "Iza sees Tom or Tom sees Iza",
        ),
    ];
    command_support::write_level_report("coordination-or", &results);
    command_support::assert_level_passed("coordination-or", &results);
}

#[test]
fn negated_assertions_preserve_semantics_in_both_languages() {
    let results = vec![
        assert_b1_translation(0, "en", "pl", "Not Tom sees Iza.", "nie Tomek widzi Izę"),
        assert_b1_translation(0, "pl", "en", "Nie Tomek widzi Izę.", "not Tom sees Iza"),
    ];
    command_support::write_level_report("negation", &results);
    command_support::assert_level_passed("negation", &results);
}

#[test]
fn nested_boolean_scope_preserves_semantics_in_both_languages() {
    let results = vec![
        assert_b1_translation(
            0,
            "en",
            "pl",
            "Not Tom sees Iza or Iza sees Tom.",
            "Iza widzi Tomka albo nie Tomek widzi Izę",
        ),
        assert_b1_translation(
            1,
            "pl",
            "en",
            "Nie Tomek widzi Izę albo Iza widzi Tomka.",
            "Iza sees Tom or not Tom sees Iza",
        ),
        assert_b1_translation(
            2,
            "en",
            "pl",
            "Not Tom sees Iza and Iza sees Tom.",
            "Iza widzi Tomka i nie Tomek widzi Izę",
        ),
        assert_b1_translation(
            3,
            "pl",
            "en",
            "Nie Tomek widzi Izę i Iza widzi Tomka.",
            "Iza sees Tom and not Tom sees Iza",
        ),
    ];
    command_support::write_level_report("nested-boolean-scope", &results);
    command_support::assert_level_passed("nested-boolean-scope", &results);
}

fn canonical_english_subject(source: &str) -> &str {
    match source {
        "Thomas" | "Tommy" => "Tom",
        "Isabelle" | "Izzy" => "Iza",
        value => value,
    }
}

fn canonical_polish_subject(source: &str) -> &str {
    match source {
        "Tomasz" | "Tomcio" => "Tomek",
        "Izabela" | "Izka" => "Iza",
        value => value,
    }
}
