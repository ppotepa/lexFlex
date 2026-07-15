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
    Trace(Vec<String>),
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

    pub fn replace(&mut self, turn: usize, title: impl Into<String>, blocks: Vec<MessageBlock>, meta: MessageMeta) -> bool {
        let Some(message) = self.messages.iter_mut().find(|message| message.turn == turn) else { return false; };
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
