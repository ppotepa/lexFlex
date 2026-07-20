use crate::{Form, FormId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormIndex {
    by_normalized_surface: BTreeMap<String, Vec<FormId>>,
}

impl FormIndex {
    pub fn build(forms: impl IntoIterator<Item = Form>) -> Self {
        let mut index = Self::default();
        for form in forms {
            index
                .by_normalized_surface
                .entry(form.normalized.clone())
                .or_default()
                .push(form.id.clone());
        }
        for ids in index.by_normalized_surface.values_mut() {
            ids.sort();
            ids.dedup();
        }
        index
    }

    pub fn lookup(&self, normalized_surface: &str) -> &[FormId] {
        self.by_normalized_surface
            .get(normalized_surface)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}
