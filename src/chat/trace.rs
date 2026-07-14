use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatTraceEntry {
    pub turn: usize,
    pub input: String,
    pub language: String,
    pub parse_status: String,
    pub sentence_count: usize,
    pub sentences: Vec<String>,
    pub question: Option<String>,
    pub facts_added: usize,
    pub answer_status: Option<String>,
    pub answer: Option<String>,
    pub evidence: Vec<String>,
    pub fallback: Option<String>,
}

impl ChatTraceEntry {
    pub fn as_notes(&self) -> Vec<String> {
        let mut notes = vec![format!(
            "trace: language={} parse={} sentences={}",
            self.language, self.parse_status, self.sentence_count
        )];
        notes.extend(self.sentences.iter().map(|line| format!("trace: {line}")));
        if let Some(question) = &self.question {
            notes.push(format!("trace: question={question}"));
        }
        if self.facts_added > 0 {
            notes.push(format!("trace: facts_added={}", self.facts_added));
        }
        if let Some(status) = &self.answer_status {
            notes.push(format!("trace: answer_status={status}"));
        }
        if let Some(fallback) = &self.fallback {
            notes.push(format!("trace: fallback={fallback}"));
        }
        notes
    }
}
