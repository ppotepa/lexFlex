use crate::id::{FunctionId, SymbolId};
use crate::syntax::{ConceptDeclaration, ExpansionPolicy, FunctionDeclaration};
use crate::types::SemanticType;
use crate::verifier::VerificationReport;
use lexflex_model::{ConceptId, EntityId, ParameterId, SemanticValue, VariableId};
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedExpression {
    Concept(ConceptId),
    Entity(EntityId),
    Value(SemanticValue),
    Local(SymbolId),
    QueryVariable(VariableId),
    Function(FunctionId),
    Lambda {
        parameters: Vec<ResolvedParameter>,
        body: Box<ResolvedExpression>,
    },
    Call {
        callee: Box<ResolvedExpression>,
        arguments: BTreeMap<ParameterId, ResolvedExpression>,
    },
    ApplyConcept {
        concept: ConceptId,
        bindings: BTreeMap<ParameterId, ResolvedExpression>,
    },
    Satisfies {
        subject: Box<ResolvedExpression>,
        concept: Box<ResolvedExpression>,
    },
    Equals {
        left: Box<ResolvedExpression>,
        right: Box<ResolvedExpression>,
    },
    And(Vec<ResolvedExpression>),
    Or(Vec<ResolvedExpression>),
    Not(Box<ResolvedExpression>),
    Exists {
        variable: VariableId,
        value_type: SemanticType,
        body: Box<ResolvedExpression>,
    },
    ForAll {
        variable: VariableId,
        value_type: SemanticType,
        body: Box<ResolvedExpression>,
    },
    Let {
        symbol: SymbolId,
        value: Box<ResolvedExpression>,
        body: Box<ResolvedExpression>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedParameter {
    pub parameter_id: ParameterId,
    pub symbol: SymbolId,
    pub value_type: SemanticType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompiledConceptSemantics {
    Primitive,
    Defined { body: ResolvedExpression },
}

#[derive(Debug, Clone)]
pub struct CompiledConcept {
    pub(crate) declaration: ConceptDeclaration,
    pub(crate) self_parameter: Option<ResolvedParameter>,
    pub(crate) parameters: Vec<ResolvedParameter>,
    pub(crate) semantics: CompiledConceptSemantics,
    pub(crate) expansion: ExpansionPolicy,
}

#[derive(Debug, Clone)]
pub struct CompiledFunction {
    pub(crate) declaration: FunctionDeclaration,
    pub(crate) parameters: Vec<ResolvedParameter>,
    pub(crate) body: ResolvedExpression,
}

#[derive(Debug, Clone)]
pub(crate) struct CompiledProgram {
    pub(crate) concepts: Arc<BTreeMap<ConceptId, CompiledConcept>>,
    pub(crate) functions: Arc<BTreeMap<FunctionId, CompiledFunction>>,
    pub(crate) entry: ResolvedExpression,
    pub(crate) entry_type: SemanticType,
}

#[derive(Debug, Clone)]
pub struct VerifiedCompiledEntry {
    model: Arc<super::VerifiedCompiledModel>,
    entry: ResolvedExpression,
    entry_type: SemanticType,
    verification: VerificationReport,
}

#[derive(Debug, Clone)]
pub struct VerifiedStandaloneProgram {
    concepts: Arc<BTreeMap<ConceptId, CompiledConcept>>,
    functions: Arc<BTreeMap<FunctionId, CompiledFunction>>,
    entry: ResolvedExpression,
    entry_type: SemanticType,
    verification: VerificationReport,
}

impl VerifiedStandaloneProgram {
    pub(crate) fn new(program: CompiledProgram, verification: VerificationReport) -> Self {
        Self {
            concepts: program.concepts,
            functions: program.functions,
            entry: program.entry,
            entry_type: program.entry_type,
            verification,
        }
    }

    pub fn entry(&self) -> &ResolvedExpression {
        &self.entry
    }
    pub fn entry_type(&self) -> &SemanticType {
        &self.entry_type
    }
    pub fn verification(&self) -> &VerificationReport {
        &self.verification
    }
    pub(crate) fn concepts(&self) -> &BTreeMap<ConceptId, CompiledConcept> {
        &self.concepts
    }
    pub(crate) fn functions(&self) -> &BTreeMap<FunctionId, CompiledFunction> {
        &self.functions
    }
}

impl VerifiedCompiledEntry {
    pub(crate) fn new(
        model: Arc<super::VerifiedCompiledModel>,
        program: CompiledProgram,
        verification: VerificationReport,
    ) -> Self {
        Self {
            model,
            entry: program.entry,
            entry_type: program.entry_type,
            verification,
        }
    }

    pub fn model(&self) -> &Arc<super::VerifiedCompiledModel> {
        &self.model
    }

    pub fn entry(&self) -> &ResolvedExpression {
        &self.entry
    }

    pub fn entry_type(&self) -> &SemanticType {
        &self.entry_type
    }

    pub fn verification(&self) -> &VerificationReport {
        &self.verification
    }
}
