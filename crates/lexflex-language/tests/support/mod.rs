use lexflex_model::ConceptCatalog;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Deserialize)]
struct EntityPackage {
    entities: BTreeMap<lexflex_model::EntityId, lexflex_model::EntityDefinition>,
}

pub fn catalog() -> ConceptCatalog {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/model");
    let concepts_source =
        std::fs::read_to_string(root.join("concepts.ron")).expect("concept catalog");
    let entities_source =
        std::fs::read_to_string(root.join("entities.ron")).expect("entity catalog");
    let mut catalog: ConceptCatalog = ron::from_str(&concepts_source).expect("parse catalog");
    let entities: EntityPackage = ron::from_str(&entities_source).expect("parse entities");
    catalog.entities = entities.entities;
    catalog
}

#[allow(dead_code)]
pub fn temp_root(prefix: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should move forward")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{stamp}"))
}
