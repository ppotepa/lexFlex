use crate::core::interlingua::SemanticRole;

pub fn parse_role_str(s: &str) -> Option<SemanticRole> {
    match s {
        "Agent" => Some(SemanticRole::Agent),
        "Patient" => Some(SemanticRole::Patient),
        "Theme" => Some(SemanticRole::Theme),
        "Recipient" => Some(SemanticRole::Recipient),
        "Experiencer" => Some(SemanticRole::Experiencer),
        "Stimulus" => Some(SemanticRole::Stimulus),
        "Source" => Some(SemanticRole::Source),
        "Goal" => Some(SemanticRole::Goal),
        "Location" => Some(SemanticRole::Location),
        "Instrument" => Some(SemanticRole::Instrument),
        "Beneficiary" => Some(SemanticRole::Beneficiary),
        "Topic" => Some(SemanticRole::Topic),
        "Creator" => Some(SemanticRole::Creator),
        "Created" => Some(SemanticRole::Created),
        "Cognizer" => Some(SemanticRole::Cognizer),
        "Content" => Some(SemanticRole::Content),
        "Speaker" => Some(SemanticRole::Speaker),
        "Message" => Some(SemanticRole::Message),
        _ => None,
    }
}
