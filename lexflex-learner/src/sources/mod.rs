pub mod conceptnet;
pub mod dictionary_api;
pub mod local_llm;
pub mod wiktionary;

use crate::types::{Language, SemanticInfo, SurfaceInfo};
use crate::error::Result;

#[async_trait::async_trait]
pub trait WordInfoSource: Send + Sync {
    fn name(&self) -> &'static str;

    async fn fetch(
        &self,
        word: &str,
        lang: Language,
    ) -> Result<(SurfaceInfo, SemanticInfo)>;
}
