use crate::core::interlingua::SemanticRole;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MentionPathSegment {
    Frame { semantic_sentence_ordinal: usize, frame_ordinal: usize },
    Role { role: SemanticRole, occurrence: usize },
    Coordination { ordinal: usize },
    Modifier { ordinal: usize },
}

pub fn role_tag(role: SemanticRole) -> &'static str {
    match role {
        SemanticRole::Agent => "Agent",
        SemanticRole::Patient => "Patient",
        SemanticRole::Theme => "Theme",
        SemanticRole::Recipient => "Recipient",
        SemanticRole::Experiencer => "Experiencer",
        SemanticRole::Stimulus => "Stimulus",
        SemanticRole::Source => "Source",
        SemanticRole::Goal => "Goal",
        SemanticRole::Location => "Location",
        SemanticRole::Instrument => "Instrument",
        SemanticRole::Beneficiary => "Beneficiary",
        SemanticRole::Topic => "Topic",
        SemanticRole::Creator => "Creator",
        SemanticRole::Created => "Created",
        SemanticRole::Cognizer => "Cognizer",
        SemanticRole::Content => "Content",
        SemanticRole::Speaker => "Speaker",
        SemanticRole::Message => "Message",
        SemanticRole::Accompaniment => "Accompaniment",
    }
}

pub fn mention_path_string(segments: &[MentionPathSegment]) -> String {
    let mut parts = Vec::new();
    for segment in segments {
        match segment {
            MentionPathSegment::Frame {
                semantic_sentence_ordinal,
                frame_ordinal,
            } => parts.push(format!("ss{semantic_sentence_ordinal:04}/f{frame_ordinal:04}")),
            MentionPathSegment::Role { role, occurrence } => {
                parts.push(format!("role-{}-{occurrence:04}", role_tag(*role)));
            }
            MentionPathSegment::Coordination { ordinal } => {
                parts.push(format!("coord-{ordinal:04}"));
            }
            MentionPathSegment::Modifier { ordinal } => {
                parts.push(format!("mod-{ordinal:04}"));
            }
        }
    }
    parts.join("/")
}
