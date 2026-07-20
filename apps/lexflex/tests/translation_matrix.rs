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
    ];

    for (index, (source, target, input, expected)) in cases.iter().enumerate() {
        let output = command_support::run(&[
            "text-translate-text",
            "--language",
            *source,
            "--target-language",
            *target,
            "--text",
            *input,
        ]);
        assert_eq!(output.code, Some(0), "translation {index} failed");
        let payload: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("translation JSON");
        assert_eq!(payload["text"], *expected, "translation {index} mismatch");

        assert_eq!(
            payload["source_semantic_hash"], payload["target_semantic_hash"],
            "semantic round-trip {index} mismatch"
        );
    }
}
