use async_trait::async_trait;
use crate::types::{Language, SemanticInfo, SurfaceInfo};
use crate::error::Result;
use crate::sources::WordInfoSource;
use serde_json::Value;

pub struct ConceptNetSource {
    client: reqwest::Client,
}

impl ConceptNetSource {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    fn lang_prefix(&self, lang: Language) -> &'static str {
        match lang {
            Language::Pl => "pl",
            Language::En => "en",
        }
    }
}

#[async_trait]
impl WordInfoSource for ConceptNetSource {
    fn name(&self) -> &'static str {
        "ConceptNet"
    }

    async fn fetch(&self, word: &str, lang: Language) -> Result<(SurfaceInfo, SemanticInfo)> {
        let prefix = self.lang_prefix(lang);
        let encoded = urlencoding::encode(word);
        let url = format!(
            "http://api.conceptnet.io/query?node=/c/{}/{}&limit=30",
            prefix, encoded
        );

        let resp: Value = self.client.get(&url).send().await?.json().await?;

        let mut is_a = vec![];
        let mut related = vec![];
        let mut synonyms = vec![];

        if let Some(edges) = resp["edges"].as_array() {
            for edge in edges {
                let rel = edge["rel"]["@id"].as_str().unwrap_or("");
                let end = edge["end"]["@id"].as_str().unwrap_or("").to_string();
                let start = edge["start"]["@id"].as_str().unwrap_or("").to_string();

                let target = if end.contains(&format!("/c/{}/", prefix)) && !end.contains(word) {
                    end
                } else if start.contains(&format!("/c/{}/", prefix)) && !start.contains(word) {
                    start
                } else {
                    continue;
                };

                let label = target.split('/').last().unwrap_or(&target).replace('_', " ");

                match rel {
                    "/r/IsA" | "/r/InstanceOf" => is_a.push(label),
                    "/r/RelatedTo" => related.push(label),
                    "/r/Synonym" => synonyms.push(label),
                    "/r/HasA" | "/r/PartOf" | "/r/MemberOf" | "/r/HasProperty" => {
                        // maximize data: treat as related/hypernym-ish evidence
                        related.push(label);
                    }
                    _ => {}
                }
            }
        }

        let surface = SurfaceInfo {
            lemma: Some(word.to_string()),
            ..Default::default()
        };

        let semantics = SemanticInfo {
            is_a,
            related,
            synonyms,
            ..Default::default()
        };

        // Honest: return only what API provided. No seeding for verification.
        // Real IsA/relations come from ConceptNet /r/IsA etc when available.
        Ok((surface, semantics))
    }
}
