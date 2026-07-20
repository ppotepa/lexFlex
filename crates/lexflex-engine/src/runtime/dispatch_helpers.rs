use super::*;
use crate::runtime::error_mapping::request_program_error_response;

impl LexFlexRuntime {
    pub(crate) fn handle_evaluate(
        &self,
        program: LinguaProgram,
        policy: ExecutionPolicy,
        include_trace: bool,
    ) -> EngineResponse {
        match self.lingua.evaluate_with_policy(&program, policy) {
            Ok(mut result) => {
                if !include_trace {
                    result.execution.trace = Default::default();
                }
                EngineResponse::LinguaEvaluated {
                    result: result.execution,
                }
            }
            Err(error) => request_program_error_response(error),
        }
    }

    pub(crate) fn parse_text(
        &self,
        input: &TextInput,
    ) -> Result<ParseOutput, lexflex_parser::ParseError> {
        let language = self.languages.get(&input.language).ok_or_else(|| {
            lexflex_parser::ParseError::UnsupportedLanguage(input.language.clone())
        })?;
        let parser = lexflex_parser::LexicalCompositionParser::with_budget(
            Arc::clone(self.lingua.catalog()),
            Arc::clone(language),
            lexflex_parser::ParseBudget::default(),
        );
        parser.parse(lexflex_parser::ParseInput {
            source_id: input.source_id.clone(),
            language: input.language.clone(),
            text: input.text.clone(),
        })
    }
}
