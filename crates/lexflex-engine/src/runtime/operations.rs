use super::*;
use crate::api::outcome::AssertionWriteOutcome;
use crate::api::response::EngineDiagnostic;
use lexflex_parser::ParseError;

impl LexFlexRuntime {
    pub fn handle(&mut self, request: EngineRequest) -> EngineResponse {
        match request {
            EngineRequest::EvaluateLingua { program, trace } => {
                self.handle_evaluate(program, trace)
            }
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

    fn handle_evaluate(&self, program: LinguaProgram, trace: bool) -> EngineResponse {
        match self.lingua.evaluate_with_policy(
            &program,
            ExecutionPolicy {
                expansion: ExpansionMode::PreserveApplications,
            },
        ) {
            Ok(mut result) => {
                if !trace {
                    result.trace = Default::default();
                }
                EngineResponse::LinguaEvaluated { result }
            }
            Err(error) => EngineResponse::Error {
                code: EngineErrorCode::InvalidProgram,
                message: error.to_string(),
                diagnostics: Vec::new(),
            },
        }
    }

    fn handle_ingest(&mut self, program: LinguaProgram, evidence: Vec<Evidence>) -> EngineResponse {
        let result = match self.lingua.evaluate_with_policy(
            &program,
            ExecutionPolicy {
                expansion: ExpansionMode::PreserveApplications,
            },
        ) {
            Ok(result) => result,
            Err(error) => {
                return EngineResponse::Error {
                    code: EngineErrorCode::InvalidProgram,
                    message: error.to_string(),
                    diagnostics: Vec::new(),
                };
            }
        };

        let assertion =
            SemanticAssertion::create(result.value, evidence, WorldId::new_unchecked("actual"));
        let assertion_id = assertion.id.clone();
        let before = self.session.state.clone();
        let outcome = match self.session.state.knowledge.upsert(assertion) {
            crate::knowledge::UpsertOutcome::Inserted { assertion_id } => {
                AssertionWriteOutcome::Inserted { assertion_id }
            }
            crate::knowledge::UpsertOutcome::EvidenceMerged {
                assertion_id,
                added,
            } => AssertionWriteOutcome::EvidenceMerged {
                assertion_id,
                added_evidence: added,
            },
            crate::knowledge::UpsertOutcome::Unchanged { assertion_id } => {
                AssertionWriteOutcome::Unchanged { assertion_id }
            }
        };
        self.session.state.rebuild_knowledge();
        self.session.knowledge_index =
            crate::knowledge::KnowledgeIndex::rebuild(&self.session.state.knowledge);
        if let Err(error) = self
            .store
            .save(&self.session.state.session_id, &self.session.state)
        {
            self.session.state = before;
            return EngineResponse::Error {
                code: EngineErrorCode::StoreError,
                message: error.to_string(),
                diagnostics: Vec::new(),
            };
        }
        let Some(assertion) = self
            .session
            .state
            .knowledge
            .assertions
            .get(&assertion_id)
            .cloned()
        else {
            return EngineResponse::Error {
                code: EngineErrorCode::InternalInvariant,
                message: format!("missing assertion after upsert: {assertion_id}"),
                diagnostics: vec![EngineDiagnostic::MissingAssertion {
                    assertion_id: assertion_id.clone(),
                }],
            };
        };
        EngineResponse::LinguaIngested {
            assertion,
            outcome,
            snapshot_hash: self.session.state.snapshot_hash().to_string(),
        }
    }

    fn handle_query(&self, goal: LinguaGoal) -> EngineResponse {
        let candidates = match self.candidate_assertions(&goal) {
            Ok(candidates) => candidates,
            Err(assertion_id) => {
                return EngineResponse::Error {
                    code: EngineErrorCode::InternalInvariant,
                    message: format!("missing indexed assertion: {assertion_id}"),
                    diagnostics: vec![EngineDiagnostic::MissingAssertion { assertion_id }],
                };
            }
        };
        match self.lingua.solve(&goal, candidates) {
            Ok(solutions) => EngineResponse::LinguaQueryResult {
                goal,
                solutions,
                snapshot_hash: self.session.state.snapshot_hash().to_string(),
            },
            Err(error) => EngineResponse::Error {
                code: EngineErrorCode::InvalidGoal,
                message: error.to_string(),
                diagnostics: Vec::new(),
            },
        }
    }

    fn handle_analyze_text(&self, input: TextInput, include_derivation: bool) -> EngineResponse {
        match self.parse_text(&input) {
            Ok(ParseOutput::Assertion(draft)) => {
                let analysis = self.assertion_analysis(&draft, include_derivation);
                EngineResponse::TextAnalyzed { analysis }
            }
            Ok(ParseOutput::Goal(draft)) => {
                let analysis = self.goal_analysis(&draft, include_derivation);
                EngineResponse::TextAnalyzed { analysis }
            }
            Ok(ParseOutput::Ambiguous { alternatives }) => EngineResponse::TextAmbiguous {
                alternatives: alternatives
                    .into_iter()
                    .map(|alternative| {
                        TextAnalysisAlternative::new(
                            alternative.expression,
                            include_derivation.then_some(alternative.derivation),
                            alternative.score,
                        )
                    })
                    .collect(),
            },
            Err(error) => EngineResponse::TextNotParsed {
                diagnostics: vec![error],
            },
        }
    }

    fn handle_ingest_text(&mut self, input: TextInput) -> EngineResponse {
        let parse = match self.parse_text(&input) {
            Ok(ParseOutput::Assertion(draft)) => draft,
            Ok(ParseOutput::Goal(_)) => {
                return EngineResponse::TextNotParsed {
                    diagnostics: vec![ParseError::NoParse],
                }
            }
            Ok(ParseOutput::Ambiguous { alternatives }) => {
                return EngineResponse::TextAmbiguous {
                    alternatives: alternatives
                        .into_iter()
                        .map(|alternative| {
                            TextAnalysisAlternative::new(
                                alternative.expression,
                                Some(alternative.derivation),
                                alternative.score,
                            )
                        })
                        .collect(),
                }
            }
            Err(error) => {
                return EngineResponse::TextNotParsed {
                    diagnostics: vec![error],
                }
            }
        };

        let analysis = self.assertion_analysis(&parse, false);
        let semantic_expression = match self.evaluate_text_expression(&parse.expression) {
            Ok(value) => value,
            Err(error) => {
                return EngineResponse::Error {
                    code: EngineErrorCode::InvalidProgram,
                    message: error,
                    diagnostics: Vec::new(),
                }
            }
        };
        let source_hash = canonical_hash(&input.text);
        let evidence = Evidence {
            id: EvidenceId::new_unchecked(format!("evidence:{source_hash}")),
            source_id: input.source_id.clone(),
            span: Some(SourceSpan {
                start: 0,
                end: input.text.len() as u64,
            }),
            source_hash: Some(source_hash),
        };
        let assertion = SemanticAssertion::create(
            semantic_expression,
            vec![evidence],
            WorldId::new_unchecked("actual"),
        );
        let assertion_id = assertion.id.clone();
        let before = self.session.state.clone();
        let outcome = match self.session.state.knowledge.upsert(assertion) {
            crate::knowledge::UpsertOutcome::Inserted { assertion_id } => {
                AssertionWriteOutcome::Inserted { assertion_id }
            }
            crate::knowledge::UpsertOutcome::EvidenceMerged {
                assertion_id,
                added,
            } => AssertionWriteOutcome::EvidenceMerged {
                assertion_id,
                added_evidence: added,
            },
            crate::knowledge::UpsertOutcome::Unchanged { assertion_id } => {
                AssertionWriteOutcome::Unchanged { assertion_id }
            }
        };
        self.session.state.rebuild_knowledge();
        self.session.knowledge_index =
            crate::knowledge::KnowledgeIndex::rebuild(&self.session.state.knowledge);
        if let Err(error) = self
            .store
            .save(&self.session.state.session_id, &self.session.state)
        {
            self.session.state = before;
            return EngineResponse::Error {
                code: EngineErrorCode::StoreError,
                message: error.to_string(),
                diagnostics: Vec::new(),
            };
        }
        let Some(assertion) = self
            .session
            .state
            .knowledge
            .assertions
            .get(&assertion_id)
            .cloned()
        else {
            return EngineResponse::Error {
                code: EngineErrorCode::InternalInvariant,
                message: format!("missing assertion after upsert: {assertion_id}"),
                diagnostics: vec![EngineDiagnostic::MissingAssertion {
                    assertion_id: assertion_id.clone(),
                }],
            };
        };
        EngineResponse::TextIngested {
            analysis,
            assertion,
            outcome,
            snapshot_hash: self.session.state.snapshot_hash().to_string(),
        }
    }

    fn handle_ask_text(
        &self,
        input: TextInput,
        evidence_policy: EvidencePolicy,
        limit: Option<usize>,
    ) -> EngineResponse {
        let parse = match self.parse_text(&input) {
            Ok(ParseOutput::Goal(draft)) => draft,
            Ok(ParseOutput::Assertion(_)) => {
                return EngineResponse::TextNotParsed {
                    diagnostics: vec![ParseError::QuestionWithoutProjection],
                }
            }
            Ok(ParseOutput::Ambiguous { alternatives }) => {
                return EngineResponse::TextAmbiguous {
                    alternatives: alternatives
                        .into_iter()
                        .map(|alternative| {
                            TextAnalysisAlternative::new(
                                alternative.expression,
                                Some(alternative.derivation),
                                alternative.score,
                            )
                        })
                        .collect(),
                }
            }
            Err(error) => {
                return EngineResponse::TextNotParsed {
                    diagnostics: vec![error],
                }
            }
        };

        let analysis = self.goal_analysis(&parse, false);
        let semantic_expression = match self.evaluate_text_expression(&parse.expression) {
            Ok(value) => value,
            Err(error) => {
                return EngineResponse::Error {
                    code: EngineErrorCode::InvalidProgram,
                    message: error,
                    diagnostics: Vec::new(),
                }
            }
        };
        let goal = LinguaGoal {
            expression: semantic_expression,
            variables: parse.variables.clone(),
            projection: parse.projection.clone(),
            evidence_policy,
            world: None,
            limit,
        };
        let candidates = match self.candidate_assertions(&goal) {
            Ok(candidates) => candidates,
            Err(assertion_id) => {
                return EngineResponse::Error {
                    code: EngineErrorCode::InternalInvariant,
                    message: format!("missing indexed assertion: {assertion_id}"),
                    diagnostics: vec![EngineDiagnostic::MissingAssertion { assertion_id }],
                };
            }
        };
        match self.lingua.solve(&goal, candidates) {
            Ok(solutions) => EngineResponse::TextAnswer {
                analysis,
                goal,
                solutions,
                snapshot_hash: self.session.state.snapshot_hash().to_string(),
            },
            Err(error) => EngineResponse::Error {
                code: EngineErrorCode::InvalidGoal,
                message: error.to_string(),
                diagnostics: Vec::new(),
            },
        }
    }

    fn handle_inspect(&self) -> EngineResponse {
        EngineResponse::SessionInspection {
            assertion_count: self.session.state.knowledge.assertions.len(),
            evidence_count: self.session.state.evidence_count(),
            snapshot_hash: self.session.state.snapshot_hash().to_string(),
            model_hash: self.session.state.model_hash.clone(),
            language_hash: self.languages.registry_hash.clone(),
        }
    }

    fn handle_clear(&mut self) -> EngineResponse {
        let before = self.session.state.clone();
        self.session.state.knowledge.assertions.clear();
        self.session.state.rebuild_knowledge();
        self.session.knowledge_index =
            crate::knowledge::KnowledgeIndex::rebuild(&self.session.state.knowledge);
        if let Err(error) = self
            .store
            .save(&self.session.state.session_id, &self.session.state)
        {
            self.session.state = before;
            return EngineResponse::Error {
                code: EngineErrorCode::StoreError,
                message: error.to_string(),
                diagnostics: Vec::new(),
            };
        }
        EngineResponse::SessionCleared {
            snapshot_hash: self.session.state.snapshot_hash().to_string(),
        }
    }

    fn parse_text(&self, input: &TextInput) -> Result<ParseOutput, ParseError> {
        let language = self
            .languages
            .get(&input.language)
            .ok_or_else(|| ParseError::UnsupportedLanguage(input.language.clone()))?;
        let parser =
            LexicalCompositionParser::with_budget(Arc::clone(language), ParseBudget::default());
        parser.parse(ParseInput {
            source_id: input.source_id.clone(),
            language: input.language.clone(),
            text: input.text.clone(),
        })
    }

    fn evaluate_text_expression(
        &self,
        expression: &lexflex_lingua::LinguaExpression,
    ) -> Result<SemanticExpression, String> {
        evaluate_text_expression(expression)
    }

    fn assertion_analysis(
        &self,
        draft: &lexflex_parser::AssertionDraft,
        include_derivation: bool,
    ) -> TextAnalysis {
        TextAnalysis::new(
            draft.source_id.clone(),
            draft.language.clone(),
            TextAnalysisKind::Assertion,
            draft.expression.clone(),
            Default::default(),
            Vec::new(),
            include_derivation.then(|| draft.derivation.clone()),
        )
    }

    fn goal_analysis(
        &self,
        draft: &lexflex_parser::GoalDraft,
        include_derivation: bool,
    ) -> TextAnalysis {
        TextAnalysis::new(
            draft.source_id.clone(),
            draft.language.clone(),
            TextAnalysisKind::Goal,
            draft.expression.clone(),
            draft.variables.clone(),
            draft.projection.clone(),
            include_derivation.then(|| draft.derivation.clone()),
        )
    }

    fn candidate_assertions<'a>(
        &'a self,
        goal: &LinguaGoal,
    ) -> Result<Vec<&'a SemanticAssertion>, lexflex_model::AssertionId> {
        let candidate_ids = self
            .session
            .knowledge_index
            .candidate_ids(goal, &self.session.state.knowledge);
        candidate_ids
            .into_iter()
            .map(|id| self.session.state.knowledge.assertions.get(&id).ok_or(id))
            .collect()
    }
}

pub(crate) fn workspace_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .join(relative)
}
