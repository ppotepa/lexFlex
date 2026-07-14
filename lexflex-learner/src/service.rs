use crate::types::{DeductionResult, Language, ConceptMatch, SemanticInfo, SurfaceInfo, ProposedConcept, ProposedConceptDef, LexiconProposal};
use crate::sources::{WordInfoSource, conceptnet::ConceptNetSource, dictionary_api::DictionaryApiSource, local_llm::LocalLlmSource, wiktionary::WiktionarySource};
use crate::error::Result;
use crate::scoring::{score_from_is_a, narrow_context_score};
use std::collections::HashSet;
use std::sync::OnceLock;

/// LocalKnowledgeSource — fully local, no network.
///
/// Loads shipped data/*.ron files (lexicons + concepts) that live in the repo.
/// This lets `lexlearn` work completely offline for any word that is already
/// covered by our lexicon (which includes all key words from Adam's text).
///
/// For words present in lexicon we return:
///   - surface features (gender, number, case, pos, lemma)
///   - is_a derived from the entry's concept + simple useful labels ("rok", "kobieta", "osoba"...)
///   - best concept will be derived later by the mapper from those is_a labels.
pub struct LocalKnowledgeSource;

fn load_concept_ids_local() -> Vec<String> {
    let candidates = [
        "data/concepts/concepts.ron",
        "../data/concepts/concepts.ron",
        "../../data/concepts/concepts.ron",
    ];
    for p in &candidates {
        if let Ok(content) = std::fs::read_to_string(p) {
            let ids: Vec<String> = content.lines().filter_map(|l| {
                if let Some(start) = l.find("id: \"") {
                    if let Some(end) = l[start+5..].find('"') { return Some(l[start+5..start+5+end].to_string()); }
                }
                None
            }).collect();
            if !ids.is_empty() { return ids; }
        }
    }
    vec!["PERSON".into(), "WIFE".into(), "DOG".into(), "YEAR".into(), "LIVE".into()]
}

/// Very small structs just to parse the parts we care about from the .ron files.
/// (We don't want to depend on the main lexflex crate types here.)
#[derive(Debug, Clone, serde::Deserialize)]
struct RawLexEntry {
    lemma: String,
    pos: String,
    concept: String,
    features: Option<RawFeatures>,
}

#[derive(Debug, Clone, serde::Deserialize, Default)]
struct RawFeatures {
    gender: Option<String>,
    number: Option<String>,
    case: Option<String>,
    animacy: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct RawConcept {
    id: String,
    #[serde(default)]
    parent: Option<String>,
}

impl LocalKnowledgeSource {
    fn find_lexicon_path(lang: Language) -> Option<String> {
        let base = match lang {
            Language::Pl => "lexicons/pl/lexicon.ron",
            Language::En => "lexicons/en/lexicon.ron",
        };
        // Robust paths for local dev, Docker (/app), installed locations etc.
        let candidates = [
            format!("data/{}", base),
            format!("./data/{}", base),
            format!("/app/data/{}", base),
            format!("/usr/local/share/lexflex/data/{}", base),
            format!("/usr/share/lexflex/data/{}", base),
            format!("../data/{}", base),
            format!("../../data/{}", base),
            // dev machine absolute (kept for convenience)
            format!("/home/ppotepa/git/lexFlex/data/{}", base),
        ];
        for c in &candidates {
            if std::path::Path::new(c).exists() {
                return Some(c.clone());
            }
        }
        None
    }

    fn find_concepts_path() -> Option<String> {
        let candidates = [
            "data/concepts/concepts.ron".to_string(),
            "./data/concepts/concepts.ron".to_string(),
            "/app/data/concepts/concepts.ron".to_string(),
            "/usr/local/share/lexflex/data/concepts/concepts.ron".to_string(),
            "../data/concepts/concepts.ron".to_string(),
            "../../data/concepts/concepts.ron".to_string(),
            "/home/ppotepa/git/lexFlex/data/concepts/concepts.ron".to_string(),
        ];
        for c in &candidates {
            if std::path::Path::new(c).exists() {
                return Some(c.clone());
            }
        }
        None
    }

    fn load_lexicon(path: &str) -> Vec<(String, RawLexEntry)> {
        if let Ok(content) = std::fs::read_to_string(path) {
            // Parse as raw values first (the file has extra fields like frame_type, roles, paradigm)
            if let Ok(raw) = ron::from_str::<Vec<(String, ron::Value)>>(&content) {
                let mut out = vec![];
                for (k, v) in raw {
                    if let ron::Value::Map(map) = v {
                        let mut e = RawLexEntry {
                            lemma: k.clone(),
                            pos: "Unknown".into(),
                            concept: "PERSON".into(),
                            features: None,
                        };
                        for (field, val) in map {
                            if let ron::Value::String(s) = field {
                                match s.as_str() {
                                    "lemma" => if let ron::Value::String(x) = val { e.lemma = x; }
                                    "pos" => if let ron::Value::String(x) = val { e.pos = x; }
                                    "concept" => if let ron::Value::String(x) = val { e.concept = x; }
                                    "features" => {
                                        // best effort for common feature keys
                                        if let ron::Value::Map(fmap) = val {
                                            let mut rf = RawFeatures::default();
                                            for (fk, fv) in fmap {
                                                if let ron::Value::String(fs) = fk {
                                                    if let ron::Value::String(vs) = fv {
                                                        match fs.as_str() {
                                                            "gender" => rf.gender = Some(vs),
                                                            "number" => rf.number = Some(vs),
                                                            "case" => rf.case = Some(vs),
                                                            "animacy" => rf.animacy = Some(vs),
                                                            _ => {}
                                                        }
                                                    }
                                                }
                                            }
                                            e.features = Some(rf);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        out.push((k, e));
                    }
                }
                return out;
            }
        }
        vec![]
    }

    fn load_concepts(path: &str) -> Vec<RawConcept> {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(concepts) = ron::from_str::<Vec<RawConcept>>(&content) {
                return concepts;
            }
        }
        vec![]
    }

    /// Minimal data-driven lemmatizer for Polish (adj + common noun patterns).
    /// Tries to turn an inflected surface back to a lemma that exists in the loaded lexicon entries.
    /// Pure lookup, no word-specific logic.
    fn recover_pl_lemma(form: &str, entries: &[(String, RawLexEntry)]) -> Option<String> {
        let f = form.to_lowercase();

        // Generic PL inflected form recovery for adjectives (y-paradigm is common) and nouns.
        // Longer suffixes first to avoid bad splits (e.g. "imi" before "i").
        // Data-driven: only returns a lemma that actually exists in the loaded lexicon.
        let common_strips = [
            "iego", "iej", "ymi", "imi", "ych", "ich", "ego", "emu", "ej", "ym", "im",
            "ie", "y", "i", "ą", "om", "ami", "ach", "a", "u", "o", "e", "em",
        ];

        for suf in &common_strips {
            if f.ends_with(suf) && f.len() > suf.len() + 1 {
                let stem = &f[..f.len() - suf.len()];
                // Try common adjective/noun lemma endings
                for end in ["", "y", "i", "a", "e", "o", "ego", "ka", "ki"] {
                    let cand = format!("{}{}", stem, end);
                    if entries.iter().any(|(_, e)| e.lemma.to_lowercase() == cand) {
                        return Some(cand);
                    }
                }
            }
        }

        // Last resort exact lemma match
        if entries.iter().any(|(_, e)| e.lemma.to_lowercase() == f) {
            return Some(f);
        }

        None
    }
}

#[async_trait::async_trait]
impl WordInfoSource for LocalKnowledgeSource {
    fn name(&self) -> &'static str { "LocalKnowledge" }

    async fn fetch(&self, word: &str, lang: Language) -> Result<(SurfaceInfo, SemanticInfo)> {
        let w = word.to_lowercase();
        let mut surface = SurfaceInfo {
            lemma: Some(word.to_string()),
            ..Default::default()
        };
        let mut semantics = SemanticInfo::default();
        let mut had_lexicon_entry = false;

        // 1. Try to load the local lexicon for this language
        if let Some(lex_path) = Self::find_lexicon_path(lang) {
            let entries = Self::load_lexicon(&lex_path);
            // lookup by form or by lemma
            let mut candidate = w.clone();
            let mut entry = entries.iter().find(|(k, _)| k.to_lowercase() == candidate)
                .or_else(|| {
                    entries.iter().find(|(_, e)| e.lemma.to_lowercase() == candidate)
                })
                .map(|(_, e)| e);

            // Generic offline recovery for PL inflected forms (adj/noun endings) to hit base lemma in lexicon for is_a.
            // Purely data-driven (lookup in loaded entries); lang-specific only because morphology is.
            if entry.is_none() && lang == Language::Pl {
                if let Some(recovered) = Self::recover_pl_lemma(&w, &entries) {
                    candidate = recovered;
                    entry = entries.iter().find(|(_, e)| e.lemma.to_lowercase() == candidate)
                        .map(|(_, e)| e);
                }
            }

            if let Some(e) = entry {
                had_lexicon_entry = true;
                surface.pos = Some(e.pos.clone());
                if let Some(f) = &e.features {
                    surface.gender = f.gender.clone();
                    surface.number = f.number.clone();
                    surface.case = f.case.clone();
                    surface.animacy = f.animacy.clone();
                }
                // Use the recovered lemma for surface when we did morph recovery
                if candidate != w {
                    surface.lemma = Some(candidate.clone());
                }
                // Turn the concept into IsA label (data-driven; cross-lang via ontology parents later)
                semantics.is_a.push(e.concept.to_lowercase());
                semantics.definitions.push(format!("local-lexicon: {}", e.concept));
            }
        }

        // 2. Optionally enrich with ontology parents as IsA (extra data, still local)
        if let Some(con_path) = Self::find_concepts_path() {
            let concepts = Self::load_concepts(&con_path);
            for c in &concepts {
                if c.id.to_uppercase() == semantics.is_a.first().unwrap_or(&String::new()).to_uppercase() {
                    if let Some(p) = &c.parent {
                        semantics.is_a.push(p.to_lowercase());
                    }
                }
            }
        }

        // Dedup
        semantics.is_a.sort();
        semantics.is_a.dedup();

        // For unknown (no lexicon entry) we do not synthesize is_a here.
        // infer_concept returns None for unmapped; proposals handled in build for novel.
        if !had_lexicon_entry && semantics.is_a.is_empty() && semantics.definitions.is_empty() {
            surface.lemma = None;
        }
        Ok((surface, semantics))
    }
}

pub struct LexicalDeductionService {
    sources: Vec<Box<dyn WordInfoSource>>,
}

/// Global cached service so we don't recreate HTTP clients / LLM sources on every word.
/// This is especially important for interactive use (chat.sh) and repeated deduction.
static GLOBAL_SERVICE: OnceLock<LexicalDeductionService> = OnceLock::new();

impl LexicalDeductionService {
    /// Returns a process-wide cached instance (preferred for chat / repeated use).
    pub fn global() -> &'static Self {
        GLOBAL_SERVICE.get_or_init(|| Self::new())
    }

    pub fn new() -> Self {
        let mut sources: Vec<Box<dyn WordInfoSource>> = vec![];

        // 1. LLM first (higher priority for difficult inflected words)
        // Configure via:
        //   LEXFLEX_LLM_BASE_URL=http://host.docker.internal:1234/v1
        //   LEXFLEX_LLM_MODEL=google/gemma-4-e2b
        if let Some(llm) = LocalLlmSource::from_env() {
            sources.push(Box::new(llm));
        }

        // 2. Then the original lexical sources (LocalKnowledge + APIs)
        sources.push(Box::new(LocalKnowledgeSource));
        sources.push(Box::new(ConceptNetSource::new()));
        sources.push(Box::new(DictionaryApiSource::new()));
        sources.push(Box::new(WiktionarySource::new()));

        Self { sources }
    }

    /// Purely local mode — no HTTP sources at all.
    /// Uses only LocalKnowledge that reads the repo's data/lexicons and data/concepts.
    /// Local LLM is also excluded in this mode.
    pub fn new_offline() -> Self {
        let sources: Vec<Box<dyn WordInfoSource>> = vec![
            Box::new(LocalKnowledgeSource),
        ];
        Self { sources }
    }

    pub async fn deduce(
        &self,
        word: &str,
        lang: Language,
        context: Option<&str>,
    ) -> Result<DeductionResult> {
        let mut combined_surface = SurfaceInfo::default();
        let mut combined_semantics = SemanticInfo::default();
        let mut sources_used = vec![];

        for source in &self.sources {
            match source.fetch(word, lang).await {
                Ok((surf, sem)) => {
                    // Only record sources that actually contributed data (no fake "used" for failed/empty fetches)
                    let contributed = surf.lemma.is_some()
                        || surf.pos.is_some()
                        || !surf.example_sentences.is_empty()
                        || !sem.definitions.is_empty()
                        || !sem.is_a.is_empty()
                        || !sem.related.is_empty()
                        || !sem.synonyms.is_empty();
                    if contributed {
                        sources_used.push(source.name().to_string());
                    }

                    // Merge surface (simple take first non-empty)
                    if combined_surface.lemma.is_none() {
                        combined_surface.lemma = surf.lemma;
                    }
                    if combined_surface.pos.is_none() {
                        combined_surface.pos = surf.pos;
                    }
                    combined_surface.example_sentences.extend(surf.example_sentences);

                    // Merge semantics - especially IsA is gold
                    combined_semantics.definitions.extend(sem.definitions);
                    combined_semantics.is_a.extend(sem.is_a);
                    combined_semantics.synonyms.extend(sem.synonyms);
                    combined_semantics.related.extend(sem.related);
                    combined_semantics.hypernyms.extend(sem.hypernyms);
                }
                Err(_) => continue,
            }
        }

        // Deduplicate
        combined_semantics.is_a = combined_semantics.is_a.into_iter().collect::<HashSet<_>>().into_iter().collect();
        combined_semantics.related = combined_semantics.related.into_iter().collect::<HashSet<_>>().into_iter().collect();

        let best_concept = self.infer_concept(&combined_semantics, word, context, lang);

        // === Phase 1/2: attempt to produce a main Interlingua concept proposal (new or strong) + bilingual seeds ===
        let (concept_proposal, lexicon_proposals) =
            self.build_concept_and_lexicon_proposals(&combined_semantics, word, lang, &combined_surface, &best_concept);

        let confidence = concept_proposal.as_ref().map(|c| c.confidence)
            .or_else(|| best_concept.as_ref().map(|c| c.confidence))
            .unwrap_or(0.3);

        Ok(DeductionResult {
            word: word.to_string(),
            lang,
            surface: combined_surface,
            semantics: combined_semantics,
            best_concept,
            confidence,
            sources_used,
            concept_proposal,
            lexicon_proposals,
        })
    }

    fn infer_concept(
        &self,
        semantics: &SemanticInfo,
        word: &str,
        context: Option<&str>,
        _lang: Language,
    ) -> Option<ConceptMatch> {
        let mut scores: Vec<(String, f32, String)> = vec![];

        let is_a_lower: Vec<String> = semantics.is_a.iter().map(|s| s.to_lowercase()).collect();

        // Scoped context window (±5 tokens) purely for optional disambiguation / role hints.
        // No global sentence bleed into concept choice.
        let context_lower = if let Some(ctx) = context {
            let ctx_l = ctx.to_lowercase();
            let tokens: Vec<&str> = ctx_l.split_whitespace().collect();
            if let Some(pos) = tokens.iter().position(|t| *t == word.to_lowercase()) {
                let start = pos.saturating_sub(5);
                let end = (pos + 6).min(tokens.len());
                tokens[start..end].join(" ")
            } else {
                ctx_l
            }
        } else {
            String::new()
        };

        // Primary: derive directly from concepts.ron list.
        // If an is_a label (from Local lexicon concept or parents or external) uppercases to a known ConceptId, use it.
        // Learner works on language-agnostic concepts; lexicon provides the surface->concept mapping.
        let known_concepts = load_concept_ids_local();
        for isa in &is_a_lower {
            if let Some((cid, conf, reason)) = score_from_is_a(&[isa.clone()], &known_concepts) {
                scores.push((cid, conf, reason));
                continue;
            }
        }

        // Use extracted narrow context score for age (the only kept exception per AC3/strategy).
        if scores.is_empty() {
            if let Some((cid, conf, reason)) = narrow_context_score(&word.to_lowercase(), &context_lower) {
                scores.push((cid, conf, reason));
            }
        }

        if scores.is_empty() {
            // Per AC3 pure exact: no score -> None (even for zero is_a; conservative handled in build or caller)
            return None;
        }

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let (concept, conf, reason) = scores.into_iter().next().unwrap();

        Some(ConceptMatch {
            concept_id: concept,
            confidence: conf,
            reason,
            matched_relations: semantics.is_a.clone(),
        })
    }

    // scoring is now purely exact in scoring.rs (AC3 + strategy)

    fn features_to_ron(&self, surface: &SurfaceInfo) -> String {
        let mut parts = vec![];
        if let Some(g) = &surface.gender {
            parts.push(format!("gender: Some({})", g));
        }
        if let Some(n) = &surface.number {
            parts.push(format!("number: Some({})", n));
        }
        if let Some(c) = &surface.case {
            parts.push(format!("case: Some({})", c));
        }
        if parts.is_empty() {
            "number: Some(Singular)".to_string()
        } else {
            parts.join(", ")
        }
    }

    /// Core of the new deducer: given collected semantics, produce (if warranted)
    /// a Proposed main IL concept (new or primary) + lexicon proposals for EN and PL.
    ///
    /// This implements the "create a main Interlingua concept and related words in eng or pol"
    /// part of the goal. Conservative: only proposes new concepts on sufficient evidence.
    fn build_concept_and_lexicon_proposals(
        &self,
        semantics: &SemanticInfo,
        word: &str,
        lang: Language,
        surface: &SurfaceInfo,
        best: &Option<ConceptMatch>,
    ) -> (Option<ProposedConcept>, Vec<LexiconProposal>) {
        let mut lexicon_proposals: Vec<LexiconProposal> = vec![];
        let mut concept_proposal: Option<ProposedConcept> = None;

        let lemma = surface.lemma.as_deref().unwrap_or(word);
        let pos = surface.pos.as_deref().unwrap_or("Unknown");
        let features = self.features_to_ron(surface);

        // === Attempt to decide / create the main Interlingua concept ===
        // For novel words (!best or conservative PERSON), derive new concept id + proposal.
        // Use best id when a real match from scoring. Ensures primary lex proposal is correct.
        //
        // STRENGTHENED (per goal): ALWAYS prefer algorithmic match from LLM's is_a (or any semantics.is_a)
        // to existing concepts in concepts.ron BEFORE creating any novel generic proposal.
        // Only create new if no match at all + high-evidence case. This stops junk like CZARNY/FLUBERCZYK
        // when LLM gives "red" etc.

        let is_a_lower: Vec<String> = semantics.is_a.iter().map(|s| s.to_lowercase()).collect();
        let known_concepts = load_concept_ids_local();
        let mut matched_existing: Option<String> = None;
        for isa in &is_a_lower {
            if let Some((cid, _, _)) = score_from_is_a(&[isa.clone()], &known_concepts) {
                matched_existing = Some(cid);
                break;
            }
        }

        let lower = word.to_lowercase();
        let all_text = format!(
            "{} {} {}",
            semantics.is_a.join(" "),
            semantics.definitions.join(" "),
            semantics.related.join(" ")
        ).to_lowercase();
        let proposed_id = self.derive_concept_id(word, &semantics.is_a);
        // Only create novel if we have some evidence (is_a or best) AND no match; empty is_a -> no novel generic
        let has_evidence = !is_a_lower.is_empty() || best.is_some();
        let use_derived = has_evidence && matched_existing.is_none() && (best.is_none() || best.as_ref().map(|b| b.concept_id == "PERSON").unwrap_or(false));

        let primary_concept_id: String;
        let encountered_ron: String;

        if use_derived {
            let (roles, frame, features_ron) = self.infer_definition_shape(&all_text, &lower);
            let def = ProposedConceptDef {
                id: proposed_id.clone(),
                frame_type: frame,
                roles,
                inherent_features_ron: features_ron,
            };
            concept_proposal = Some(ProposedConcept {
                concept_id: proposed_id.clone(),
                definition: def,
                confidence: 0.65,
                reason: format!("novel from relations for new word: {}", word),
                parent_suggestion: self.suggest_parent(&all_text),
                matched_relations: semantics.is_a.clone(),
            });
            primary_concept_id = proposed_id.clone();
            encountered_ron = format!(
                r#"("{}" , (lemma: "{}", pos: "{}", concept: "{}", frame_type: None, roles: [], paradigm: None, features: ({})))"#,
                word, lemma, pos, proposed_id, features
            );
        } else if let Some(cid) = matched_existing {
            // prefer the algorithmically matched existing from is_a (even if best was none)
            primary_concept_id = cid.clone();
            encountered_ron = format!(
                r#"("{}" , (lemma: "{}", pos: "{}", concept: "{}", frame_type: None, roles: [], paradigm: None, features: ({})))"#,
                word, lemma, pos, primary_concept_id, features
            );
        } else if let Some(b) = best.as_ref() {
            primary_concept_id = b.concept_id.clone();
            encountered_ron = format!(
                r#"("{}" , (lemma: "{}", pos: "{}", concept: "{}", frame_type: None, roles: [], paradigm: None, features: ({})))"#,
                word, lemma, pos, primary_concept_id, features
            );
        } else {
            // no evidence, no best -> fallback, no proposal
            primary_concept_id = "UNKNOWN".to_string();
            encountered_ron = format!(
                r#"("{}" , (lemma: "{}", pos: "{}", concept: "UNKNOWN", frame_type: None, roles: [], paradigm: None, features: ({})))"#,
                word, lemma, pos, features
            );
        }

        // Seed the primary lexicon proposal for the input language (AFTER decision, using correct id)
        lexicon_proposals.push(LexiconProposal {
            lang,
            key: word.to_string(),
            ron_line: encountered_ron.clone(),
            pos: Some(pos.to_string()),
            concept_id: primary_concept_id.clone(),
        });

        // === Bilingual seeds (EN/PL related words) — Phase 2 foundation ===
        // For the primary concept (existing or the one we just proposed), emit a companion entry
        // in the *other* language using best-effort lemma (often same surface or simple transform).
        let companion_lang = match lang {
            Language::Pl => Language::En,
            Language::En => Language::Pl,
        };
        let companion_lemma = self.best_companion_lemma(lemma, lang);
        let companion_concept = concept_proposal.as_ref().map(|p| p.concept_id.clone())
            .or_else(|| best.as_ref().map(|b| b.concept_id.clone()))
            .unwrap_or_else(|| "PERSON".to_string());

        let companion_ron = format!(
            r#"("{}" , (lemma: "{}", pos: "{}", concept: "{}", frame_type: None, roles: [], paradigm: None, features: ({})))"#,
            companion_lemma, companion_lemma, pos, companion_concept, features
        );

        lexicon_proposals.push(LexiconProposal {
            lang: companion_lang,
            key: companion_lemma.clone(),
            ron_line: companion_ron,
            pos: Some(pos.to_string()),
            concept_id: companion_concept,
        });

        (concept_proposal, lexicon_proposals)
    }

    fn derive_concept_id(&self, word: &str, is_a: &[String]) -> String {
        // Prefer a clean label from the strongest is_a if present
        if let Some(first) = is_a.first() {
            let cleaned: String = first.chars()
                .map(|c| if c.is_alphanumeric() { c.to_ascii_uppercase() } else { '_' })
                .collect();
            let cleaned = cleaned.trim_matches('_').to_string();
            if !cleaned.is_empty() && cleaned.len() > 2 {
                return cleaned;
            }
        }
        // Fallback: word uppercased, sanitized
        let mut id: String = word.chars()
            .map(|c| if c.is_alphanumeric() { c.to_ascii_uppercase() } else { '_' })
            .collect();
        id = id.trim_matches('_').to_string();
        if id.is_empty() { "NEW_ENTITY".to_string() } else { id }
    }

    fn infer_definition_shape(&self, _all_text: &str, _word_lower: &str) -> (Vec<String>, Option<String>, String) {
        // Generic shape only; exact matching via score_from_is_a (AC3)
        let roles = vec!["Theme".to_string(), "Patient".to_string()];
        let feat = "countability: Some(Count)".to_string();
        (roles, None, feat)
    }

    fn suggest_parent(&self, _all_text: &str) -> Option<String> {
        // No hardcoded specific parents here; rely on data/concepts + is_a from sources
        None
    }

    fn best_companion_lemma(&self, lemma: &str, input_lang: Language) -> String {
        let l = lemma.to_lowercase();
        if input_lang == Language::Pl {
            // Provide proper English forms for common words so we don't pollute EN lexicon
            // with Polish surface forms (e.g. "czerwony" -> "red" not "czerwony").
            match l.as_str() {
                "czerwony" => return "red".to_string(),
                "zielony" => return "green".to_string(),
                "czarny" => return "black".to_string(),
                "biały" => return "white".to_string(),
                "niebieski" | "błękitny" => return "blue".to_string(),
                "mój" => return "my".to_string(),
                "twój" => return "your".to_string(),
                "nasz" => return "our".to_string(),
                "wasz" => return "your".to_string(),
                "ten" | "ta" | "to" => return "this".to_string(),
                "tamten" => return "that".to_string(),
                _ => {}
            }
        }
        // Generic: use as-is (real cross-lang should come from LLM evidence when possible).
        lemma.to_string()
    }
}

impl Default for LexicalDeductionService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_deduce_offline_exercises_shipped_service() {
        // Exercise the public shipped API (LexicalDeductionService) on real LocalKnowledge path.
        let service = LexicalDeductionService::new_offline();
        let result = service.deduce("żona", Language::Pl, Some("mieszkam z żoną")).await.unwrap();
        assert!(result.best_concept.is_some());
        let bc = result.best_concept.unwrap();
        assert_eq!(bc.concept_id, "WIFE");
        assert!(result.semantics.is_a.iter().any(|s| s.contains("wife") || s.contains("żon")));
        assert!(result.sources_used.iter().any(|s| s.contains("Local")));
    }

    #[tokio::test]
    async fn test_deduce_is_a_present_unmapped_returns_no_best() {
        // AC3: if is_a collected but unmapped to concept (e.g. 'together' for 'razem'), no best_concept (no ignore + fallback)
        let service = LexicalDeductionService::new_offline();
        let result = service.deduce("razem", Language::Pl, Some("mieszkam razem z żoną")).await.unwrap();
        assert!(result.best_concept.is_none(), "should not assign best when is_a present but unmapped");
        assert!(!result.semantics.is_a.is_empty());
        assert!(result.semantics.is_a.iter().any(|s| s.contains("together")));
    }

}
