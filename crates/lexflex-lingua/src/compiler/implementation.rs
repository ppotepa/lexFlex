use crate::compiler::{
    CompileError, CompiledConcept, CompiledConceptSemantics, CompiledFunction, CompiledProgram,
    ResolvedParameter, SymbolResolver,
};
use crate::syntax::{ConceptDeclaration, ConceptSemantics, LinguaDeclaration, LinguaProgram};
use crate::types::{FunctionType, TypeChecker, TypeEnvironment};
use crate::verifier::LinguaVerifier;
use lexflex_model::{ConceptCatalog, ConceptId};
use std::collections::BTreeMap;
use std::sync::Arc;

pub struct LinguaCompiler {
    catalog: Arc<ConceptCatalog>,
}

impl LinguaCompiler {
    pub fn new(catalog: Arc<ConceptCatalog>) -> Self {
        Self { catalog }
    }

    pub fn compile(&self, program: &LinguaProgram) -> Result<CompiledProgram, CompileError> {
        LinguaVerifier::default().verify_program(program)?;
        let concepts = self.compile_concepts(&program.declarations)?;
        let functions = self.compile_functions(&program.declarations)?;
        let environment = TypeEnvironment::new(
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
        );

        self.validate_compiled_units(&environment, &concepts, &functions)?;

        let mut resolver = SymbolResolver::new();
        resolver.push_scope();
        let entry = resolver.resolve_expression(&program.entry)?;
        let checker = TypeChecker::new(&environment);
        let _ = checker.infer(&entry)?;
        Ok(CompiledProgram {
            concepts,
            functions,
            entry,
        })
    }
}

impl LinguaCompiler {
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
