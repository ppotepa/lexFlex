use crate::core::capability::{Capability, InexpressibleFeature, Limitation};
use crate::core::interlingua::{Interlingua, LanguageId};
use crate::error::{GenerateError, ParseError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanguageKind {
    Natural,
    Formal,
    Programming,
    Domain,
}

pub trait IMeaningRepresentation {
    type Input: ?Sized;
    type Output;

    fn language_id(&self) -> &LanguageId;
    fn name(&self) -> &str;
    fn language_kind(&self) -> LanguageKind;
    fn capabilities(&self) -> &[Capability];
    fn limitations(&self) -> &[Limitation];

    fn to_interlingua(&self, input: &Self::Input) -> Result<Interlingua, ParseError>;
    fn from_interlingua(&self, il: &Interlingua) -> Result<Self::Output, GenerateError>;

    fn can_express(&self, il: &Interlingua) -> Vec<InexpressibleFeature> {
        let required = il.required_capabilities();
        let available = self.capabilities();

        required
            .iter()
            .filter(|req_cap| !available.contains(req_cap))
            .map(|cap| InexpressibleFeature {
                capability: *cap,
                suggestion: Some(format!(
                    "{} cannot express {:?}",
                    self.name(),
                    cap
                )),
            })
            .collect()
    }
}

impl Interlingua {
    pub fn required_capabilities(&self) -> Vec<Capability> {
        use crate::core::interlingua::*;

        let mut caps = vec![Capability::Reference];

        match self {
            Interlingua::Natural(utterance) => {
                // Check discourse features
                if utterance.discourse.is_some() {
                    caps.push(Capability::Pragmatics);
                }

                // Check sentence-level features
                for sentence in &utterance.sentences {
                    // Temporal features
                    if sentence.tense.is_some() || sentence.temporal.is_some() {
                        if !caps.contains(&Capability::TemporalReference) {
                            caps.push(Capability::TemporalReference);
                        }
                    }

                    // Negation
                    if sentence.polarity == Polarity::Negative {
                        if !caps.contains(&Capability::Negation) {
                            caps.push(Capability::Negation);
                        }
                    }

                    // Questions and commands
                    match sentence.illocution {
                        Illocution::Question => {
                            if !caps.contains(&Capability::Coordination) {
                                caps.push(Capability::Coordination);
                            }
                        }
                        Illocution::Command => {
                            // Commands require specific capability
                        }
                        _ => {}
                    }

                    // Quantification
                    if sentence.quantification.is_some() {
                        if !caps.contains(&Capability::Quantification) {
                            caps.push(Capability::Quantification);
                        }
                    }

                    // Deictic temporal references
                    if let Some(TemporalReference::Deictic { .. }) = &sentence.temporal {
                        if !caps.contains(&Capability::Deixis) {
                            caps.push(Capability::Deixis);
                        }
                    }

                    // Check frame-specific features
                    for frame in &sentence.frames {
                        Self::add_frame_capabilities(frame, &mut caps);
                    }
                }
            }
            Interlingua::MathExpression(_) => {
                caps.push(Capability::NumericPrecision);
                caps.push(Capability::LogicalConnectives);
            }
            Interlingua::LogicalProposition(_) => {
                caps.push(Capability::FormalProof);
                caps.push(Capability::LogicalConnectives);
            }
            Interlingua::ProgramStatement(_) => {
                caps.push(Capability::Procedures);
                caps.push(Capability::ControlFlow);
            }
        }

        caps
    }

    fn add_frame_capabilities(frame: &crate::core::interlingua::Frame, caps: &mut Vec<Capability>) {
        use crate::core::interlingua::Frame;

        match frame {
            Frame::Emotion { .. } => {
                if !caps.contains(&Capability::EmotionExpression) {
                    caps.push(Capability::EmotionExpression);
                }
            }
            Frame::Transfer { .. } | Frame::Motion { .. } => {
                // These are basic capabilities already covered by Reference
            }
            _ => {}
        }
    }
}
