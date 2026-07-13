//! Deterministic lexicon lookup — guards against GO/WALK ambiguity for Polish "iść".

use lexflex::data::loader;
use std::path::Path;

fn pl_lexicon() -> lexflex::data::lexicon::Lexicon {
    let data = Path::new("data");
    loader::load_lexicon(&data.join("lexicons/pl/lexicon.ron")).expect("PL lexicon")
}

#[test]
fn test_lookup_by_form_poszedl_is_go() {
    let lex = pl_lexicon();
    let entry = lex.lookup_by_form("poszedł").expect("poszedł must be in lexicon");
    assert_eq!(entry.concept, "GO");
    assert_eq!(entry.frame_type.as_deref(), Some("Motion"));
    assert!(entry.roles.iter().any(|r| r == "Goal"));
}

#[test]
fn test_lookup_by_lemma_isc_prefers_go_with_goal_role() {
    let lex = pl_lexicon();
    let entry = lex.lookup_by_lemma("iść").expect("iść must be in lexicon");
    assert_eq!(entry.concept, "GO", "lemma pool must pick GO over WALK");
    assert!(
        entry.roles.iter().any(|r| r == "Goal"),
        "selected iść entry must include Goal role, got {:?}",
        entry.roles
    );
}

#[test]
fn test_lookup_concept_store_deterministic_in_pl_lexicon() {
    let lex = pl_lexicon();
    let entry = lex.lookup_concept("STORE").expect("STORE concept must exist");
    assert_eq!(entry.lemma, "sklep");
}

#[test]
fn test_lookup_concept_store_deterministic_in_en_lexicon() {
    let data = Path::new("data");
    let lex = loader::load_lexicon(&data.join("lexicons/en/lexicon.ron")).expect("EN lexicon");
    let entry = lex.lookup_concept("STORE").expect("STORE concept must exist");
    assert_eq!(entry.lemma, "store");
}