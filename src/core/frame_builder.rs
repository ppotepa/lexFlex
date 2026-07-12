use crate::core::interlingua::{
    Animacy, Case, Coordination, Entity, Frame, Person, SemanticRole,
};
use crate::data::lexicon::Lexicon;

/// Assign frame entities to lexical roles using semantic features (not raw token order).
pub fn assign_entities_to_roles(roles: &[SemanticRole], entities: &[Entity]) -> Vec<Option<Entity>> {
    let mut assigned: Vec<Option<Entity>> = vec![None; roles.len()];
    let mut unassigned: Vec<Entity> = Vec::new();

    for entity in entities {
        let mut placed = false;
        if let Some(person) = entity.features.person {
            if person == Person::First || person == Person::Second {
                for target in [SemanticRole::Agent, SemanticRole::Experiencer] {
                    if let Some(idx) = roles.iter().position(|r| *r == target) {
                        if assigned[idx].is_none() {
                            assigned[idx] = Some(entity.clone());
                            placed = true;
                            break;
                        }
                    }
                }
            }
        }
        if !placed {
            unassigned.push(entity.clone());
        }
    }

    let mut still_unassigned: Vec<Entity> = Vec::new();
    for entity in unassigned {
        if let Some(role) = entity.features.semantic_role {
            let mapped = match role {
                SemanticRole::Goal if roles.contains(&SemanticRole::Recipient) => {
                    SemanticRole::Recipient
                }
                SemanticRole::Beneficiary if roles.contains(&SemanticRole::Recipient) => {
                    SemanticRole::Recipient
                }
                other => other,
            };
            if let Some(idx) = roles.iter().position(|r| *r == mapped) {
                if assigned[idx].is_none() {
                    assigned[idx] = Some(entity.clone());
                    continue;
                }
            }
        }

        if let Some(c) = entity.features.case {
            let target_role = match c {
                Case::Nominative => roles
                    .iter()
                    .find(|r| **r == SemanticRole::Agent || **r == SemanticRole::Experiencer)
                    .copied(),
                Case::Dative => roles
                    .iter()
                    .find(|r| **r == SemanticRole::Recipient)
                    .copied(),
                Case::Accusative => roles
                    .iter()
                    .find(|r| **r == SemanticRole::Theme || **r == SemanticRole::Patient)
                    .copied(),
                Case::Genitive => roles
                    .iter()
                    .find(|r| **r == SemanticRole::Source || **r == SemanticRole::Patient)
                    .copied(),
                Case::Instrumental => roles
                    .iter()
                    .find(|r| **r == SemanticRole::Instrument || **r == SemanticRole::Location)
                    .copied(),
                Case::Locative => roles
                    .iter()
                    .find(|r| **r == SemanticRole::Location)
                    .copied(),
                _ => None,
            };
            if let Some(role) = target_role {
                if let Some(idx) = roles.iter().position(|r| *r == role) {
                    if assigned[idx].is_none() {
                        assigned[idx] = Some(entity.clone());
                        continue;
                    }
                }
            }
        }
        still_unassigned.push(entity.clone());
    }

    for entity in &still_unassigned {
        let is_animate = entity.features.animacy == Some(Animacy::Animate);
        let preferred: &[SemanticRole] = if is_animate {
            &[
                SemanticRole::Agent,
                SemanticRole::Experiencer,
                SemanticRole::Recipient,
            ]
        } else {
            &[
                SemanticRole::Theme,
                SemanticRole::Patient,
                SemanticRole::Location,
                SemanticRole::Goal,
            ]
        };
        let mut placed = false;
        for role in preferred {
            if let Some(idx) = roles.iter().position(|r| *r == *role) {
                if assigned[idx].is_none() {
                    assigned[idx] = Some(entity.clone());
                    placed = true;
                    break;
                }
            }
        }
        if !placed {
            if let Some(slot) = assigned.iter_mut().find(|s| s.is_none()) {
                *slot = Some(entity.clone());
            }
        }
    }

    assigned
}

pub fn entity_for_role(roles: &[SemanticRole], assigned: &[Option<Entity>], role: SemanticRole) -> Entity {
    roles
        .iter()
        .position(|r| *r == role)
        .and_then(|idx| assigned.get(idx).and_then(|o| o.clone()))
        .unwrap_or_else(|| Entity::new(crate::core::interlingua::ConceptId::new("unknown")))
}

pub fn build_frame_from_roles(
    frame_type: &str,
    roles: &[SemanticRole],
    entities: &[Entity],
    verb_concept: &str,
    lexicon: &Lexicon,
) -> Frame {
    let assigned = assign_entities_to_roles(roles, entities);
    let get = |role: SemanticRole| entity_for_role(roles, &assigned, role);

    if verb_concept == "HAVE_NAME" {
        let name_entity = entities
            .iter()
            .rev()
            .find(|e| e.name.as_deref() != Some("I") || e.features.person.is_none())
            .or_else(|| entities.last());
        if let Some(entity) = name_entity {
            return Frame::Possession {
                possessor: Entity::new(crate::core::interlingua::ConceptId::new("PERSON"))
                    .with_name("I"),
                possessed: entity.clone(),
                verb_concept: verb_concept.to_string(),
            };
        }
    }

    match frame_type {
        "Transfer" => Frame::Transfer {
            agent: get(SemanticRole::Agent),
            recipient: get(SemanticRole::Recipient),
            theme: get(SemanticRole::Theme),
            verb_concept: verb_concept.to_string(),
        },
        "Motion" => Frame::Motion {
            mover: get(SemanticRole::Agent),
            source: {
                let s = get(SemanticRole::Source);
                if s.concept.0 == "unknown" {
                    None
                } else {
                    Some(s)
                }
            },
            goal: {
                let g = get(SemanticRole::Goal);
                if g.concept.0 == "unknown" {
                    None
                } else {
                    Some(g)
                }
            },
            path: None,
            verb_concept: verb_concept.to_string(),
        },
        "Perception" => Frame::Perception {
            experiencer: get(SemanticRole::Experiencer),
            stimulus: get(SemanticRole::Stimulus),
            verb_concept: verb_concept.to_string(),
        },
        "Cognition" => Frame::Cognition {
            cognizer: get(SemanticRole::Cognizer),
            content: get(SemanticRole::Content),
            verb_concept: verb_concept.to_string(),
        },
        "Emotion" => Frame::Emotion {
            experiencer: get(SemanticRole::Experiencer),
            stimulus: get(SemanticRole::Stimulus),
            verb_concept: verb_concept.to_string(),
        },
        "Destruction" => Frame::Destruction {
            agent: get(SemanticRole::Agent),
            patient: get(SemanticRole::Patient),
            instrument: None,
            verb_concept: verb_concept.to_string(),
        },
        "Consumption" => Frame::Consumption {
            agent: get(SemanticRole::Agent),
            patient: get(SemanticRole::Patient),
            verb_concept: verb_concept.to_string(),
        },
        "Communication" => Frame::Communication {
            speaker: get(SemanticRole::Agent),
            addressee: {
                let a = get(SemanticRole::Recipient);
                if a.concept.0 == "unknown" {
                    None
                } else {
                    Some(a)
                }
            },
            message: get(SemanticRole::Theme),
            verb_concept: verb_concept.to_string(),
        },
        "Creation" => Frame::Creation {
            creator: get(SemanticRole::Agent),
            created: get(SemanticRole::Theme),
            material: None,
            verb_concept: verb_concept.to_string(),
        },
        "Possession" => Frame::Possession {
            possessor: get(SemanticRole::Agent),
            possessed: get(SemanticRole::Theme),
            verb_concept: verb_concept.to_string(),
        },
        "Existence" => {
            let mut entity = get(SemanticRole::Theme);
            if entity.concept.0 == "unknown" {
                entity = get(SemanticRole::Agent);
            }
            let loc = get(SemanticRole::Location);
            let is_adj = |name: &str| {
                lexicon
                    .lookup_by_form(&name.to_lowercase())
                    .or_else(|| lexicon.lookup_by_lemma(name))
                    .map_or(false, |e| e.pos == "Adjective")
            };
            if loc.concept.0 != "unknown" {
                let loc_name = loc.name.as_deref().unwrap_or("");
                if is_adj(loc_name) {
                    let all_adj: Vec<Entity> = entities
                        .iter()
                        .filter(|e| is_adj(e.name.as_deref().unwrap_or("")))
                        .cloned()
                        .collect();
                    let property = if all_adj.len() >= 2 {
                        let first = all_adj[0].clone();
                        let coord = Coordination {
                            items: all_adj,
                            conjunction: "and".to_string(),
                        };
                        let mut ce = first;
                        ce.coordination = Some(coord);
                        ce
                    } else if all_adj.len() == 1 {
                        all_adj[0].clone()
                    } else {
                        loc.clone()
                    };
                    let non_adj: Vec<Entity> = entities
                        .iter()
                        .filter(|e| !is_adj(e.name.as_deref().unwrap_or("")))
                        .cloned()
                        .collect();
                    let subject = if non_adj.is_empty() {
                        Entity::new(crate::core::interlingua::ConceptId::new("DUMMY_SUBJECT"))
                            .with_name("it")
                    } else if non_adj.len() == 1 {
                        non_adj[0].clone()
                    } else {
                        let first = non_adj[0].clone();
                        let coord = Coordination {
                            items: non_adj,
                            conjunction: "and".to_string(),
                        };
                        let mut ce = first;
                        ce.coordination = Some(coord);
                        ce
                    };
                    return Frame::Statement {
                        subject,
                        property,
                        verb_concept: verb_concept.to_string(),
                    };
                }
            }
            Frame::Existence {
                entity,
                location: if loc.concept.0 != "unknown" {
                    Some(loc)
                } else {
                    None
                },
                verb_concept: verb_concept.to_string(),
            }
        }
        _ => Frame::Statement {
            subject: get(SemanticRole::Topic),
            property: get(SemanticRole::Theme),
            verb_concept: verb_concept.to_string(),
        },
    }
}