use crate::core::graph::{FrameNode, LinguisticGraph};
use crate::core::interlingua::{Frame, SemanticRole};
use crate::core::interlingua::NodeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameBindingQuality {
    ExactOrdinalSignature,
    UniqueSignatureFallback,
    NoGraph,
    MissingFrame,
    AmbiguousFrame,
    RoleMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleEntityBinding {
    pub role: SemanticRole,
    pub occurrence: usize,
    pub graph_entity_node_id: Option<NodeId>,
    pub concept_matches: bool,
    pub name_matches: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticFrameBinding {
    pub graph_frame_node_id: Option<NodeId>,
    pub role_bindings: Vec<RoleEntityBinding>,
    pub binding_quality: FrameBindingQuality,
}

pub struct FrameGraphAdapter<'a> {
    graph: Option<&'a LinguisticGraph>,
}

impl<'a> FrameGraphAdapter<'a> {
    pub fn new(graph: Option<&'a LinguisticGraph>) -> Self {
        Self { graph }
    }

    pub fn bind_frame(&self, frame: &Frame, frame_ordinal: usize) -> SemanticFrameBinding {
        let Some(graph) = self.graph else {
            return SemanticFrameBinding {
                graph_frame_node_id: None,
                role_bindings: Vec::new(),
                binding_quality: FrameBindingQuality::NoGraph,
            };
        };
        let mut frames: Vec<&FrameNode> = graph
            .nodes
            .iter()
            .filter_map(|node| match node {
                crate::core::graph::GraphNode::Frame(frame) => Some(frame),
                _ => None,
            })
            .collect();
        frames.sort_by_key(|node| node.id.0);
        let matched = frames.get(frame_ordinal).copied().or_else(|| {
            frames
                .iter()
                .find(|candidate| candidate.kind == frame.frame_type_name() && candidate.verb_concept.0 == frame_verb_concept(frame))
                .copied()
        });
        let binding_quality = if matched.is_some() {
            if frames.get(frame_ordinal).copied().is_some() {
                FrameBindingQuality::ExactOrdinalSignature
            } else {
                FrameBindingQuality::UniqueSignatureFallback
            }
        } else {
            FrameBindingQuality::MissingFrame
        };
        let role_bindings = frame
            .required_roles()
            .into_iter()
            .enumerate()
            .map(|(occurrence, role)| RoleEntityBinding {
                role,
                occurrence,
                graph_entity_node_id: None,
                concept_matches: false,
                name_matches: false,
            })
            .collect();
        SemanticFrameBinding {
            graph_frame_node_id: matched.map(|frame| frame.id),
            role_bindings,
            binding_quality,
        }
    }
}

fn frame_verb_concept(frame: &Frame) -> String {
    match frame {
        Frame::Transfer { verb_concept, .. }
        | Frame::Motion { verb_concept, .. }
        | Frame::Creation { verb_concept, .. }
        | Frame::Destruction { verb_concept, .. }
        | Frame::Perception { verb_concept, .. }
        | Frame::Cognition { verb_concept, .. }
        | Frame::Emotion { verb_concept, .. }
        | Frame::Communication { verb_concept, .. }
        | Frame::Statement { verb_concept, .. }
        | Frame::Existence { verb_concept, .. }
        | Frame::Possession { verb_concept, .. }
        | Frame::Consumption { verb_concept, .. } => verb_concept.clone(),
        Frame::Custom { name, .. } => name.clone(),
    }
}
