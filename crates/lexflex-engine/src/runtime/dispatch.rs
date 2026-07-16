use super::*;

impl LexFlexRuntime {
    pub fn handle(&mut self, request: EngineRequest) -> EngineResponse {
        match request {
            EngineRequest::EvaluateLingua {
                program,
                policy,
                include_trace,
            } => self.handle_evaluate(program, policy, include_trace),
            EngineRequest::IngestLingua { program, evidence } => {
                self.handle_ingest(program, evidence)
            }
            EngineRequest::QueryLingua { goal } => self.handle_query(goal),
            EngineRequest::AnalyzeText {
                input,
                include_derivation,
            } => self.handle_analyze_text(input, include_derivation),
            EngineRequest::IngestText { input } => self.handle_ingest_text(input),
            EngineRequest::AskText {
                input,
                evidence_policy,
                limit,
            } => self.handle_ask_text(input, evidence_policy, limit),
            EngineRequest::InspectSession => self.handle_inspect(),
            EngineRequest::ClearSession => self.handle_clear(),
            EngineRequest::TranslateText { .. } => EngineResponse::Unsupported {
                capability: "natural-language-translation".into(),
                message: "Translation is not implemented yet.".into(),
            },
        }
    }
}
