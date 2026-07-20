use crate::fixtures;
use crate::report::{BenchmarkReport, BenchmarkResult, ParserMetricsSnapshot};
use clap::ValueEnum;
use lexflex_engine::api::{input::TextInput, text::TextAnalysis};
use lexflex_engine::{EngineRequest, EngineResponse, LexFlexRuntime};
use lexflex_lingua::solve::LinguaSolver;
use lexflex_lingua::{
    ExecutionPolicy, ExpansionMode, LinguaCompiler, LinguaInterpreter, LinguaProgram,
};
use lexflex_model::{canonical_hash, ConceptCatalog, SemanticAssertion};
use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BenchmarkError {
    #[error("benchmark operation failed: {0}")]
    Operation(String),
    #[error("benchmark output was not deterministic")]
    Nondeterministic,
}

impl From<Box<dyn std::error::Error>> for BenchmarkError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        Self::Operation(error.to_string())
    }
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum BenchmarkCase {
    Entity,
    Lambda,
    ConceptApplication,
    DefinedConceptExpansion,
    GlobalFunction,
    SolverSingle,
    SolverThousand,
    AnalyzeCapitalEn,
    AnalyzeCapitalPl,
    AnalyzeEventEn,
    AnalyzeEventPl,
    AskCapital,
    AskEvent,
}

impl BenchmarkCase {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Entity,
            Self::Lambda,
            Self::ConceptApplication,
            Self::DefinedConceptExpansion,
            Self::GlobalFunction,
            Self::SolverSingle,
            Self::SolverThousand,
            Self::AnalyzeCapitalEn,
            Self::AnalyzeCapitalPl,
            Self::AnalyzeEventEn,
            Self::AnalyzeEventPl,
            Self::AskCapital,
            Self::AskEvent,
        ]
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Entity => "entity",
            Self::Lambda => "lambda",
            Self::ConceptApplication => "concept-application",
            Self::DefinedConceptExpansion => "defined-concept-expansion",
            Self::GlobalFunction => "global-function",
            Self::SolverSingle => "solver-single",
            Self::SolverThousand => "solver-thousand",
            Self::AnalyzeCapitalEn => "analyze-capital-en",
            Self::AnalyzeCapitalPl => "analyze-capital-pl",
            Self::AnalyzeEventEn => "analyze-event-en",
            Self::AnalyzeEventPl => "analyze-event-pl",
            Self::AskCapital => "ask-capital",
            Self::AskEvent => "ask-event",
        }
    }
}

pub struct BenchmarkSuite {
    catalog: Arc<ConceptCatalog>,
}

impl BenchmarkSuite {
    pub fn new(catalog: Arc<ConceptCatalog>) -> Self {
        Self { catalog }
    }

    pub fn run_all(
        &self,
        cases: &[BenchmarkCase],
        iterations: usize,
    ) -> Result<BenchmarkReport, BenchmarkError> {
        let cases = if cases.is_empty() {
            BenchmarkCase::all()
        } else {
            cases.to_vec()
        };
        Ok(BenchmarkReport {
            cases: cases
                .into_iter()
                .map(|case| self.run(case, iterations))
                .collect::<Result<_, _>>()?,
        })
    }

    pub fn run(
        &self,
        case: BenchmarkCase,
        iterations: usize,
    ) -> Result<BenchmarkResult, BenchmarkError> {
        match case {
            BenchmarkCase::Entity => self.run_compiled_case(
                case,
                iterations,
                vec!["compile", "execute"],
                fixtures::entity_program(),
                ExecutionPolicy::default(),
            ),
            BenchmarkCase::Lambda => self.run_compiled_case(
                case,
                iterations,
                vec!["compile", "execute"],
                fixtures::lambda_program(),
                ExecutionPolicy::default(),
            ),
            BenchmarkCase::ConceptApplication => self.run_compiled_case(
                case,
                iterations,
                vec!["compile", "execute"],
                fixtures::concept_application_program(false),
                ExecutionPolicy::default(),
            ),
            BenchmarkCase::DefinedConceptExpansion => self.run_compiled_case(
                case,
                iterations,
                vec!["compile", "execute", "expand"],
                fixtures::concept_application_program(true),
                ExecutionPolicy {
                    expansion: ExpansionMode::ExpandTransparent,
                },
            ),
            BenchmarkCase::GlobalFunction => self.run_compiled_case(
                case,
                iterations,
                vec!["compile", "execute", "closure-call"],
                fixtures::global_function_program(),
                ExecutionPolicy::default(),
            ),
            BenchmarkCase::SolverSingle => self.run_solver_case(
                case,
                iterations,
                vec!["build goal", "solve against one assertion"],
                fixtures::solver_single_goal(),
                fixtures::solver_single_assertions()?,
            ),
            BenchmarkCase::SolverThousand => self.run_solver_case(
                case,
                iterations,
                vec!["build goal", "solve against 1000 assertions"],
                fixtures::solver_single_goal(),
                fixtures::solver_thousand_assertions()?,
            ),
            BenchmarkCase::AnalyzeCapitalEn => self.run_runtime_case(
                case,
                iterations,
                vec!["parse", "compile", "execute"],
                fixtures::analyze_input(
                    "bench:capital:en",
                    "en",
                    "Paris is the capital of France.",
                )?,
            ),
            BenchmarkCase::AnalyzeCapitalPl => self.run_runtime_case(
                case,
                iterations,
                vec!["parse", "compile", "execute"],
                fixtures::analyze_input("bench:capital:pl", "pl", "Paryż jest stolicą Francji.")?,
            ),
            BenchmarkCase::AnalyzeEventEn => self.run_runtime_case(
                case,
                iterations,
                vec!["parse", "compile", "execute"],
                fixtures::analyze_input("bench:event:en", "en", "Tom sees Iza.")?,
            ),
            BenchmarkCase::AnalyzeEventPl => self.run_runtime_case(
                case,
                iterations,
                vec!["parse", "compile", "execute"],
                fixtures::analyze_input("bench:event:pl", "pl", "Tomek widzi Izę.")?,
            ),
            BenchmarkCase::AskCapital => self.run_stateful_runtime_case(
                case,
                iterations,
                vec!["ingest", "ask", "solve"],
                |runtime| {
                    black_box(runtime.handle(EngineRequest::IngestText {
                        input: fixtures::analyze_input(
                            "bench:capital:ingest",
                            "en",
                            "Paris is the capital of France.",
                        )?,
                    }));
                    Ok(runtime.handle(EngineRequest::AskText {
                        input: fixtures::analyze_input(
                            "bench:capital:ask",
                            "en",
                            "What is the capital of France?",
                        )?,
                        evidence_policy: lexflex_lingua::EvidencePolicy::Required,
                        limit: Some(1),
                    }))
                },
            ),
            BenchmarkCase::AskEvent => self.run_stateful_runtime_case(
                case,
                iterations,
                vec!["ingest", "ask", "solve"],
                |runtime| {
                    black_box(runtime.handle(EngineRequest::IngestText {
                        input: fixtures::analyze_input(
                            "bench:event:ingest",
                            "en",
                            "Tom sees Iza.",
                        )?,
                    }));
                    Ok(runtime.handle(EngineRequest::AskText {
                        input: fixtures::analyze_input("bench:event:ask", "pl", "Kto widzi Izę?")?,
                        evidence_policy: lexflex_lingua::EvidencePolicy::Required,
                        limit: Some(1),
                    }))
                },
            ),
        }
    }

    fn run_compiled_case(
        &self,
        case: BenchmarkCase,
        iterations: usize,
        steps: Vec<&'static str>,
        program: LinguaProgram,
        policy: ExecutionPolicy,
    ) -> Result<BenchmarkResult, BenchmarkError> {
        let compiler = LinguaCompiler::try_new(Arc::clone(&self.catalog))
            .map_err(|error| BenchmarkError::Operation(error.to_string()))?;
        let compiled = compiler
            .compile(&program)
            .map_err(|error| BenchmarkError::Operation(error.to_string()))?;
        let interpreter = LinguaInterpreter::with_policy(Default::default(), policy);
        let mut last_hash = None;
        warmup(10, || {
            interpreter
                .execute(&compiled)
                .map(|result| {
                    black_box(result);
                })
                .map_err(|error| BenchmarkError::Operation(error.to_string()))
                .map(|_| ())
        })?;
        let samples = measure(iterations, || {
            let result = interpreter
                .execute(&compiled)
                .map_err(|error| BenchmarkError::Operation(error.to_string()))?;
            let hash = canonical_hash(&result.execution.value)
                .map_err(|error| BenchmarkError::Operation(error.to_string()))?
                .to_string();
            if let Some(existing) = &last_hash {
                if existing != &hash {
                    return Err(BenchmarkError::Nondeterministic);
                }
            }
            last_hash = Some(hash);
            black_box(result);
            Ok(())
        })?;
        Ok(BenchmarkResult::from_samples(
            case,
            iterations,
            samples,
            steps,
            last_hash.unwrap_or_default(),
            None,
        ))
    }

    fn run_solver_case(
        &self,
        case: BenchmarkCase,
        iterations: usize,
        steps: Vec<&'static str>,
        goal: lexflex_lingua::LinguaGoal,
        assertions: Vec<SemanticAssertion>,
    ) -> Result<BenchmarkResult, BenchmarkError> {
        let solver = LinguaSolver::default();
        let mut last_hash = None;
        warmup(10, || {
            solver
                .solve(&goal, assertions.iter(), Arc::clone(&self.catalog))
                .map(|result| {
                    black_box(result);
                })
                .map_err(|error| BenchmarkError::Operation(error.to_string()))
                .map(|_| ())
        })?;
        let samples = measure(iterations, || {
            let results = solver
                .solve(&goal, assertions.iter(), Arc::clone(&self.catalog))
                .map_err(|error| BenchmarkError::Operation(error.to_string()))?;
            let hash = canonical_hash(&results)
                .map_err(|error| BenchmarkError::Operation(error.to_string()))?
                .to_string();
            if let Some(existing) = &last_hash {
                if existing != &hash {
                    return Err(BenchmarkError::Nondeterministic);
                }
            }
            last_hash = Some(hash);
            black_box(results.len());
            Ok(())
        })?;
        Ok(BenchmarkResult::from_samples(
            case,
            iterations,
            samples,
            steps,
            last_hash.unwrap_or_default(),
            None,
        ))
    }

    fn run_runtime_case(
        &self,
        case: BenchmarkCase,
        iterations: usize,
        steps: Vec<&'static str>,
        input: TextInput,
    ) -> Result<BenchmarkResult, BenchmarkError> {
        let mut runtime = fixtures::runtime(case.label())?;
        let mut last_hash = None;
        let mut parser_metrics = None;
        warmup(10, || {
            black_box(runtime.handle(EngineRequest::AnalyzeText {
                input: input.clone(),
                include_derivation: true,
            }));
            Ok(())
        })?;
        let samples = measure(iterations, || {
            let response = runtime.handle(EngineRequest::AnalyzeText {
                input: input.clone(),
                include_derivation: true,
            });
            let hash = response_hash(&response)?;
            if let Some(existing) = &last_hash {
                if existing != &hash {
                    return Err(BenchmarkError::Nondeterministic);
                }
            }
            if parser_metrics.is_none() {
                parser_metrics = extract_parser_metrics(&response);
            }
            last_hash = Some(hash);
            black_box(response);
            Ok(())
        })?;
        Ok(BenchmarkResult::from_samples(
            case,
            iterations,
            samples,
            steps,
            last_hash.unwrap_or_default(),
            parser_metrics,
        ))
    }

    fn run_stateful_runtime_case(
        &self,
        case: BenchmarkCase,
        iterations: usize,
        steps: Vec<&'static str>,
        mut f: impl FnMut(&mut LexFlexRuntime) -> Result<EngineResponse, Box<dyn std::error::Error>>,
    ) -> Result<BenchmarkResult, BenchmarkError> {
        let mut last_hash = None;
        warmup(10, || {
            let mut runtime = fixtures::runtime(case.label())?;
            black_box(f(&mut runtime)?);
            Ok(())
        })?;
        let samples = measure(iterations, || {
            let mut runtime = fixtures::runtime(case.label())?;
            let response =
                f(&mut runtime).map_err(|error| BenchmarkError::Operation(error.to_string()))?;
            let hash = response_hash(&response)?;
            if let Some(existing) = &last_hash {
                if existing != &hash {
                    return Err(BenchmarkError::Nondeterministic);
                }
            }
            last_hash = Some(hash);
            black_box(response);
            Ok(())
        })?;
        Ok(BenchmarkResult::from_samples(
            case,
            iterations,
            samples,
            steps,
            last_hash.unwrap_or_default(),
            None,
        ))
    }
}

fn response_hash(response: &EngineResponse) -> Result<String, BenchmarkError> {
    canonical_hash(response)
        .map(|hash| hash.to_string())
        .map_err(|error| BenchmarkError::Operation(error.to_string()))
}

fn extract_parser_metrics(response: &EngineResponse) -> Option<ParserMetricsSnapshot> {
    match response {
        EngineResponse::TextAnalyzed { analysis, .. } => Some(snapshot_metrics(analysis)),
        _ => None,
    }
}

fn snapshot_metrics(analysis: &TextAnalysis) -> ParserMetricsSnapshot {
    ParserMetricsSnapshot {
        token_count: analysis.parser_metrics().token_count,
        lexical_candidate_count: analysis.parser_metrics().lexical_candidate_count,
        chart_item_count: analysis.parser_metrics().chart_item_count,
        chart_replacement_count: analysis.parser_metrics().chart_replacement_count,
        semantic_duplicate_count: analysis.parser_metrics().semantic_duplicate_count,
        rejected_application_count: analysis.parser_metrics().rejected_application_count,
        complete_semantic_count: analysis.parser_metrics().complete_semantic_count,
        max_cell_size: analysis.parser_metrics().max_cell_size,
        derivation_depth: analysis
            .derivations()
            .as_ref()
            .map(|derivations| derivations.primary().depth())
            .unwrap_or_default(),
        max_semantic_nodes: analysis.parser_metrics().max_semantic_nodes,
    }
}

fn warmup(
    mut iterations: usize,
    mut f: impl FnMut() -> Result<(), BenchmarkError>,
) -> Result<(), BenchmarkError> {
    while iterations > 0 {
        f()?;
        iterations -= 1;
    }
    Ok(())
}

fn measure(
    mut iterations: usize,
    mut f: impl FnMut() -> Result<(), BenchmarkError>,
) -> Result<Vec<u128>, BenchmarkError> {
    let mut samples = Vec::with_capacity(iterations);
    while iterations > 0 {
        let start = Instant::now();
        f()?;
        samples.push(start.elapsed().as_nanos());
        iterations -= 1;
    }
    Ok(samples)
}

#[cfg(test)]
mod tests;
