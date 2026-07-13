use async_trait::async_trait;
use crate::types::{Language, SemanticInfo, SurfaceInfo};
use crate::error::Result;
use crate::sources::WordInfoSource;
use serde_json::Value;

pub struct DictionaryApiSource {
    client: reqwest::Client,
}

impl DictionaryApiSource {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl WordInfoSource for DictionaryApiSource {
    fn name(&self) -> &'static str {
        "DictionaryAPI.dev"
    }

    async fn fetch(&self, word: &str, _lang: Language) -> Result<(SurfaceInfo, SemanticInfo)> {
        // DictionaryAPI.dev is English-centric; we always query the /en/ endpoint.
        // Non-English results are best-effort (definitions may be limited).
        let url = format!("https://api.dictionaryapi.dev/api/v2/entries/en/{}", word);

        let resp: Value = match self.client.get(&url).send().await {
            Ok(r) if r.status().is_success() => r.json().await?,
            _ => return Ok((SurfaceInfo::default(), SemanticInfo::default())),
        };

        let mut definitions = vec![];
        let mut examples = vec![];
        let mut pos = None;

        if let Some(entries) = resp.as_array() {
            for entry in entries {
                if let Some(_phonetic) = entry["phonetic"].as_str() {
                    // could store
                }

                if let Some(meanings) = entry["meanings"].as_array() {
                    for meaning in meanings {
                        if pos.is_none() {
                            if let Some(p) = meaning["partOfSpeech"].as_str() {
                                pos = Some(p.to_string());
                            }
                        }

                        if let Some(defs) = meaning["definitions"].as_array() {
                            for d in defs {
                                if let Some(def) = d["definition"].as_str() {
                                    definitions.push(def.to_string());
                                }
                                if let Some(ex) = d["example"].as_str() {
                                    examples.push(ex.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        let surface = SurfaceInfo {
            lemma: Some(word.to_string()),
            pos,
            example_sentences: examples,
            ..Default::default()
        };

        let semantics = SemanticInfo {
            definitions,
            ..Default::default()
        };

        Ok((surface, semantics))
    }
}
