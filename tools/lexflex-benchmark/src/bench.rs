use crate::fixtures;
use clap::ValueEnum;
use lexflex_lingua::solve::LinguaSolver;
use lexflex_lingua::{
    ExecutionPolicy, ExpansionMode, LinguaCompiler, LinguaInterpreter, LinguaProgram,
};
use lexflex_model::{ConceptCatalog, SemanticAssertion};
use serde::Serialize;
use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum BenchmarkCase {
    Entity,
    Lambda,
    ConceptApplication,
    DefinedConceptExpansion,
    GlobalFunction,
    SolverSingle,
    SolverThousand,
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
        }
    }
}

#[derive(Debug, Serialize)]
pub struct BenchmarkReport {
    pub cases: Vec<BenchmarkResult>,
}

#[derive(Debug, Serialize)]
pub struct BenchmarkResult {
    pub case: String,
    pub iterations: usize,
    pub total_ns: u128,
    pub avg_ns: u128,
    pub min_ns: u128,
    pub median_ns: u128,
    pub p95_ns: u128,
    pub max_ns: u128,
    pub steps: Vec<String>,
}

pub struct BenchmarkSuite {
    catalog: Arc<ConceptCatalog>,
}

impl BenchmarkSuite {
    pub fn new(catalog: Arc<ConceptCatalog>) -> Self {
        Self { catalog }
    }

    pub fn run_all(&self, cases: &[BenchmarkCase], iterations: usize) -> BenchmarkReport {
        let cases = if cases.is_empty() {
            BenchmarkCase::all()
        } else {
            cases.to_vec()
        };
        BenchmarkReport {
            cases: cases
                .into_iter()
                .map(|case| self.run(case, iterations))
                .collect(),
        }
    }

    pub fn run(&self, case: BenchmarkCase, iterations: usize) -> BenchmarkResult {
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
                fixtures::solver_single_assertions(),
            ),
            BenchmarkCase::SolverThousand => self.run_solver_case(
                case,
                iterations,
                vec!["build goal", "solve against 1000 assertions"],
                fixtures::solver_single_goal(),
                fixtures::solver_thousand_assertions(),
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
    ) -> BenchmarkResult {
        let compiler = LinguaCompiler::new(Arc::clone(&self.catalog));
        let compiled = compiler
            .compile(&program)
            .expect("benchmark program compiles");
        let interpreter = LinguaInterpreter::with_policy(Default::default(), policy);
        let samples = measure(iterations, || {
            let result = interpreter
                .execute(&compiled)
                .expect("benchmark program executes");
            black_box(result);
        });
        BenchmarkResult::from_samples(case, iterations, samples, steps)
    }

    fn run_solver_case(
        &self,
        case: BenchmarkCase,
        iterations: usize,
        steps: Vec<&'static str>,
        goal: lexflex_lingua::LinguaGoal,
        assertions: Vec<SemanticAssertion>,
    ) -> BenchmarkResult {
        let solver = LinguaSolver::default();
        let samples = measure(iterations, || {
            let results = solver
                .solve(&goal, assertions.iter(), Arc::clone(&self.catalog))
                .expect("benchmark goal solves");
            black_box(results.len());
        });
        BenchmarkResult::from_samples(case, iterations, samples, steps)
    }
}

impl BenchmarkResult {
    fn from_samples(
        case: BenchmarkCase,
        iterations: usize,
        mut samples: Vec<u128>,
        steps: Vec<&'static str>,
    ) -> Self {
        samples.sort_unstable();
        let total_ns = samples.iter().copied().sum();
        let min_ns = *samples.first().unwrap_or(&0);
        let max_ns = *samples.last().unwrap_or(&0);
        let median_ns = samples[samples.len() / 2];
        let p95_ns = samples[((samples.len().saturating_sub(1)) * 95) / 100];
        let avg_ns = total_ns / iterations as u128;
        Self {
            case: case.label().to_string(),
            iterations,
            total_ns,
            avg_ns,
            min_ns,
            median_ns,
            p95_ns,
            max_ns,
            steps: steps.into_iter().map(str::to_string).collect(),
        }
    }
}

fn measure(mut iterations: usize, mut f: impl FnMut()) -> Vec<u128> {
    let mut samples = Vec::with_capacity(iterations);
    while iterations > 0 {
        let start = Instant::now();
        f();
        samples.push(start.elapsed().as_nanos());
        iterations -= 1;
    }
    samples
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn benchmark_report_serializes() {
        let report = BenchmarkReport {
            cases: vec![BenchmarkResult {
                case: "entity".into(),
                iterations: 1,
                total_ns: 1,
                avg_ns: 1,
                min_ns: 1,
                median_ns: 1,
                p95_ns: 1,
                max_ns: 1,
                steps: vec!["compile".into(), "execute".into()],
            }],
        };
        let json = serde_json::to_string(&report).expect("serialize");
        assert!(json.contains("entity"));
    }
}
