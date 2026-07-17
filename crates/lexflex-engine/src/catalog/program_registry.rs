use crate::catalog::ProgramRegistryError;
use lexflex_lingua::{
    ConceptDeclaration, FunctionDeclaration, LinguaDeclaration, LinguaProgram, ProgramId,
};
use lexflex_model::{canonical_hash, CanonicalDigest, ConceptCatalog, ConceptId};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelProgramRegistry {
    programs: BTreeMap<ProgramId, LinguaProgram>,
    concepts: BTreeMap<ConceptId, ConceptDeclaration>,
    functions: BTreeMap<lexflex_lingua::FunctionId, FunctionDeclaration>,
    declarations: Vec<LinguaDeclaration>,
    declaration_hash: CanonicalDigest,
}

impl ModelProgramRegistry {
    pub fn build(
        programs: impl IntoIterator<Item = LinguaProgram>,
        catalog: &ConceptCatalog,
    ) -> Result<Self, ProgramRegistryError> {
        let mut by_program = BTreeMap::new();
        for program in programs {
            let program_id = program.id.clone();
            if by_program.insert(program_id.clone(), program).is_some() {
                return Err(ProgramRegistryError::DuplicateProgram(program_id));
            }
        }

        let mut declaration_ids = BTreeSet::new();
        let mut concepts = BTreeMap::new();
        let mut functions = BTreeMap::new();

        for program in by_program.values() {
            for declaration in &program.declarations {
                match declaration {
                    LinguaDeclaration::Concept(concept) => {
                        if !declaration_ids.insert(concept.declaration_id.clone()) {
                            return Err(ProgramRegistryError::DuplicateDeclaration(
                                concept.declaration_id.clone(),
                            ));
                        }
                        if !catalog.concepts.contains_key(&concept.concept_id) {
                            return Err(ProgramRegistryError::UnknownConcept(
                                concept.concept_id.clone(),
                            ));
                        }
                        if concepts
                            .insert(concept.concept_id.clone(), concept.clone())
                            .is_some()
                        {
                            return Err(ProgramRegistryError::DuplicateConcept(
                                concept.concept_id.clone(),
                            ));
                        }
                    }
                    LinguaDeclaration::Function(function) => {
                        if !declaration_ids.insert(function.declaration_id.clone()) {
                            return Err(ProgramRegistryError::DuplicateDeclaration(
                                function.declaration_id.clone(),
                            ));
                        }
                        if functions
                            .insert(function.function_id.clone(), function.clone())
                            .is_some()
                        {
                            return Err(ProgramRegistryError::DuplicateFunction(
                                function.function_id.clone(),
                            ));
                        }
                    }
                }
            }
        }

        let declarations = concepts
            .values()
            .cloned()
            .map(LinguaDeclaration::Concept)
            .chain(functions.values().cloned().map(LinguaDeclaration::Function))
            .collect::<Vec<_>>();
        let declaration_hash = canonical_hash(&declarations)?;

        Ok(Self {
            programs: by_program,
            concepts,
            functions,
            declarations,
            declaration_hash,
        })
    }

    pub fn declarations(&self) -> &[LinguaDeclaration] {
        &self.declarations
    }

    pub fn declaration_hash(&self) -> &CanonicalDigest {
        &self.declaration_hash
    }

    pub fn program_count(&self) -> usize {
        self.programs.len()
    }

    pub fn declaration_count(&self) -> usize {
        self.declarations.len()
    }

    pub fn concept_count(&self) -> usize {
        self.concepts.len()
    }

    pub fn function_count(&self) -> usize {
        self.functions.len()
    }
}
