use crate::catalog::ModelPackageLoader;
use lexflex_lingua::runtime::RuntimeError;
use lexflex_lingua::{
    compiler::CompileError, CompileContext, CompiledModelContext, ExecutionPolicy, ExecutionResult,
    ExpansionMode, LinguaCompiler, LinguaGoal, LinguaInterpreter, LinguaProgram, LinguaSolver,
    TypedExecutionResult,
};
use lexflex_model::{ConceptCatalog, SemanticAssertion};
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;

pub struct LinguaRuntime {
    catalog: Arc<ConceptCatalog>,
    compiled_model: Arc<CompiledModelContext>,
    compiler: LinguaCompiler,
    interpreter: LinguaInterpreter,
    solver: LinguaSolver,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum EngineError {
    #[error("model load: {0}")]
    ModelLoad(#[from] crate::catalog::ModelLoadError),
    #[error("compile: {0}")]
    Compile(#[from] CompileError),
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
        Ok(Self::try_from_model(&model)?)
    }

    pub fn try_from_model(
        model: &crate::catalog::LoadedModelPackage,
    ) -> Result<Self, CompileError> {
        let catalog = model.catalog.clone();
        let compiler = LinguaCompiler::try_new(catalog.clone())?;

        Ok(Self {
            catalog,
            compiled_model: model.compiled_model.clone(),
            compiler,
            interpreter: LinguaInterpreter::default(),
            solver: LinguaSolver::default(),
        })
    }

    #[cfg(test)]
    pub fn from_catalog(catalog: Arc<ConceptCatalog>) -> Self {
        let compiler = LinguaCompiler::try_new(catalog.clone()).expect("test catalog hash");
        Self {
            catalog,
            compiled_model: Arc::new(
                compiler
                    .compile_model_declarations(&[])
                    .expect("empty model context"),
            ),
            compiler,
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
        self.interpreter
            .execute_program(&compiled, policy)
            .map_err(EngineError::Runtime)
    }

    pub fn evaluate_entry_with_context(
        &self,
        entry: &lexflex_lingua::LinguaExpression,
        context: &CompileContext,
        policy: ExecutionPolicy,
    ) -> Result<TypedExecutionResult, EngineError> {
        let compiled =
            self.compiler
                .compile_verified_entry(entry, self.compiled_model.clone(), context)?;
        self.interpreter
            .execute_entry(&compiled, policy)
            .map_err(EngineError::Runtime)
    }

    pub fn evaluate_with_context(
        &self,
        program: &LinguaProgram,
        context: &CompileContext,
        policy: ExecutionPolicy,
    ) -> Result<TypedExecutionResult, EngineError> {
        if !program.declarations.is_empty() {
            return Err(EngineError::Compile(
                CompileError::UnexpectedEntryDeclarations {
                    count: program.declarations.len(),
                },
            ));
        }
        self.evaluate_entry_with_context(&program.entry, context, policy)
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

    pub fn declaration_hash(&self) -> &lexflex_model::CanonicalDigest {
        self.compiled_model.declaration_hash()
    }
}
