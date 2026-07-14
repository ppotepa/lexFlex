use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageBlock {
    Paragraph(String),
    KeyValue { key: String, value: String },
    BulletList(Vec<String>),
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
            blocks,
            meta,
        });
        self.next_turn += 1;
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
