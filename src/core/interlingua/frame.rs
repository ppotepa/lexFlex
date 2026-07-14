use super::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Frame {
    Transfer {
        agent: Entity,
        recipient: Entity,
        theme: Entity,
        verb_concept: String,
    },
    Motion {
        mover: Entity,
        source: Option<Entity>,
        goal: Option<Entity>,
        path: Option<Entity>,
        verb_concept: String,
    },
    Creation {
        creator: Entity,
        created: Entity,
        material: Option<Entity>,
        verb_concept: String,
    },
    Destruction {
        agent: Entity,
        patient: Entity,
        instrument: Option<Entity>,
        verb_concept: String,
    },
    Perception {
        experiencer: Entity,
        stimulus: Entity,
        verb_concept: String,
    },
    Cognition {
        cognizer: Entity,
        content: Entity,
        verb_concept: String,
    },
    Emotion {
        experiencer: Entity,
        stimulus: Entity,
        verb_concept: String,
    },
    Communication {
        speaker: Entity,
        addressee: Option<Entity>,
        message: Entity,
        verb_concept: String,
    },
    Statement {
        subject: Entity,
        property: Entity,
        verb_concept: String,
    },
    Existence {
        entity: Entity,
        location: Option<Entity>,
        verb_concept: String,
    },
    Possession {
        possessor: Entity,
        possessed: Entity,
        verb_concept: String,
    },
    Consumption {
        agent: Entity,
        patient: Entity,
        verb_concept: String,
    },
    Custom {
        name: String,
        roles: Vec<(SemanticRole, Entity)>,
    },
}

impl Frame {
    pub fn frame_type_name(&self) -> &str {
        match self {
            Frame::Transfer { .. } => "Transfer",
            Frame::Motion { .. } => "Motion",
            Frame::Creation { .. } => "Creation",
            Frame::Destruction { .. } => "Destruction",
            Frame::Perception { .. } => "Perception",
            Frame::Cognition { .. } => "Cognition",
            Frame::Emotion { .. } => "Emotion",
            Frame::Communication { .. } => "Communication",
            Frame::Statement { .. } => "Statement",
            Frame::Existence { .. } => "Existence",
            Frame::Possession { .. } => "Possession",
            Frame::Consumption { .. } => "Consumption",
            Frame::Custom { name, .. } => name,
        }
    }

    pub fn required_roles(&self) -> Vec<SemanticRole> {
        match self {
            Frame::Transfer { .. } => {
                vec![SemanticRole::Agent, SemanticRole::Recipient, SemanticRole::Theme]
            }
            Frame::Motion { .. } => vec![SemanticRole::Agent],
            Frame::Creation { .. } => {
                vec![SemanticRole::Creator, SemanticRole::Created]
            }
            Frame::Destruction { .. } => {
                vec![SemanticRole::Agent, SemanticRole::Patient]
            }
            Frame::Perception { .. } => {
                vec![SemanticRole::Experiencer, SemanticRole::Stimulus]
            }
            Frame::Cognition { .. } => {
                vec![SemanticRole::Cognizer, SemanticRole::Content]
            }
            Frame::Emotion { .. } => {
                vec![SemanticRole::Experiencer, SemanticRole::Stimulus]
            }
            Frame::Communication { .. } => {
                vec![SemanticRole::Speaker, SemanticRole::Message]
            }
            Frame::Statement { .. } => {
                vec![SemanticRole::Topic, SemanticRole::Theme]
            }
            Frame::Existence { .. } => vec![SemanticRole::Theme],
            Frame::Possession { .. } => {
                vec![SemanticRole::Agent, SemanticRole::Theme]
            }
            Frame::Consumption { .. } => {
                vec![SemanticRole::Agent, SemanticRole::Patient]
            }
            Frame::Custom { roles, .. } => roles.iter().map(|(r, _)| *r).collect(),
        }
    }

    pub fn entities(&self) -> Vec<&Entity> {
        match self {
            Frame::Transfer { agent, recipient, theme, .. } => {
                vec![agent, recipient, theme]
            }
            Frame::Motion { mover, source, goal, path, .. } => {
                let mut v = vec![mover];
                if let Some(s) = source { v.push(s); }
                if let Some(g) = goal { v.push(g); }
                if let Some(p) = path { v.push(p); }
                v
            }
            Frame::Creation { creator, created, material, .. } => {
                let mut v = vec![creator, created];
                if let Some(m) = material { v.push(m); }
                v
            }
            Frame::Destruction { agent, patient, instrument, .. } => {
                let mut v = vec![agent, patient];
                if let Some(i) = instrument { v.push(i); }
                v
            }
            Frame::Perception { experiencer, stimulus, .. } => vec![experiencer, stimulus],
            Frame::Cognition { cognizer, content, .. } => vec![cognizer, content],
            Frame::Emotion { experiencer, stimulus, .. } => vec![experiencer, stimulus],
            Frame::Communication { speaker, addressee, message, .. } => {
                let mut v = vec![speaker, message];
                if let Some(a) = addressee { v.push(a); }
                v
            }
            Frame::Statement { subject, property, .. } => vec![subject, property],
            Frame::Existence { entity, location, .. } => {
                let mut v = vec![entity];
                if let Some(l) = location { v.push(l); }
                v
            }
            Frame::Possession { possessor, possessed, .. } => vec![possessor, possessed],
            Frame::Consumption { agent, patient, .. } => vec![agent, patient],
            Frame::Custom { roles, .. } => roles.iter().map(|(_, e)| e).collect(),
        }
    }

    /// Primary agent/subject role for discourse topic tracking and zero-anaphora resolution.
    pub fn agent_entity(&self) -> Option<&Entity> {
        match self {
            Frame::Transfer { agent, .. } => Some(agent),
            Frame::Motion { mover, .. } => Some(mover),
            Frame::Creation { creator, .. } => Some(creator),
            Frame::Destruction { agent, .. } => Some(agent),
            Frame::Perception { experiencer, .. } => Some(experiencer),
            Frame::Cognition { cognizer, .. } => Some(cognizer),
            Frame::Emotion { experiencer, .. } => Some(experiencer),
            Frame::Communication { speaker, .. } => Some(speaker),
            Frame::Statement { subject, .. } => Some(subject),
            Frame::Possession { possessor, .. } => Some(possessor),
            Frame::Consumption { agent, .. } => Some(agent),
            Frame::Existence { .. } => None,
            Frame::Custom { roles, .. } => roles
                .iter()
                .find(|(r, _)| {
                    matches!(
                        r,
                        SemanticRole::Agent
                            | SemanticRole::Experiencer
                            | SemanticRole::Speaker
                    )
                })
                .map(|(_, e)| e),
        }
    }

    pub fn agent_entity_mut(&mut self) -> Option<&mut Entity> {
        match self {
            Frame::Transfer { agent, .. } => Some(agent),
            Frame::Motion { mover, .. } => Some(mover),
            Frame::Creation { creator, .. } => Some(creator),
            Frame::Destruction { agent, .. } => Some(agent),
            Frame::Perception { experiencer, .. } => Some(experiencer),
            Frame::Cognition { cognizer, .. } => Some(cognizer),
            Frame::Emotion { experiencer, .. } => Some(experiencer),
            Frame::Communication { speaker, .. } => Some(speaker),
            Frame::Statement { subject, .. } => Some(subject),
            Frame::Possession { possessor, .. } => Some(possessor),
            Frame::Consumption { agent, .. } => Some(agent),
            Frame::Existence { .. } => None,
            Frame::Custom { roles, .. } => roles
                .iter_mut()
                .find(|(r, _)| {
                    matches!(
                        r,
                        SemanticRole::Agent
                            | SemanticRole::Experiencer
                            | SemanticRole::Speaker
                    )
                })
                .map(|(_, e)| e),
        }
    }

    pub fn goal_entity_mut(&mut self) -> Option<&mut Entity> {
        match self {
            Frame::Motion { goal, .. } => goal.as_mut(),
            Frame::Transfer { recipient, .. } => Some(recipient),
            _ => None,
        }
    }

    pub fn theme_entity_mut(&mut self) -> Option<&mut Entity> {
        match self {
            Frame::Transfer { theme, .. } => Some(theme),
            Frame::Motion { .. } => None,
            Frame::Consumption { patient, .. } => Some(patient),
            _ => None,
        }
    }

    pub fn set_agent_entity(&mut self, entity: Entity) {
        match self {
            Frame::Transfer { agent, .. } => *agent = entity,
            Frame::Motion { mover, .. } => *mover = entity,
            Frame::Creation { creator, .. } => *creator = entity,
            Frame::Destruction { agent, .. } => *agent = entity,
            Frame::Perception { experiencer, .. } => *experiencer = entity,
            Frame::Cognition { cognizer, .. } => *cognizer = entity,
            Frame::Emotion { experiencer, .. } => *experiencer = entity,
            Frame::Communication { speaker, .. } => *speaker = entity,
            Frame::Statement { subject, .. } => *subject = entity,
            Frame::Possession { possessor, .. } => *possessor = entity,
            Frame::Consumption { agent, .. } => *agent = entity,
            Frame::Existence { .. } => {}
            Frame::Custom { roles, .. } => {
                if let Some((_, e)) = roles
                    .iter_mut()
                    .find(|(r, _)| matches!(r, SemanticRole::Agent | SemanticRole::Experiencer))
                {
                    *e = entity;
                }
            }
        }
    }

    pub fn entities_mut(&mut self) -> Vec<&mut Entity> {
        match self {
            Frame::Transfer { agent, recipient, theme, .. } => {
                vec![agent, recipient, theme]
            }
            Frame::Motion { mover, source, goal, path, .. } => {
                let mut v = vec![mover];
                if let Some(s) = source { v.push(s); }
                if let Some(g) = goal { v.push(g); }
                if let Some(p) = path { v.push(p); }
                v
            }
            Frame::Creation { creator, created, material, .. } => {
                let mut v = vec![creator, created];
                if let Some(m) = material { v.push(m); }
                v
            }
            Frame::Destruction { agent, patient, instrument, .. } => {
                let mut v = vec![agent, patient];
                if let Some(i) = instrument { v.push(i); }
                v
            }
            Frame::Perception { experiencer, stimulus, .. } => vec![experiencer, stimulus],
            Frame::Cognition { cognizer, content, .. } => vec![cognizer, content],
            Frame::Emotion { experiencer, stimulus, .. } => vec![experiencer, stimulus],
            Frame::Communication { speaker, addressee, message, .. } => {
                let mut v = vec![speaker, message];
                if let Some(a) = addressee { v.push(a); }
                v
            }
            Frame::Statement { subject, property, .. } => vec![subject, property],
            Frame::Existence { entity, location, .. } => {
                let mut v = vec![entity];
                if let Some(l) = location { v.push(l); }
                v
            }
            Frame::Possession { possessor, possessed, .. } => vec![possessor, possessed],
            Frame::Consumption { agent, patient, .. } => vec![agent, patient],
            Frame::Custom { roles, .. } => roles.iter_mut().map(|(_, e)| e).collect(),
        }
    }
}
