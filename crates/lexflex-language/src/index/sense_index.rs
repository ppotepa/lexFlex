use crate::{LexemeId, LexicalSense, LexicalSenseId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SenseIndex {
    by_lexeme: BTreeMap<LexemeId, Vec<LexicalSenseId>>,
}

impl SenseIndex {
    pub fn build(senses: impl IntoIterator<Item = LexicalSense>) -> Self {
        let mut index = Self::default();
        for sense in senses {
            index
                .by_lexeme
                .entry(sense.lexeme_id.clone())
                .or_default()
                .push(sense.id.clone());
        }
        for ids in index.by_lexeme.values_mut() {
            ids.sort();
            ids.dedup();
        }
        index
    }

    pub fn lookup(&self, lexeme_id: &LexemeId) -> &[LexicalSenseId] {
        self.by_lexeme
            .get(lexeme_id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}
