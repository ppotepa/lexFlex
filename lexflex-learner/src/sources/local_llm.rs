//! Optional Local LLM source for lexflex-learner.
//!
//! Talks to any OpenAI-compatible local server (Ollama, llama.cpp, LM Studio, etc.)
//! over the local network / VPN.
//!
//! Enable by setting:
//!   LEXFLEX_LLM_BASE_URL=http://host.docker.internal:1234/v1
//!   LEXFLEX_LLM_MODEL=google/gemma-4-e2b   (or whatever you loaded)
//!
//! System prompt control:
//!   By default we send a long meta-prompt describing the full lexFlex data model.
//!   To use *nothing* (or /nothink mode):
//!     export LEXFLEX_LLM_NOTHINK=1
//!     export LEXFLEX_LLM_SYSTEM_PROMPT=nothing
//!   This makes the model see almost no system instructions (raw-ish behavior).

//! It is added AFTER LocalKnowledgeSource so that real lexicon data always wins.

use crate::sources::WordInfoSource;
use crate::types::{Language, SemanticInfo, SurfaceInfo};
use crate::error::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct LocalLlmSource {
    client: Client,
    base_url: String,
    model: String,
    /// Used so we do a cheap warmup request exactly once (model cold start is expensive).
    warmed: Arc<AtomicBool>,
}

impl LocalLlmSource {
    pub fn from_env() -> Option<Self> {
        let base_url = env::var("LEXFLEX_LLM_BASE_URL").ok()?;
        let model = env::var("LEXFLEX_LLM_MODEL").unwrap_or_else(|_| "gemma2:2b".to_string());

        if base_url.trim().is_empty() {
            return None;
        }

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .build()
            .ok()?;

        Some(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            model,
            warmed: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        let client = Client::builder()
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            base_url: base_url.into().trim_end_matches('/').to_string(),
            model: model.into(),
            warmed: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Cheap "warmup" request. Local inference engines (llama.cpp, LM Studio, etc.)
    /// often have high latency on the very first request because the model needs
    /// to be loaded / weights paged / CUDA context created.
    /// We do a tiny completion (1 token) so the real work later is fast.
    async fn warmup(&self) -> Result<()> {
        let body = json!({
            "model": self.model,
            "messages": [ {"role": "user", "content": "warmup"} ],
            "max_tokens": 1,
            "temperature": 0.0,
            "stream": false
            // deliberately no response_format — faster for pure warmup
        });

        let url = format!("{}/chat/completions", self.base_url);

        // Use a slightly shorter timeout for the warmup probe itself
        let _ = self
            .client
            .post(&url)
            .json(&body)
            .timeout(std::time::Duration::from_secs(15))
            .send()
            .await?;

        Ok(())
    }

    async fn maybe_warmup(&self) {
        if self.warmed.load(Ordering::Relaxed) {
            return;
        }

        // Best effort. We mark as warmed immediately so we don't retry on every word.
        // If it fails the first real request will still try.
        let _ = self.warmup().await;
        self.warmed.store(true, Ordering::Relaxed);
    }
}

#[async_trait]
impl WordInfoSource for LocalLlmSource {
    fn name(&self) -> &'static str {
        "LocalLlm"
    }

    async fn fetch(&self, word: &str, lang: Language) -> Result<(SurfaceInfo, SemanticInfo)> {
        // Warmup the model on first use (cold start for local LLMs can be several seconds).
        // Subsequent calls (in same process / chat session) will be much faster.
        self.maybe_warmup().await;

        // Support "system prompt = nothing" as requested.
        // Set LEXFLEX_LLM_NOTHINK=1 or LEXFLEX_LLM_SYSTEM_PROMPT=nothing|empty|""
        // to send no (or minimal) system prompt. Useful for raw model behavior testing.
        let want_nothing = std::env::var("LEXFLEX_LLM_NOTHINK").is_ok()
            || std::env::var("LEXFLEX_LLM_SYSTEM_PROMPT")
                .map(|v| matches!(v.to_lowercase().as_str(), "nothing" | "none" | "empty" | ""))
                .unwrap_or(false);

        let system_content: String = if want_nothing {
            // "system prompt ma być nothing" — send almost nothing.
            // "/nothink" is recognized by some LM Studio / model templates to suppress CoT.
            "/nothink".to_string()
        } else {
            build_system_prompt()
        };

        let user_prompt = format!(
            "Analyze this word for linguistic features and semantics.\n\nWord: {}\nLanguage: {}",
            word, lang
        );

        // Build messages. When user wants literally nothing, we can drop the system message.
        let messages = if want_nothing && system_content == "/nothink" {
            // Keep a tiny system so the frontend sees the nothink directive.
            json!([ {"role": "system", "content": system_content}, {"role": "user", "content": user_prompt} ])
        } else if system_content.trim().is_empty() {
            json!([ {"role": "user", "content": user_prompt} ])
        } else {
            json!([
                {"role": "system", "content": system_content},
                {"role": "user", "content": user_prompt}
            ])
        };

        let body = json!({
            "model": self.model,
            "messages": messages,
            "temperature": 0.0,
            "max_tokens": 400,
            // Many local servers ignore this, but some respect it
            "response_format": { "type": "json_object" }
        });

        let url = format!("{}/chat/completions", self.base_url);

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        let raw: serde_json::Value = resp.json().await?;

        // Extract the assistant message content
        let content = raw["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("{}")
            .trim();

        // Try to extract JSON even if the model added some noise
        let json_str = extract_json(content).unwrap_or_else(|| content.to_string());

        let parsed: LlmResponse = match serde_json::from_str(&json_str) {
            Ok(p) => p,
            Err(e) => {
                // Try one more time with a stricter follow-up if possible (best effort)
                eprintln!("[LocalLlm] JSON parse failed for '{}': {}. Falling back to empty result. Ensure your LM supports strict JSON.", word, e);
                return Ok((SurfaceInfo::default(), SemanticInfo::default()));
            }
        };

        // Basic validation for structured output (important for concept tree building)
        if parsed.is_a.is_empty() {
            eprintln!("[LocalLlm] Warning: model gave empty is_a for '{}'. This hurts concept tree quality. Check model/prompt.", word);
        }

        let mut surface = SurfaceInfo::default();
        surface.lemma = parsed.lemma.filter(|s| !s.is_empty());
        surface.pos = parsed.pos;

        let mut semantics = SemanticInfo::default();
        semantics.is_a = parsed.is_a.into_iter().map(|s| s.to_lowercase()).collect();
        semantics.definitions = parsed.definitions;

        // Use direct fields from the meta prompt for better proposal quality
        if let Some(p) = &parsed.parent_suggestion {
            if !p.is_empty() {
                semantics.hypernyms.push(p.to_lowercase());
            }
        }
        if !parsed.suggested_roles.is_empty() {
            semantics.related.extend(parsed.suggested_roles.iter().map(|r| r.to_lowercase()));
        }

        // Store extra info in definitions for the proposal writer to pick up
        if let Some(f) = &parsed.inherent_features {
            if !f.is_empty() {
                semantics.definitions.push(format!("inherent_features: {}", f));
            }
        }
        if let Some(p) = &parsed.paradigm {
            if !p.is_empty() {
                semantics.definitions.push(format!("paradigm: {}", p));
            }
        }

        Ok((surface, semantics))
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct LlmResponse {
    lemma: Option<String>,
    pos: Option<String>,
    concept: Option<String>,           // preferred existing or new concept id
    is_a: Vec<String>,
    definitions: Vec<String>,
    #[serde(default)]
    parent_suggestion: Option<String>,
    #[serde(default)]
    suggested_roles: Vec<String>,
    #[serde(default)]
    inherent_features: Option<String>,
    #[serde(default)]
    paradigm: Option<String>,
    #[serde(default)]
    confidence: Option<f32>,
}

/// Build a strong system prompt (the full meta description of lexFlex data model).
/// 
/// IMPORTANT: the system prompt can be set to "nothing" (see fetch()):
///   export LEXFLEX_LLM_NOTHINK=1
///   export LEXFLEX_LLM_SYSTEM_PROMPT=nothing
/// When nothing, we send only a minimal "/nothink" (or empty) system message so the
/// model responds more "raw" / without the long architecture instructions.
fn build_system_prompt() -> String {
    r#"You are an expert linguistic data creator for the lexFlex Interlingua system.

=== COMPLETE TOP-DOWN ARCHITECTURE (internalize this) ===

lexFlex has a two-layer design for language independence:

1. GENERIC INTERLINGUA CONCEPT DATABASE (data/concepts/concepts.ron)
   - This is the single source of truth for all meaning.
   - It is a list of concept definitions.
   - Each concept definition has this structure:
     (id: "UPPER_SNAKE_CASE_ID", 
      frame_type: None or Some("Existence" | "Motion" | "Transfer" | "Perception" | "Cognition" | "Emotion" | "Consumption" | "Creation" | "Destruction" | "Possession" | "Communication" | "Statement" | "Custom" | ...),
      roles: list of semantic roles this concept can assign (Agent, Theme, Patient, Recipient, Experiencer, Location, Goal, Source, Instrument, Content, Cognizer, Stimulus, Speaker, Addressee, Message, ...),
      inherent_features: tuple with grammatical/semantic defaults such as gender, number, case, animacy, person, definiteness, countability, concreteness, tense, aspect, mood, voice, evidentiality, honorific_level, classifier, degree, initial_sound, suppletive_comparative, suppletive_superlative, semantic_role
     )
   - Concepts are organized in families/categories via the ontology (e.g. properties like size/quality, entities like people/animals/objects, actions, etc.). A concept belongs to a "rodzina" (family/category) through its parent relationships.
   - Purpose of a concept: it represents a language-independent meaning unit. It defines default features, what frames it can head (frame_type), and what roles it can fill. This allows the same meaning to be expressed in different languages.

2. LANGUAGE-SPECIFIC LEXICONS (data/lexicons/pl/lexicon.ron and data/lexicons/en/lexicon.ron)
   - These map actual words in a language to entries in the generic concept DB.
   - Each entry in a lexicon is a tuple of this exact form:
     ("surface_key", 
      (lemma: "base_form",
       pos: "Noun" | "Verb" | "Adjective" | "Adverb" | "Pronoun" | "Particle" | "Conjunction" | "Preposition" | "Determiner" | "Unknown",
       concept: "EXISTING_GENERIC_ID_FROM_CONCEPTS_RON",
       frame_type: None or Some("...") ,   // usually copied or compatible with the concept
       roles: [ ... ],                     // usually copied or compatible with the concept
       paradigm: None or Some("masculine_consonant" | "feminine_a" | "masculine_personal" | "masculine_ec" | "verb_byc" | "verb_miec" | "verb_ic" | "verb_ec" | "verb_dac" | "verb_ac" | "verb_wać" | "verb_ować" | "adj_y" | other paradigm class),
       features: ( ... full grammatical info: gender, number, case, animacy, person, definiteness, countability, concreteness, tense, aspect, mood, voice, evidentiality, honorific_level, classifier, degree, initial_sound, suppletive_comparative, suppletive_superlative, semantic_role ... )
      )
     )
   - Purpose of a lexicon entry: it tells the system how a particular word form in a particular language realizes a generic concept. The paradigm tells how to generate other forms. The features provide the grammatical information needed for parsing (e.g. case, person, tense) and generation.

The generic concept DB is language-independent. The lexicons are the "translation layer" that connects real words to the generic layer. This is why you can translate between Polish and English: both languages ultimately point to the same generic concepts.

=== ALL POSSIBLE WAYS TO CREATE / EXTEND LEXICON AND CONCEPT ENTRIES ===

When the system encounters an unknown word (via the learner), you are called to help extend the database.

Ways to create lexicon entries:
1. Attach a new surface/lemma to an **existing** generic concept:
   - Choose an already-existing "concept" ID from the generic DB.
   - Provide the lemma (base form).
   - Provide pos, paradigm (if known for the language), and as many features as make sense for this form.
   - The system will create the full tuple in the language's lexicon.ron.

2. Create a **brand new generic concept** + lexicon entry(ies):
   - Invent a new clean UPPER_SNAKE_CASE id (semantically clear, consistent with style of the DB).
   - Provide frame_type, roles, and inherent_features for the concept definition.
   - The system will add the concept to concepts.ron.
   - Then create lexicon entry(ies) pointing to this new id.

3. Bilingual creation:
   - When you analyze a word in one language, the system will typically create:
     - A primary lexicon entry in the input language.
     - A companion entry in the other language (same concept, best-guess lemma for that language).
   - This keeps the two lexicons in sync with the generic DB.

4. Adding morphological detail:
   - For a form, you can specify paradigm (so the morphology engine knows how to inflect it) and features (gender/number/case/person/tense etc.).
   - This is crucial for correct parsing and generation.

5. For closed-class or irregular items:
   - Sometimes explicit inflected forms are added (not just lemmas).

The code in the learner takes your analysis (lemma, pos, chosen/proposed concept, features, paradigm, etc.) and builds the exact RON strings to append.

=== YOUR TASK ===

You will be given one word and its language (pl or en).

Your ONLY output must be a single valid JSON object. No other text.

You must help create the correct data so the system can:
- Recognize the word in future text.
- Assign it the right generic concept.
- Know its grammatical features.
- Generate correct inflected forms.
- Use it correctly in frames and deduction.
- Keep the generic DB and the two language lexicons consistent.

REQUIRED JSON (all keys always present, use null or [] when appropriate):
{
  "lemma": string | null,                    // the base form of the word (critical for Polish)
  "pos": "Noun" | "Verb" | "Adjective" | "Adverb" | "Pronoun" | "Particle" | "Conjunction" | "Preposition" | "Determiner" | "Unknown",
  "concept": string,                         // the generic concept this word should be linked to. Prefer an existing ID from the generic DB if it fits semantically. Only create a new UPPER_SNAKE_CASE ID if truly necessary. The ID must be clear and follow the style of the database.
  "is_a": [string, ...],                     // semantic categories. This helps place the concept in the right family/category in the ontology/hierarchy.
  "definitions": [string, ...],              // 1-3 short, precise English definitions of the meaning.
  "parent_suggestion": string | null,        // suggestion for a parent concept in the hierarchy (a more general concept in the same family). Can be an existing ID or a new one.
  "suggested_roles": [string, ...],          // roles this concept/word can play (Agent, Theme, Patient, Location, etc.).
  "inherent_features": string,               // string describing the feature tuple, e.g. "(gender: Some(Masculine), number: Some(Singular), person: Some(First))" or "()"
  "paradigm": string | null,                 // the morphological paradigm class this belongs to (e.g. "adj_y", "masculine_consonant", "verb_byc", "feminine_a", etc.). Crucial for generation/parsing of forms.
  "frame_type": string | null,               // if this concept is typically used to head a frame/predicate, suggest the appropriate frame_type from the possible ones in the generic DB
  "roles": [string, ...],                    // roles this concept/word can play or assign
  "confidence": number                       // your confidence 0.0-1.0
}

STRICT RULES:
- For Polish, if the given word is inflected, you MUST return the correct base lemma.
- Always output ONLY the JSON. Start with { and end with }.
- Be precise and conservative. The data you provide will be turned into real entries in the live database.

YOU MUST RETURN ONLY THE JSON. NO OTHER TEXT AT ALL.

Analyze the following word and language and produce the JSON."#.to_string()
}

/// Very tolerant JSON extractor (handles cases where model adds a bit of text anyway).
fn extract_json(text: &str) -> Option<String> {
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            if end > start {
                return Some(text[start..=end].to_string());
            }
        }
    }
    None
}
