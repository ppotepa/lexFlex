use super::resolved::CompiledProgram;
use crate::compiler::{
    type_reference_validation::validate_expression_type_references, CompileContext, CompileError,
    CompileTypeReferenceError, CompileTypeReferenceLocation, CompiledConcept,
    CompiledConceptSemantics, CompiledFunction, CompiledModelContext, ModelContextIdentity,
    ResolvedParameter, SymbolResolver, VerifiedCompiledEntry, VerifiedStandaloneProgram,
};
use crate::syntax::{ConceptDeclaration, ConceptSemantics, LinguaDeclaration, LinguaProgram};
use crate::types::{FunctionType, TypeChecker, TypeEnvironment};
use crate::verifier::LinguaVerifier;
use lexflex_model::{canonical_hash, validate_semantic_type_references, ConceptCatalog, ConceptId};
use std::collections::BTreeMap;
use std::sync::Arc;

pub struct LinguaCompiler {
    catalog: Arc<ConceptCatalog>,
    catalog_hash: lexflex_model::CanonicalDigest,
}

impl LinguaCompiler {
    pub fn try_new(catalog: Arc<ConceptCatalog>) -> Result<Self, CompileError> {
        let catalog_hash = canonical_hash(catalog.as_ref())?;
        Ok(Self {
            catalog,
            catalog_hash,
        })
    }

    pub fn compile(
        &self,
        program: &LinguaProgram,
    ) -> Result<VerifiedStandaloneProgram, CompileError> {
        self.compile_with_context(program, &CompileContext::default())
    }

    pub fn compile_model_declarations(
        &self,
        declarations: &[LinguaDeclaration],
    ) -> Result<CompiledModelContext, CompileError> {
        let entry = crate::syntax::LinguaExpression::Value(true.into());
        let program = LinguaProgram {
            id: crate::id::ProgramId::new_unchecked("model-context"),
            declarations: declarations.to_vec(),
            entry,
        };
        let verification = LinguaVerifier::default().verify_program(&program)?;
        self.validate_declaration_type_references(declarations)?;
        let concepts = self.compile_concepts(declarations)?;
        let functions = self.compile_functions(declarations)?;
        let environment = self.environment_for_functions(&functions);
        self.validate_compiled_units(&environment, &concepts, &functions)?;
        Ok(CompiledModelContext {
            concepts: Arc::new(concepts),
            functions: Arc::new(functions),
            environment: Arc::new(environment),
            identity: ModelContextIdentity::new(
                self.catalog_hash.clone(),
                canonical_hash(declarations)?,
            )?,
            verification,
        })
    }

    fn compile_entry_internal(
        &self,
        entry: &crate::syntax::LinguaExpression,
        model: &CompiledModelContext,
        context: &CompileContext,
    ) -> Result<CompiledProgram, CompileError> {
        if &self.catalog_hash != model.catalog_hash() {
            return Err(CompileError::ModelContextMismatch {
                compiler_catalog: self.catalog_hash.clone(),
                model_catalog: model.catalog_hash().clone(),
            });
        }
        context.validate(&self.catalog)?;
        validate_expression_type_references(entry, &self.catalog)?;
        let mut resolver = SymbolResolver::new();
        resolver.push_scope();
        let resolved = resolver.resolve_expression(entry)?;
        let mut environment = (*model.environment).clone();
        environment.variables = context.query_variables.clone();
        let entry_type = TypeChecker::new(&environment).infer(&resolved)?;
        Ok(CompiledProgram {
            concepts: model.concepts.clone(),
            functions: model.functions.clone(),
            entry: resolved,
            entry_type,
        })
    }

    pub fn compile_entry_with_context(
        &self,
        entry: &crate::syntax::LinguaExpression,
        model: &CompiledModelContext,
        context: &CompileContext,
    ) -> Result<VerifiedStandaloneProgram, CompileError> {
        let program = self.compile_entry_internal(entry, model, context)?;
        let verification = LinguaVerifier::default().verify_resolved_entry(&program.entry)?;
        Ok(VerifiedStandaloneProgram::new(program, verification))
    }

    pub fn compile_verified_entry(
        &self,
        entry: &crate::syntax::LinguaExpression,
        model: Arc<CompiledModelContext>,
        context: &CompileContext,
    ) -> Result<VerifiedCompiledEntry, CompileError> {
        let program = self.compile_entry_internal(entry, &model, context)?;
        let verification = LinguaVerifier::default().verify_resolved_entry(&program.entry)?;
        Ok(VerifiedCompiledEntry::new(model, program, verification))
    }

    pub fn compile_with_context(
        &self,
        program: &LinguaProgram,
        context: &CompileContext,
    ) -> Result<VerifiedStandaloneProgram, CompileError> {
        let verification = LinguaVerifier::default().verify_program(program)?;
        context.validate(&self.catalog)?;
        self.validate_declaration_type_references(&program.declarations)?;
        validate_expression_type_references(&program.entry, &self.catalog)?;
        let concepts = self.compile_concepts(&program.declarations)?;
        let functions = self.compile_functions(&program.declarations)?;
        let mut environment = self.environment_for_functions(&functions);
        environment.variables = context.query_variables.clone();

        self.validate_compiled_units(&environment, &concepts, &functions)?;

        let mut resolver = SymbolResolver::new();
        resolver.push_scope();
        let entry = resolver.resolve_expression(&program.entry)?;
        let checker = TypeChecker::new(&environment);
        let entry_type = checker.infer(&entry)?;
        Ok(VerifiedStandaloneProgram::new(
            CompiledProgram {
                concepts: Arc::new(concepts),
                functions: Arc::new(functions),
                entry,
                entry_type,
            },
            verification,
        ))
    }

    fn environment_for_functions(
        &self,
        functions: &BTreeMap<crate::id::FunctionId, CompiledFunction>,
    ) -> TypeEnvironment {
        TypeEnvironment::new(
            self.catalog.clone(),
            functions
                .iter()
                .map(|(id, function)| {
                    (
                        id.clone(),
                        FunctionType {
                            parameters: function
                                .parameters
                                .iter()
                                .map(|parameter| {
                                    (parameter.parameter_id.clone(), parameter.value_type.clone())
                                })
                                .collect(),
                            result: Box::new(function.declaration.result_type.clone()),
                        },
                    )
                })
                .collect(),
        )
    }
}

impl LinguaCompiler {
    fn validate_declaration_type_references(
        &self,
        declarations: &[LinguaDeclaration],
    ) -> Result<(), CompileError> {
        for declaration in declarations {
            match declaration {
                LinguaDeclaration::Concept(concept) => {
                    if let Some(parameter) = &concept.self_parameter {
                        validate_semantic_type_references(&parameter.value_type, &self.catalog)
                            .map_err(|source| {
                                CompileError::TypeReference(CompileTypeReferenceError {
                                    location: CompileTypeReferenceLocation::ConceptSelfParameter {
                                        concept: concept.concept_id.clone(),
                                        parameter: parameter.parameter_id.clone(),
                                    },
                                    source,
                                })
                            })?;
                    }
                    for parameter in &concept.parameters {
                        validate_semantic_type_references(&parameter.value_type, &self.catalog)
                            .map_err(|source| {
                                CompileError::TypeReference(CompileTypeReferenceError {
                                    location: CompileTypeReferenceLocation::ConceptParameter {
                                        concept: concept.concept_id.clone(),
                                        parameter: parameter.parameter_id.clone(),
                                    },
                                    source,
                                })
                            })?;
                    }
                    if let ConceptSemantics::Defined { body } = &concept.semantics {
                        validate_expression_type_references(body, &self.catalog)?;
                    }
                }
                LinguaDeclaration::Function(function) => {
                    for parameter in &function.parameters {
                        validate_semantic_type_references(&parameter.value_type, &self.catalog)
                            .map_err(|source| {
                                CompileError::TypeReference(CompileTypeReferenceError {
                                    location: CompileTypeReferenceLocation::FunctionParameter {
                                        function: function.function_id.clone(),
                                        parameter: parameter.parameter_id.clone(),
                                    },
                                    source,
                                })
                            })?;
                    }
                    validate_semantic_type_references(&function.result_type, &self.catalog)
                        .map_err(|source| {
                            CompileError::TypeReference(CompileTypeReferenceError {
                                location: CompileTypeReferenceLocation::FunctionResult {
                                    function: function.function_id.clone(),
                                },
                                source,
                            })
                        })?;
                    validate_expression_type_references(&function.body, &self.catalog)?;
                }
            }
        }
        Ok(())
    }

    fn compile_concepts(
        &self,
        declarations: &[LinguaDeclaration],
    ) -> Result<BTreeMap<ConceptId, CompiledConcept>, CompileError> {
        let mut concepts = BTreeMap::new();
        for declaration in declarations {
            let LinguaDeclaration::Concept(concept) = declaration else {
                continue;
            };
            if concepts.contains_key(&concept.concept_id) {
                return Err(CompileError::Diagnostic(
                    format!("duplicate concept id: {}", concept.concept_id).into(),
                ));
            }
            concepts.insert(concept.concept_id.clone(), self.compile_concept(concept)?);
        }
        Ok(concepts)
    }

    fn compile_concept(
        &self,
        declaration: &ConceptDeclaration,
    ) -> Result<CompiledConcept, CompileError> {
        let mut resolver = SymbolResolver::new();
        resolver.push_scope();

        let self_parameter = if let Some(parameter) = declaration.self_parameter.as_ref() {
            let symbol = resolver.declare(&parameter.name)?;
            Some(ResolvedParameter {
                parameter_id: parameter.parameter_id.clone(),
                symbol,
                value_type: parameter.value_type.clone(),
            })
        } else {
            None
        };

        let mut parameters = Vec::with_capacity(declaration.parameters.len());
        for parameter in &declaration.parameters {
            let symbol = resolver.declare(&parameter.name)?;
            parameters.push(ResolvedParameter {
                parameter_id: parameter.parameter_id.clone(),
                symbol,
                value_type: parameter.value_type.clone(),
            });
        }

        let semantics = match &declaration.semantics {
            ConceptSemantics::Primitive => CompiledConceptSemantics::Primitive,
            ConceptSemantics::Defined { body } => CompiledConceptSemantics::Defined {
                body: resolver.resolve_expression(body)?,
            },
        };

        resolver.pop_scope()?;

        Ok(CompiledConcept {
            declaration: declaration.clone(),
            self_parameter,
            parameters,
            semantics,
            expansion: declaration.expansion,
        })
    }

    fn compile_functions(
        &self,
        declarations: &[LinguaDeclaration],
    ) -> Result<BTreeMap<crate::id::FunctionId, CompiledFunction>, CompileError> {
        let mut functions = BTreeMap::new();
        for declaration in declarations {
            let LinguaDeclaration::Function(function) = declaration else {
                continue;
            };
            if functions.contains_key(&function.function_id) {
                return Err(CompileError::Diagnostic(
                    format!("duplicate function id: {}", function.function_id).into(),
                ));
            }

            let mut resolver = SymbolResolver::new();
            resolver.push_scope();

            let mut parameters = Vec::with_capacity(function.parameters.len());
            for parameter in &function.parameters {
                let symbol = resolver.declare(&parameter.name)?;
                parameters.push(ResolvedParameter {
                    parameter_id: parameter.parameter_id.clone(),
                    symbol,
                    value_type: parameter.value_type.clone(),
                });
            }

            let body = resolver.resolve_expression(&function.body)?;
            resolver.pop_scope()?;

            functions.insert(
                function.function_id.clone(),
                CompiledFunction {
                    declaration: function.clone(),
                    parameters,
                    body,
                },
            );
        }
        Ok(functions)
    }

    fn validate_compiled_units(
        &self,
        environment: &TypeEnvironment,
        concepts: &BTreeMap<ConceptId, CompiledConcept>,
        functions: &BTreeMap<crate::id::FunctionId, CompiledFunction>,
    ) -> Result<(), CompileError> {
        for concept in concepts.values() {
            let Some(schema) = self.catalog.concepts.get(&concept.declaration.concept_id) else {
                return Err(CompileError::Diagnostic(
                    format!(
                        "unknown concept in declaration: {}",
                        concept.declaration.concept_id
                    )
                    .into(),
                ));
            };

            let expected = schema.result_type.clone();

            if let CompiledConceptSemantics::Defined { body } = &concept.semantics {
                let mut child = environment.clone();
                if let Some(self_parameter) = &concept.self_parameter {
                    child.locals.insert(
                        self_parameter.symbol.clone(),
                        self_parameter.value_type.clone(),
                    );
                }
                for parameter in &concept.parameters {
                    child
                        .locals
                        .insert(parameter.symbol.clone(), parameter.value_type.clone());
                }

                let checker = TypeChecker::new(&child);
                let actual = checker.infer(body)?;
                if !checker.compatible(&actual, &expected) {
                    return Err(CompileError::Diagnostic(
                        format!(
                            "concept definition result mismatch for {}: expected {:?}, actual {:?}",
                            concept.declaration.concept_id, expected, actual
                        )
                        .into(),
                    ));
                }
            }
        }

        for function in functions.values() {
            let mut child = environment.clone();
            for parameter in &function.parameters {
                child
                    .locals
                    .insert(parameter.symbol.clone(), parameter.value_type.clone());
            }

            let checker = TypeChecker::new(&child);
            let actual = checker.infer(&function.body)?;
            let expected = function.declaration.result_type.clone();
            if !checker.compatible(&actual, &expected) {
                return Err(CompileError::Diagnostic(
                    format!(
                        "function result mismatch for {}: expected {:?}, actual {:?}",
                        function.declaration.function_id, expected, actual
                    )
                    .into(),
                ));
            }
        }

        Ok(())
    }
}
