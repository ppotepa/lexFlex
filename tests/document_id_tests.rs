use lexflex::document::{DocumentId, DocumentIdError, DocumentIdFactory};

#[test]
fn document_ids_are_deterministic() {
    let a = DocumentIdFactory::from_source("pl", "Tomek poszedł.");
    let b = DocumentIdFactory::from_source("pl", "Tomek poszedł.");
    let c = DocumentIdFactory::from_source("pl", "Anna poszła.");
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn document_id_roundtrip_and_validation() {
    let id = DocumentId::new("doc-pl-0123456789abcdef0123456789abcdef").unwrap();
    assert_eq!(id.as_str(), "doc-pl-0123456789abcdef0123456789abcdef");
    assert_eq!(id.to_string(), "doc-pl-0123456789abcdef0123456789abcdef");
    assert_eq!(id.clone().into_string(), "doc-pl-0123456789abcdef0123456789abcdef");
    assert!(matches!(DocumentId::new(""), Err(DocumentIdError::Empty)));
    assert!(matches!(DocumentId::new("bad id"), Err(DocumentIdError::InvalidCharacter { .. })));
    assert!(matches!(DocumentId::new("bad/id"), Err(DocumentIdError::InvalidCharacter { .. })));
    assert!(DocumentId::new(&"a".repeat(97)).is_err());
    assert!(DocumentId::new("id-🙂").is_err());
}

#[test]
fn document_factories_produce_valid_ids() {
    let doc = DocumentIdFactory::from_source("pl", "Tomek poszedł.");
    assert!(DocumentId::new(doc.as_str()).is_ok());
    assert!(DocumentId::new(DocumentIdFactory::block(&doc, 1).as_str()).is_ok());
    assert!(DocumentId::new(DocumentIdFactory::paragraph(&doc, 1).as_str()).is_ok());
    assert!(DocumentId::new(DocumentIdFactory::sentence(&doc, 1, 2).as_str()).is_ok());
    assert!(DocumentId::new(DocumentIdFactory::diagnostic(&doc, 1).as_str()).is_ok());
    assert!(DocumentId::new(DocumentIdFactory::provenance(&doc, 1).as_str()).is_ok());
}

#[test]
fn document_factory_hash_is_32_hex_chars() {
    let id = DocumentIdFactory::from_source("pl", "Tomek poszedł.");
    let hash = id.as_str().rsplit('-').next().unwrap();
    assert_eq!(hash.len(), 32);
    assert!(hash.chars().all(|ch| ch.is_ascii_hexdigit()));
}

#[test]
fn document_factory_is_sensitive_to_language_and_source() {
    let pl = DocumentIdFactory::from_source("pl", "Tomek poszedł.");
    let en = DocumentIdFactory::from_source("en", "Tomek poszedł.");
    let other = DocumentIdFactory::from_source("pl", "Anna poszła.");
    assert_ne!(pl, en);
    assert_ne!(pl, other);
}

#[test]
fn provenance_ids_are_deterministic() {
    let doc = DocumentIdFactory::from_source("pl", "Tomek poszedł.");
    let a = DocumentIdFactory::provenance(&doc, 7);
    let b = DocumentIdFactory::provenance(&doc, 7);
    assert_eq!(a, b);
}
