use crate::compiler::{CompileError, ResolvedExpression, ResolvedParameter};
use crate::id::{SymbolId, SymbolName};
use crate::syntax::LinguaExpression;
use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct SymbolResolver {
    scopes: Vec<BTreeMap<SymbolName, SymbolId>>,
    next_symbol: u64,
}

impl SymbolResolver {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(BTreeMap::new());
    }

    pub fn pop_scope(&mut self) -> Result<(), CompileError> {
        self.scopes
            .pop()
            .ok_or_else(|| CompileError::Diagnostic("pop_scope on empty stack".into()))?;
        Ok(())
    }

    pub fn declare(&mut self, name: &SymbolName) -> Result<SymbolId, CompileError> {
        let scope = self
            .scopes
            .last_mut()
            .ok_or_else(|| CompileError::Diagnostic("no active scope".into()))?;
        if scope.contains_key(name) {
            return Err(CompileError::Diagnostic(
                format!("duplicate symbol '{}'", name.as_str()).into(),
            ));
        }
        let symbol = SymbolId::new_unchecked(format!("sym:{}", self.next_symbol));
        self.next_symbol += 1;
        scope.insert(name.clone(), symbol.clone());
        Ok(symbol)
    }

    pub fn resolve(&self, name: &SymbolName) -> Result<SymbolId, CompileError> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Ok(symbol.clone());
            }
        }
        Err(CompileError::Diagnostic(
            format!("unknown symbol '{}'", name.as_str()).into(),
        ))
    }

    pub fn resolve_expression(
        &mut self,
        expression: &LinguaExpression,
    ) -> Result<ResolvedExpression, CompileError> {
        match expression {
            LinguaExpression::Concept(id) => Ok(ResolvedExpression::Concept(id.clone())),
            LinguaExpression::Entity(id) => Ok(ResolvedExpression::Entity(id.clone())),
            LinguaExpression::Value(value) => Ok(ResolvedExpression::Value(value.clone())),
            LinguaExpression::Variable(name) => Ok(ResolvedExpression::Local(self.resolve(name)?)),
            LinguaExpression::QueryVariable(variable) => {
                Ok(ResolvedExpression::QueryVariable(variable.clone()))
            }
            LinguaExpression::Function(function_id) => {
                Ok(ResolvedExpression::Function(function_id.clone()))
            }
            LinguaExpression::Lambda { parameters, body } => {
                self.push_scope();
                let mut resolved = Vec::with_capacity(parameters.len());
                for parameter in parameters {
                    let symbol = self.declare(&parameter.name)?;
                    resolved.push(ResolvedParameter {
                        parameter_id: parameter.parameter_id.clone(),
                        symbol,
                        value_type: parameter.value_type.clone(),
                    });
                }
                let body = self.resolve_expression(body)?;
                self.pop_scope()?;
                Ok(ResolvedExpression::Lambda {
                    parameters: resolved,
                    body: Box::new(body),
                })
            }
            LinguaExpression::Call { callee, arguments } => Ok(ResolvedExpression::Call {
                callee: Box::new(self.resolve_expression(callee)?),
                arguments: arguments
                    .iter()
                    .map(|(parameter, value)| {
                        Ok((parameter.clone(), self.resolve_expression(value)?))
                    })
                    .collect::<Result<BTreeMap<_, _>, CompileError>>()?,
            }),
            LinguaExpression::ApplyConcept { concept, bindings } => {
                Ok(ResolvedExpression::ApplyConcept {
                    concept: concept.clone(),
                    bindings: bindings
                        .iter()
                        .map(|(parameter, value)| {
                            Ok((parameter.clone(), self.resolve_expression(value)?))
                        })
                        .collect::<Result<BTreeMap<_, _>, CompileError>>()?,
                })
            }
            LinguaExpression::Satisfies { subject, concept } => Ok(ResolvedExpression::Satisfies {
                subject: Box::new(self.resolve_expression(subject)?),
                concept: Box::new(self.resolve_expression(concept)?),
            }),
            LinguaExpression::Equals { left, right } => Ok(ResolvedExpression::Equals {
                left: Box::new(self.resolve_expression(left)?),
                right: Box::new(self.resolve_expression(right)?),
            }),
            LinguaExpression::And(items) => Ok(ResolvedExpression::And(
                items
                    .iter()
                    .map(|item| self.resolve_expression(item))
                    .collect::<Result<_, _>>()?,
            )),
            LinguaExpression::Or(items) => Ok(ResolvedExpression::Or(
                items
                    .iter()
                    .map(|item| self.resolve_expression(item))
                    .collect::<Result<_, _>>()?,
            )),
            LinguaExpression::Not(item) => Ok(ResolvedExpression::Not(Box::new(
                self.resolve_expression(item)?,
            ))),
            LinguaExpression::Exists {
                variable,
                value_type,
                body,
            } => Ok(ResolvedExpression::Exists {
                variable: variable.clone(),
                value_type: value_type.clone(),
                body: Box::new(self.resolve_expression(body)?),
            }),
            LinguaExpression::ForAll {
                variable,
                value_type,
                body,
            } => Ok(ResolvedExpression::ForAll {
                variable: variable.clone(),
                value_type: value_type.clone(),
                body: Box::new(self.resolve_expression(body)?),
            }),
            LinguaExpression::Let { name, value, body } => {
                self.push_scope();
                let value = self.resolve_expression(value)?;
                let symbol = self.declare(name)?;
                let body = self.resolve_expression(body)?;
                self.pop_scope()?;
                Ok(ResolvedExpression::Let {
                    symbol,
                    value: Box::new(value),
                    body: Box::new(body),
                })
            }
        }
    }
}
