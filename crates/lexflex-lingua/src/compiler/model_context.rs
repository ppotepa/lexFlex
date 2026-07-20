use super::{CompiledConcept, CompiledFunction};
use crate::types::TypeEnvironment;
use crate::verifier::VerificationReport;
use lexflex_model::{CanonicalDigest, ConceptId};
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct VerifiedCompiledModel {
    pub(crate) identity: ModelContextIdentity,
    pub(crate) concepts: Arc<BTreeMap<ConceptId, CompiledConcept>>,
    pub(crate) functions: Arc<BTreeMap<crate::id::FunctionId, CompiledFunction>>,
    pub(crate) environment: Arc<TypeEnvironment>,
    pub(crate) verification: VerificationReport,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelContextIdentity {
    catalog_hash: CanonicalDigest,
    declaration_hash: CanonicalDigest,
    context_hash: CanonicalDigest,
}

impl ModelContextIdentity {
    pub(crate) fn new(
        catalog_hash: CanonicalDigest,
        declaration_hash: CanonicalDigest,
    ) -> Result<Self, lexflex_model::CanonicalHashError> {
        let context_hash = lexflex_model::canonical_hash(&(&catalog_hash, &declaration_hash))?;
        Ok(Self {
            catalog_hash,
            declaration_hash,
            context_hash,
        })
    }

    pub fn catalog_hash(&self) -> &CanonicalDigest {
        &self.catalog_hash
    }
    pub fn declaration_hash(&self) -> &CanonicalDigest {
        &self.declaration_hash
    }
    pub fn context_hash(&self) -> &CanonicalDigest {
        &self.context_hash
    }
}

pub type CompiledModelContext = VerifiedCompiledModel;

impl VerifiedCompiledModel {
    pub fn concepts(&self) -> &BTreeMap<ConceptId, CompiledConcept> {
        &self.concepts
    }
    pub fn functions(&self) -> &BTreeMap<crate::id::FunctionId, CompiledFunction> {
        &self.functions
    }
    pub fn identity(&self) -> &ModelContextIdentity {
        &self.identity
    }
    pub fn declaration_hash(&self) -> &CanonicalDigest {
        self.identity.declaration_hash()
    }
    pub fn catalog_hash(&self) -> &CanonicalDigest {
        self.identity.catalog_hash()
    }
    pub fn context_hash(&self) -> &CanonicalDigest {
        self.identity.context_hash()
    }

    pub fn verification(&self) -> &VerificationReport {
        &self.verification
    }
}
