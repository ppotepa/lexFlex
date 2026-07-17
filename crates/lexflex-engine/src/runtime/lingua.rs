use crate::catalog::ModelPackageLoader;
use lexflex_lingua::runtime::RuntimeError;
use lexflex_lingua::{
    compiler::CompileError, CompileContext, ExecutionPolicy, ExecutionResult, ExpansionMode,
    LinguaCompiler, LinguaDeclaration, LinguaGoal, LinguaInterpreter, LinguaProgram, LinguaSolver,
    LinguaVerifier, TypedExecutionResult,
};
use lexflex_model::{ConceptCatalog, SemanticAssertion};
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;

pub struct LinguaRuntime {
    catalog: Arc<ConceptCatalog>,
    base_declarations: Arc<Vec<LinguaDeclaration>>,
    compiler: LinguaCompiler,
    verifier: LinguaVerifier,
    interpreter: LinguaInterpreter,
    solver: LinguaSolver,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum EngineError {
    #[error("model load: {0}")]
    ModelLoad(#[from] crate::catalog::ModelLoadError),
    #[error("compile: {0}")]
    Compile(#[from] CompileError),
    #[error("verify: {0}")]
    Verify(CompileError),
    #[error("runtime: {0}")]
    Runtime(#[from] RuntimeError),
    #[error("normalize: {0:?}")]
    Normalize(lexflex_lingua::NormalizationReport),
    #[error("solve: {0}")]
    Solve(#[from] lexflex_lingua::SolveError),
}

impl LinguaRuntime {
    pub fn new(model_root: impl AsRef<Path>) -> Result<Self, EngineError> {
        let model = ModelPackageLoader
            .load(model_root.as_ref())
            .map_err(EngineError::ModelLoad)?;
        Ok(Self::from_model(&model))
    }

    pub fn from_model(model: &crate::catalog::LoadedModelPackage) -> Self {
        let catalog = Arc::new(model.catalog.clone());
        let compiler = LinguaCompiler::new(catalog.clone());
        let base_declarations = model.programs.declarations().to_vec();

        Self {
            catalog,
            base_declarations: Arc::new(base_declarations),
            compiler,
            verifier: LinguaVerifier::default(),
            interpreter: LinguaInterpreter::default(),
            solver: LinguaSolver::default(),
        }
    }

    #[cfg(test)]
    pub fn from_catalog(catalog: Arc<ConceptCatalog>) -> Self {
        let compiler = LinguaCompiler::new(catalog.clone());
        Self {
            catalog,
            base_declarations: Arc::new(Vec::new()),
            compiler,
            verifier: LinguaVerifier::default(),
            interpreter: LinguaInterpreter::default(),
            solver: LinguaSolver::default(),
        }
    }

    pub fn evaluate(&self, program: &LinguaProgram) -> Result<ExecutionResult, EngineError> {
        self.evaluate_with_policy(
            program,
            ExecutionPolicy {
                expansion: ExpansionMode::PreserveApplications,
            },
        )
        .map(|result| result.execution)
    }

    pub fn evaluate_with_policy(
        &self,
        program: &LinguaProgram,
        policy: ExecutionPolicy,
    ) -> Result<TypedExecutionResult, EngineError> {
        let compiled = self.compiler.compile(program)?;
        self.verifier
            .verify_compiled(&compiled)
            .map_err(EngineError::Verify)?;
        self.interpreter
            .execute_with_policy(&compiled, policy)
            .map_err(EngineError::Runtime)
    }

    pub fn evaluate_with_context(
        &self,
        program: &LinguaProgram,
        context: &CompileContext,
        policy: ExecutionPolicy,
    ) -> Result<TypedExecutionResult, EngineError> {
        let compiled = self.compiler.compile_with_context(program, context)?;
        self.verifier
            .verify_compiled(&compiled)
            .map_err(EngineError::Verify)?;
        self.interpreter
            .execute_with_policy(&compiled, policy)
            .map_err(EngineError::Runtime)
    }

    pub fn solve<'a>(
        &self,
        goal: &LinguaGoal,
        assertions: impl IntoIterator<Item = &'a SemanticAssertion>,
    ) -> Result<Vec<lexflex_lingua::QuerySolution>, EngineError> {
        self.solver
            .solve(goal, assertions, self.catalog.clone())
            .map_err(EngineError::Solve)
    }

    pub fn catalog(&self) -> &Arc<ConceptCatalog> {
        &self.catalog
    }

    pub fn base_declarations(&self) -> &[LinguaDeclaration] {
        self.base_declarations.as_slice()
    }
}
