use lexflex_model::{Evidence, EvidenceId, EvidenceSet};

fn evidence(source_id: &str) -> Evidence {
    Evidence::create(source_id, None, None).expect("valid evidence")
}

#[test]
fn deserialize_rejects_key_mismatch() {
    let evidence = evidence("source:a");
    let json = serde_json::json!({
        "evidence:wrong": {
            "id": evidence.id(),
            "source_id": evidence.source_id(),
            "span": evidence.span(),
            "source_hash": evidence.source_hash()
        }
    });

    let result = serde_json::from_value::<EvidenceSet>(json);

    assert!(result.is_err());
}

#[test]
fn deserialize_rejects_invalid_evidence_id() {
    let evidence = evidence("source:a");
    let json = serde_json::json!({
        evidence.id().as_str(): {
            "id": EvidenceId::new_unchecked("evidence:wrong"),
            "source_id": evidence.source_id(),
            "span": evidence.span(),
            "source_hash": evidence.source_hash()
        }
    });

    let result = serde_json::from_value::<EvidenceSet>(json);

    assert!(result.is_err());
}
