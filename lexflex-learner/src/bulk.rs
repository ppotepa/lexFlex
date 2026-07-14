use std::fs;
use std::collections::HashSet;
use tokio::time::sleep;
use std::time::Duration;
use crate::{LexicalDeductionService, Language, DeductionResult};

/// Run bulk deduction on input file, save per-item JSON/RON/graph_hint, and summary.json
pub async fn run_bulk(input: &str, lang: Language, output_dir: &str, max: usize, delay_ms: u64, offline: bool) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(input)?;
    let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty() && !l.starts_with('#')).collect();

    let process_lines = if max > 0 { &lines[..max.min(lines.len())] } else { &lines[..] };

    println!("📁 Bulk processing {} lines from {}", process_lines.len(), input);
    println!("   Language: {}", lang);
    println!("   Output dir: {}", output_dir);
    println!("   Delay: {}ms between items", delay_ms);
    if offline {
        println!("   Mode: OFFLINE (LocalKnowledge only — no network)");
    }

    fs::create_dir_all(output_dir)?;

    let service = if offline {
        LexicalDeductionService::new_offline()
    } else {
        LexicalDeductionService::new()
    };
    let mut all_results: Vec<DeductionResult> = vec![];
    let mut processed_words = HashSet::new();

    for (i, line) in process_lines.iter().enumerate() {
        let sentence = if let Some(rest) = line.strip_prefix("PL->EN:") {
            rest.trim()
        } else if let Some(rest) = line.strip_prefix("PL->EN ") {
            rest.trim()
        } else {
            line.trim()
        };

        if sentence.is_empty() { continue; }

        println!("\n[{}/{}] Processing: {}", i+1, process_lines.len(), sentence.chars().take(60).collect::<String>());

        let words: Vec<String> = sentence
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() > 2)
            .map(|w| w.to_lowercase())
            .collect();

        let unique_words: Vec<_> = words.into_iter().collect::<HashSet<_>>().into_iter().collect();

        for word in unique_words {
            if processed_words.contains(&word) { continue; }
            processed_words.insert(word.clone());

            let scoped_ctx = extract_word_context(sentence, &word, 5);
            match service.deduce(&word, lang, Some(&scoped_ctx)).await {
                Ok(result) => {
                    all_results.push(result.clone());

                    let safe_word = word.replace(|c: char| !c.is_alphanumeric(), "_");
                    let out_path = format!("{}/{}_{}.json", output_dir, i+1, safe_word);
                    let json = serde_json::to_string_pretty(&result)?;
                    fs::write(&out_path, json)?;
                    println!("   ✓ {} -> concept: {:?} (conf {:.2})  [saved: {}]", 
                             word, 
                             result.best_concept.as_ref().map(|c| &c.concept_id), 
                             result.confidence,
                             out_path);

                    // Write rich proposals (new deducer goal)
                    // 1. Concept proposal (new main IL concept) if present
                    if let Some(prop) = &result.concept_proposal {
                        let concept_path = format!("{}/{}_{}.concept.ron", output_dir, i+1, safe_word);
                        let concept_ron = format!(
                            r#"// Proposed new main Interlingua concept (from deducer for "{}")
(id: "{}", frame_type: {}, roles: {:?}, inherent_features: ({})),
"#,
                            word,
                            prop.concept_id,
                            prop.definition.frame_type.as_deref().map(|s| format!("Some(\"{}\")", s)).unwrap_or("None".to_string()),
                            prop.definition.roles,
                            prop.definition.inherent_features_ron
                        );
                        let _ = fs::write(&concept_path, concept_ron);
                        println!("      + concept proposal -> {}", concept_path);
                    }

                    // 2. Per-language lexicon proposals (bilingual)
                    for lp in &result.lexicon_proposals {
                        let lang_tag = match lp.lang {
                            crate::Language::Pl => "pl",
                            crate::Language::En => "en",
                        };
                        let lp_path = format!("{}/{}_{}.{}.lex.ron", output_dir, i+1, safe_word, lang_tag);
                        let _ = fs::write(&lp_path, &lp.ron_line);
                    }

                    // 3. Enhanced graph hint with new deducer data
                    let graph_hint = serde_json::json!({
                        "word": word,
                        "evokes": result.best_concept.as_ref().map(|c| &c.concept_id),
                        "concept_proposal": result.concept_proposal.as_ref().map(|p| &p.concept_id),
                        "confidence": result.confidence,
                        "is_a": result.semantics.is_a,
                        "related": result.semantics.related,
                        "context_window": scoped_ctx,
                        "surface": result.surface,
                        "sources": result.sources_used,
                        "suggested_concept_edge": result.best_concept.as_ref().map(|c| format!("EvokesConcept -> {}", c.concept_id)),
                        "lexicon_proposals": result.lexicon_proposals.iter().map(|lp| format!("{}:{}", match lp.lang { crate::Language::Pl=>"pl", crate::Language::En=>"en" }, lp.key)).collect::<Vec<_>>(),
                    });
                    let gh_path = format!("{}/{}_{}.graph_hint.json", output_dir, i+1, safe_word);
                    let _ = fs::write(gh_path, serde_json::to_string_pretty(&graph_hint).unwrap());
                }
                Err(e) => {
                    eprintln!("   ✗ {} error: {}", word, e);
                }
            }

            if delay_ms > 0 {
                sleep(Duration::from_millis(delay_ms)).await;
            }
        }
    }

    // Save summary
    let with_concept = all_results.iter().filter(|r| r.best_concept.is_some()).count();
    let with_real_evidence = all_results.iter().filter(|r| {
        if let Some(c) = &r.best_concept {
            let reason = c.reason.to_lowercase();
            !r.semantics.is_a.is_empty() &&
            (reason.starts_with("direct from concepts") || reason.contains("isa/relation") || reason.contains("age context"))
        } else {
            false
        }
    }).count();
    let unique_concepts: HashSet<_> = all_results.iter()
        .filter_map(|r| r.best_concept.as_ref().map(|c| c.concept_id.clone()))
        .collect();

    // New deducer metrics
    let with_concept_proposal = all_results.iter().filter(|r| r.concept_proposal.is_some()).count();
    let total_lexicon_proposals: usize = all_results.iter().map(|r| r.lexicon_proposals.len()).sum();
    let total_is_a_facts: usize = all_results.iter().map(|r| r.semantics.is_a.len()).sum();

    let summary_path = format!("{}/bulk_summary.json", output_dir);
    let summary = serde_json::json!({
        "input_file": input,
        "lang": lang.to_string(),
        "processed_lines": process_lines.len(),
        "unique_words_attempted": processed_words.len(),
        "results": all_results.len(),
        "results_with_concept": with_concept,
        "results_with_real_evidence": with_real_evidence,
        "unique_concepts_inferred": unique_concepts.len(),
        "unique_concepts_list": unique_concepts,
        "results_with_new_concept_proposal": with_concept_proposal,
        "total_lexicon_proposals_emitted": total_lexicon_proposals,
        "total_is_a_facts_collected": total_is_a_facts,
        "note": "Evidence = IsA/relations from ConceptNet/LocalKnowledge/Dict/Wikt (no synthetic seeds). New: concept proposals + bilingual lexicon proposals."
    });
    fs::write(&summary_path, serde_json::to_string_pretty(&summary)?)?;

    println!("\n✅ Bulk complete!");
    println!("   Summary saved to: {}", summary_path);
    println!("   Individual JSONs + RONs + .graph_hint.json in: {}", output_dir);
    println!("   Total unique words processed: {}", processed_words.len());
    println!("   Results with inferred concept: {}", with_concept);
    println!("   Results backed by real IsA/evidence: {}", with_real_evidence);
    println!("   Unique concepts: {} -> {:?}", unique_concepts.len(), unique_concepts);

    Ok(())
}

pub fn extract_word_context(sentence: &str, word: &str, window: usize) -> String {
    let lower = sentence.to_lowercase();
    let wlower = word.to_lowercase();
    let tokens: Vec<&str> = lower.split_whitespace().collect();
    if let Some(pos) = tokens.iter().position(|t| *t == wlower) {
        let start = pos.saturating_sub(window);
        let end = (pos + window + 1).min(tokens.len());
        tokens[start..end].join(" ")
    } else {
        sentence.to_string()
    }
}

pub fn summarize_results(all_results: &[DeductionResult], processed_lines: usize, input: &str, lang: &Language, processed_words_len: usize) -> serde_json::Value {
    let with_concept = all_results.iter().filter(|r| r.best_concept.is_some()).count();
    let with_real_evidence = all_results.iter().filter(|r| {
        if let Some(c) = &r.best_concept {
            let reason = c.reason.to_lowercase();
            !r.semantics.is_a.is_empty() &&
            (reason.starts_with("direct from concepts") || reason.contains("isa/relation") || reason.contains("age context"))
        } else {
            false
        }
    }).count();
    let unique_concepts: HashSet<_> = all_results.iter()
        .filter_map(|r| r.best_concept.as_ref().map(|c| c.concept_id.clone()))
        .collect();
    let total_is_a_facts: usize = all_results.iter().map(|r| r.semantics.is_a.len()).sum();

    serde_json::json!({
        "input_file": input,
        "lang": lang.to_string(),
        "processed_lines": processed_lines,
        "unique_words_attempted": processed_words_len,
        "results": all_results.len(),
        "results_with_concept": with_concept,
        "results_with_real_evidence": with_real_evidence,
        "unique_concepts_inferred": unique_concepts.len(),
        "unique_concepts_list": unique_concepts,
        "total_is_a_facts_collected": total_is_a_facts,
        "note": "Evidence = IsA/relations from ConceptNet/LocalKnowledge/Dict/Wikt (no synthetic seeds)"
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Language;

    #[tokio::test]
    async fn test_bulk_adam_25_offline_ac4() {
        let manifest = env!("CARGO_MANIFEST_DIR");
        let input = format!("{}/../benchmarks/input_adam.txt", manifest);
        let lang = Language::Pl;
        let out_dir = format!("/tmp/grok_bulk_test_{}", std::process::id());
        let _ = std::fs::remove_dir_all(&out_dir);
        run_bulk(&input, lang, &out_dir, 25, 0, true).await.expect("bulk should succeed");
        let summary_path = format!("{}/bulk_summary.json", out_dir);
        let summary: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&summary_path).expect("read summary")).expect("parse json");
        assert_eq!(summary["processed_lines"], 25);
        let uniq = summary["unique_concepts_inferred"].as_u64().unwrap_or(0);
        assert!(uniq >= 14, "AC4: unique_concepts_inferred >=14, got {}", uniq);
        let real_ev = summary["results_with_real_evidence"].as_u64().unwrap_or(0);
        assert!(real_ev >= 14, "AC4: results_with_real_evidence >=14, got {}", real_ev);
        let prop_count = summary.get("results_with_new_concept_proposal").and_then(|v| v.as_u64()).unwrap_or(0);
        assert!(prop_count > 0, "AC4: results_with_new_concept_proposal >0 for novel, got {}", prop_count);
        // durable case: razem should have is_a and no best_concept (unmapped)
        let mut found_razem = false;
        if let Ok(rd) = std::fs::read_dir(&out_dir) {
            for e in rd.flatten() {
                let name = e.file_name().to_string_lossy().to_string();
                if name.contains("razem") && name.ends_with(".json") && !name.contains("graph_hint") {
                    let j: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(e.path()).unwrap()).unwrap();
                    let isa = j["semantics"]["is_a"].as_array().map(|a| !a.is_empty()).unwrap_or(false);
                    let no_best = j.get("best_concept").map_or(true, |v| v.is_null());
                    if isa && no_best {
                        found_razem = true;
                    }
                }
            }
        }
        assert!(found_razem, "should have durable razem case with is_a and no best");
        // check for .concept.ron from novel proposals (durable in out_dir before rm)
        let has_concept_ron = std::fs::read_dir(&out_dir).map(|rd| {
            rd.filter_map(|e| e.ok()).any(|e| e.file_name().to_string_lossy().ends_with(".concept.ron"))
        }).unwrap_or(false);
        assert!(has_concept_ron, "should emit .concept.ron for novel concept proposals");
        if let Ok(scr) = std::env::var("GROK_GOAL_SCRATCH") {
            crate::evidence::export_bulk_evidence(std::path::Path::new(&out_dir), std::path::Path::new(&scr));
        } else {
            let _ = std::fs::remove_dir_all(&out_dir);
        }
    }
}
