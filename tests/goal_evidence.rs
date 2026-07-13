//! Goal evidence test per strategist: drive parse and write full Debug to scratch for verif3.

use std::env;
use std::fs;
use std::path::Path;

#[test]
fn test_capture_parse_unknowns() {
    use lexflex::api::LexFlexAPI;
    let api = LexFlexAPI::builder().data_dir("data").build().expect("api");
    let scratch = env::var("GROK_GOAL_SCRATCH").unwrap_or_else(|_| "/tmp/grok-goal-20f0a7a2ae76/implementer".to_string());
    let scr_path = Path::new(&scratch);
    let _ = fs::create_dir_all(scr_path);

    let mut content = String::new();
    for (sent, lang) in [
        ("Mieszkam z blargxyz.", "pl"),
        ("I saw blargxyz.", "en"),
        ("Widzę xyzqwe.", "pl"),
    ] {
        let utt = api.parse(sent, lang).expect("parse");
        content.push_str(&format!("=== {} ===\n", sent));
        content.push_str(&format!("{:#?}\n", utt));
    }
    fs::write(scr_path.join("parse_unknowns.txt"), content).expect("write");
    // also assert basics like in integration
    let utt = api.parse("Mieszkam z xyzqwe.", "pl").expect("parse");
    let natural = utt.as_natural().expect("natural");
    assert!(natural.sentences.iter().any(|s| s.frames.iter().any(|f| f.entities().iter().any(|e| e.name.as_deref() == Some("xyzqwe")))));
}
