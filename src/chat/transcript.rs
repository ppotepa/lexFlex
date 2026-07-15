use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    EngineFacts,
    EngineEvidence,
    EngineInterlingua,
    EngineEntities,
    EngineQuery,
    EnginePipeline,
    EngineSources,
    EngineSnapshot,
    EngineTrace,
    System,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageBlock {
    Section(String),
    Paragraph(String),
    KeyValue { key: String, value: String },
    BulletList(Vec<String>),
    Notice { tone: String, text: String },
    Timeline(Vec<String>),
    TechnicalList(Vec<String>),
    Trace(Vec<String>),
    Tree(Vec<TranscriptNode>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptNode {
    pub key: String,
    pub label: String,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub tone: Option<String>,
    #[serde(default)]
    pub expanded: bool,
    #[serde(default)]
    pub blocks: Vec<MessageBlock>,
    #[serde(default)]
    pub children: Vec<TranscriptNode>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageMeta {
    pub latency_ms: Option<u128>,
    pub command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptMessage {
    pub turn: usize,
    pub role: MessageRole,
    pub title: String,
    #[serde(default)]
    pub collapsed: bool,
    pub blocks: Vec<MessageBlock>,
    pub meta: MessageMeta,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Transcript {
    pub messages: Vec<TranscriptMessage>,
    pub next_turn: usize,
}

impl Transcript {
    pub fn new() -> Self {
        Self {
            messages: vec![],
            next_turn: 1,
        }
    }

    pub fn clear(&mut self) {
        self.messages.clear();
        self.next_turn = 1;
    }

    pub fn push(
        &mut self,
        role: MessageRole,
        title: impl Into<String>,
        blocks: Vec<MessageBlock>,
        meta: MessageMeta,
    ) {
        self.messages.push(TranscriptMessage {
            turn: self.next_turn,
            role,
            title: title.into(),
            collapsed: false,
            blocks,
            meta,
        });
        self.next_turn += 1;
    }

    pub fn replace(&mut self, turn: usize, title: impl Into<String>, blocks: Vec<MessageBlock>, meta: MessageMeta) -> bool {
        let Some(message) = self.messages.iter_mut().find(|message| message.turn == turn) else { return false; };
        message.title = title.into();
        message.blocks = blocks;
        message.meta = meta;
        true
    }

    pub fn replace_with_role(&mut self, turn: usize, role: MessageRole, title: impl Into<String>, blocks: Vec<MessageBlock>, meta: MessageMeta) -> bool {
        let Some(message) = self.messages.iter_mut().find(|message| message.turn == turn) else { return false; };
        message.role = role;
        message.title = title.into();
        message.blocks = blocks;
        message.meta = meta;
        true
    }

    pub fn append_trace(&mut self, turn: usize, line: String) -> bool {
        let Some(message) = self.messages.iter_mut().find(|message| message.turn == turn) else { return false; };
        if let Some(MessageBlock::Trace(lines)) = message.blocks.iter_mut().find(|block| matches!(block, MessageBlock::Trace(_))) {
            lines.push(line);
        } else {
            message.blocks.push(MessageBlock::Trace(vec![line]));
        }
        true
    }

    pub fn append_tree(&mut self, turn: usize, nodes: Vec<TranscriptNode>) -> bool {
        let Some(message) = self.messages.iter_mut().find(|message| message.turn == turn) else { return false; };
        if let Some(MessageBlock::Tree(existing)) = message.blocks.iter_mut().find(|block| matches!(block, MessageBlock::Tree(_))) {
            existing.extend(nodes);
        } else {
            message.blocks.push(MessageBlock::Tree(nodes));
        }
        true
    }

    pub fn set_collapsed(&mut self, turn: usize, collapsed: bool) -> bool {
        let Some(message) = self.messages.iter_mut().find(|message| message.turn == turn) else { return false; };
        message.collapsed = collapsed;
        true
    }

    pub fn export_plain_text(&self) -> String {
        let mut output = Vec::new();
        for message in &self.messages {
            let header = match message.meta.latency_ms {
                Some(latency) => format!("[{} #{}] {}ms", message.title, message.turn, latency),
                None => format!("[{} #{}]", message.title, message.turn),
            };
            output.push(header);
            if message.collapsed {
                output.push("(collapsed)".into());
                output.push(String::new());
                continue;
            }
            for block in &message.blocks {
                match block {
                    MessageBlock::Section(text) | MessageBlock::Paragraph(text) => output.push(text.clone()),
                    MessageBlock::KeyValue { key, value } => output.push(format!("{key}: {value}")),
                    MessageBlock::BulletList(items) => output.extend(items.iter().map(|item| format!("- {item}"))),
                    MessageBlock::Notice { tone, text } => output.push(format!("[{tone}] {text}")),
                    MessageBlock::Timeline(items) => output.extend(items.iter().map(|item| format!("> {item}"))),
                    MessageBlock::TechnicalList(items) => output.extend(items.iter().map(|item| format!("# {item}"))),
                    MessageBlock::Trace(lines) => output.extend(lines.iter().cloned()),
                    MessageBlock::Tree(nodes) => output.extend(nodes.iter().flat_map(|node| node.export_plain_text(0))),
                }
            }
            output.push(String::new());
        }
        output.join("\n")
    }
}

impl TranscriptNode {
    fn export_plain_text(&self, depth: usize) -> Vec<String> {
        let mut output = Vec::new();
        let indent = "  ".repeat(depth);
        output.push(match &self.summary {
            Some(summary) => format!("{indent}{}: {}", self.label, summary),
            None => format!("{indent}{}", self.label),
        });
        if self.expanded {
            for block in &self.blocks {
                match block {
                    MessageBlock::Section(text) | MessageBlock::Paragraph(text) => output.push(format!("{indent}  {text}")),
                    MessageBlock::KeyValue { key, value } => output.push(format!("{indent}  {key}: {value}")),
                    MessageBlock::BulletList(items) => output.extend(items.iter().map(|item| format!("{indent}  - {item}"))),
                    MessageBlock::Notice { tone, text } => output.push(format!("{indent}  [{tone}] {text}")),
                    MessageBlock::Timeline(items) => output.extend(items.iter().map(|item| format!("{indent}  > {item}"))),
                    MessageBlock::TechnicalList(items) => output.extend(items.iter().map(|item| format!("{indent}  # {item}"))),
                    MessageBlock::Trace(lines) => output.extend(lines.iter().map(|line| format!("{indent}  {line}"))),
                    MessageBlock::Tree(children) => output.extend(children.iter().flat_map(|child| child.export_plain_text(depth + 1))),
                }
            }
            for child in &self.children {
                output.extend(child.export_plain_text(depth + 1));
            }
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::{MessageBlock, MessageMeta, MessageRole, Transcript};

    #[test]
    fn transcript_increments_turns() {
        let mut transcript = Transcript::new();
        transcript.push(
            MessageRole::User,
            "you",
            vec![MessageBlock::Paragraph("hello".to_string())],
            MessageMeta::default(),
        );
        transcript.push(
            MessageRole::Assistant,
            "assistant",
            vec![MessageBlock::Paragraph("hi".to_string())],
            MessageMeta::default(),
        );
        assert_eq!(transcript.messages[0].turn, 1);
        assert_eq!(transcript.messages[1].turn, 2);
        assert_eq!(transcript.next_turn, 3);
    }
}
