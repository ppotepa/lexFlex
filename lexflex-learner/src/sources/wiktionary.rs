use async_trait::async_trait;
use crate::types::{Language, SemanticInfo, SurfaceInfo};
use crate::error::Result;
use crate::sources::WordInfoSource;
use serde_json::Value;

pub struct WiktionarySource {
    client: reqwest::Client,
}

impl WiktionarySource {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    fn wiki_domain(&self, lang: Language) -> &'static str {
        match lang {
            Language::Pl => "pl.wiktionary.org",
            Language::En => "en.wiktionary.org",
        }
    }
}

#[async_trait]
impl WordInfoSource for WiktionarySource {
    fn name(&self) -> &'static str {
        "Wiktionary"
    }

    async fn fetch(&self, word: &str, lang: Language) -> Result<(SurfaceInfo, SemanticInfo)> {
        let domain = self.wiki_domain(lang);
        let encoded = urlencoding::encode(word);

        // Use action=parse for extractable content (simplified)
        let url = format!(
            "https://{}/w/api.php?action=parse&page={}&format=json&prop=text&contentmodel=wikitext",
            domain, encoded
        );

        let resp: Value = self.client.get(&url).send().await?.json().await?;

        let mut definitions = vec![];
        let examples = vec![];

        if let Some(text) = resp["parse"]["text"]["*"].as_str() {
            // Very naive extraction - in real version use better parser
            for line in text.lines() {
                let line = line.trim();
                if line.starts_with("<li>") || line.contains("defn") {
                    let cleaned = line
                        .replace("<li>", "")
                        .replace("</li>", "")
                        .replace("<p>", "")
                        .replace("</p>", "")
                        .replace("<ol>", "")
                        .replace("</ol>", "");
                    if !cleaned.is_empty() && cleaned.len() > 10 {
                        definitions.push(cleaned);
                    }
                }
            }
        }

        let surface = SurfaceInfo {
            lemma: Some(word.to_string()),
            ..Default::default()
        };

        let semantics = SemanticInfo {
            definitions,
            usage_examples: examples,
            ..Default::default()
        };

        Ok((surface, semantics))
    }
}
