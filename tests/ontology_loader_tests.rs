use lexflex::core::interlingua::ConceptId;
use lexflex::data::loader::load_ontology;
use std::path::Path;

#[test]
fn runtime_loads_declared_ontology_hierarchy() {
    let ontology = load_ontology(Path::new("data/ontology/ontology.ron")).expect("ontology should load");
    assert!(ontology.is_a(&ConceptId::new("CITY"), &ConceptId::new("LOCATION")));
    assert!(ontology.is_a(&ConceptId::new("CITY"), &ConceptId::new("ENTITY")));
}
