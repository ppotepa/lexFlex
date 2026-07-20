use crate::diagnostic::ParseError;
use crate::meaning::MeaningInstance;
use lexflex_language::{
    CategoryType, CategoryTypeVariableId, CompiledLexicalSense, SyntacticCategory,
};
use lexflex_lingua::{LambdaParameter, LinguaExpression, SymbolName};
use lexflex_model::VariableId;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone)]
struct Freshener {
    prefix: String,
    next_category: u64,
    next_query: u64,
    next_symbol: u64,
    category_map: BTreeMap<CategoryTypeVariableId, CategoryTypeVariableId>,
    declared_query_map: BTreeMap<VariableId, VariableId>,
    used_declared_queries: BTreeSet<VariableId>,
    local_scopes: Vec<BTreeMap<SymbolName, SymbolName>>,
    bound_query_scopes: Vec<BTreeMap<VariableId, VariableId>>,
}

impl Freshener {
    fn new(seed: &str, declared_query_variables: impl IntoIterator<Item = VariableId>) -> Self {
        let prefix = seed.chars().take(16).collect::<String>();
        let mut value = Self {
            prefix,
            next_category: 0,
            next_query: 0,
            next_symbol: 0,
            category_map: BTreeMap::new(),
            declared_query_map: BTreeMap::new(),
            used_declared_queries: BTreeSet::new(),
            local_scopes: Vec::new(),
            bound_query_scopes: Vec::new(),
        };
        for variable in declared_query_variables {
            let fresh = value.allocate_query_variable();
            value.declared_query_map.insert(variable, fresh);
        }
        value
    }

    fn allocate_category_variable(&mut self) -> CategoryTypeVariableId {
        let value = CategoryTypeVariableId::new_unchecked(format!(
            "category:{}:{}",
            self.prefix, self.next_category
        ));
        self.next_category += 1;
        value
    }

    fn allocate_query_variable(&mut self) -> VariableId {
        let value = VariableId::new_unchecked(format!("query:{}:{}", self.prefix, self.next_query));
        self.next_query += 1;
        value
    }

    fn allocate_symbol_name(&mut self, original: &SymbolName) -> SymbolName {
        let value = SymbolName::new_unchecked(format!(
            "symbol:{}:{}:{}",
            self.prefix,
            self.next_symbol,
            original.as_str()
        ));
        self.next_symbol += 1;
        value
    }

    fn fresh_category_type(&mut self, category_type: &CategoryType) -> CategoryType {
        match category_type {
            CategoryType::Concrete(semantic_type) => CategoryType::Concrete(semantic_type.clone()),
            CategoryType::Variable(variable) => {
                let fresh = if let Some(existing) = self.category_map.get(variable) {
                    existing.clone()
                } else {
                    let fresh = self.allocate_category_variable();
                    self.category_map.insert(variable.clone(), fresh.clone());
                    fresh
                };
                CategoryType::Variable(fresh)
            }
        }
    }

    fn fresh_category(&mut self, category: &SyntacticCategory) -> SyntacticCategory {
        category.map_types(&mut |category_type| self.fresh_category_type(category_type))
    }

    fn fresh_expression(
        &mut self,
        expression: &LinguaExpression,
    ) -> Result<LinguaExpression, ParseError> {
        match expression {
            LinguaExpression::Concept(value) => Ok(LinguaExpression::Concept(value.clone())),
            LinguaExpression::Entity(value) => Ok(LinguaExpression::Entity(value.clone())),
            LinguaExpression::Value(value) => Ok(LinguaExpression::Value(value.clone())),
            LinguaExpression::Function(value) => Ok(LinguaExpression::Function(value.clone())),
            LinguaExpression::Variable(name) => {
                for scope in self.local_scopes.iter().rev() {
                    if let Some(mapped) = scope.get(name) {
                        return Ok(LinguaExpression::Variable(mapped.clone()));
                    }
                }
                Err(ParseError::MeaningFreshening(format!(
                    "free template symbol: {}",
                    name.as_str()
                )))
            }
            LinguaExpression::QueryVariable(variable) => {
                for scope in self.bound_query_scopes.iter().rev() {
                    if let Some(mapped) = scope.get(variable) {
                        return Ok(LinguaExpression::QueryVariable(mapped.clone()));
                    }
                }
                if let Some(mapped) = self.declared_query_map.get(variable) {
                    self.used_declared_queries.insert(variable.clone());
                    return Ok(LinguaExpression::QueryVariable(mapped.clone()));
                }
                Err(ParseError::MeaningFreshening(format!(
                    "undeclared query variable: {}",
                    variable
                )))
            }
            LinguaExpression::Lambda { parameters, body } => {
                self.local_scopes.push(BTreeMap::new());
                let mut fresh_parameters = Vec::with_capacity(parameters.len());
                for parameter in parameters {
                    let fresh_name = self.allocate_symbol_name(&parameter.name);
                    self.local_scopes
                        .last_mut()
                        .ok_or_else(|| {
                            ParseError::MeaningFreshening("missing lambda local scope".into())
                        })?
                        .insert(parameter.name.clone(), fresh_name.clone());
                    fresh_parameters.push(LambdaParameter {
                        name: fresh_name,
                        parameter_id: parameter.parameter_id.clone(),
                        value_type: parameter.value_type.clone(),
                    });
                }
                let body = self.fresh_expression(body)?;
                self.local_scopes.pop();
                Ok(LinguaExpression::Lambda {
                    parameters: fresh_parameters,
                    body: Box::new(body),
                })
            }
            LinguaExpression::Call { callee, arguments } => Ok(LinguaExpression::Call {
                callee: Box::new(self.fresh_expression(callee)?),
                arguments: arguments
                    .iter()
                    .map(|(parameter, value)| {
                        Ok((parameter.clone(), self.fresh_expression(value)?))
                    })
                    .collect::<Result<BTreeMap<_, _>, ParseError>>()?,
            }),
            LinguaExpression::ApplyConcept { concept, bindings } => {
                Ok(LinguaExpression::ApplyConcept {
                    concept: concept.clone(),
                    bindings: bindings
                        .iter()
                        .map(|(parameter, value)| {
                            Ok((parameter.clone(), self.fresh_expression(value)?))
                        })
                        .collect::<Result<BTreeMap<_, _>, ParseError>>()?,
                })
            }
            LinguaExpression::Satisfies { subject, concept } => Ok(LinguaExpression::Satisfies {
                subject: Box::new(self.fresh_expression(subject)?),
                concept: Box::new(self.fresh_expression(concept)?),
            }),
            LinguaExpression::Equals { left, right } => Ok(LinguaExpression::Equals {
                left: Box::new(self.fresh_expression(left)?),
                right: Box::new(self.fresh_expression(right)?),
            }),
            LinguaExpression::And(items) => Ok(LinguaExpression::And(
                items
                    .iter()
                    .map(|item| self.fresh_expression(item))
                    .collect::<Result<Vec<_>, ParseError>>()?,
            )),
            LinguaExpression::Or(items) => Ok(LinguaExpression::Or(
                items
                    .iter()
                    .map(|item| self.fresh_expression(item))
                    .collect::<Result<Vec<_>, ParseError>>()?,
            )),
            LinguaExpression::Not(inner) => Ok(LinguaExpression::Not(Box::new(
                self.fresh_expression(inner)?,
            ))),
            LinguaExpression::Exists {
                variable,
                value_type,
                body,
            } => {
                let fresh = self.allocate_query_variable();
                self.bound_query_scopes
                    .push(BTreeMap::from([(variable.clone(), fresh.clone())]));
                let body = self.fresh_expression(body)?;
                self.bound_query_scopes.pop();
                Ok(LinguaExpression::Exists {
                    variable: fresh,
                    value_type: value_type.clone(),
                    body: Box::new(body),
                })
            }
            LinguaExpression::ForAll {
                variable,
                value_type,
                body,
            } => {
                let fresh = self.allocate_query_variable();
                self.bound_query_scopes
                    .push(BTreeMap::from([(variable.clone(), fresh.clone())]));
                let body = self.fresh_expression(body)?;
                self.bound_query_scopes.pop();
                Ok(LinguaExpression::ForAll {
                    variable: fresh,
                    value_type: value_type.clone(),
                    body: Box::new(body),
                })
            }
            LinguaExpression::Let { name, value, body } => {
                let value = self.fresh_expression(value)?;
                let fresh_name = self.allocate_symbol_name(name);
                self.local_scopes
                    .push(BTreeMap::from([(name.clone(), fresh_name.clone())]));
                let body = self.fresh_expression(body)?;
                self.local_scopes.pop();
                Ok(LinguaExpression::Let {
                    name: fresh_name,
                    value: Box::new(value),
                    body: Box::new(body),
                })
            }
        }
    }
}

pub(crate) fn count_lingua_nodes(expression: &LinguaExpression) -> usize {
    match expression {
        LinguaExpression::Concept(_)
        | LinguaExpression::Entity(_)
        | LinguaExpression::Value(_)
        | LinguaExpression::Variable(_)
        | LinguaExpression::QueryVariable(_)
        | LinguaExpression::Function(_) => 1,
        LinguaExpression::Lambda { body, .. }
        | LinguaExpression::Not(body)
        | LinguaExpression::Exists { body, .. }
        | LinguaExpression::ForAll { body, .. } => 1 + count_lingua_nodes(body),
        LinguaExpression::Call { callee, arguments } => {
            1 + count_lingua_nodes(callee)
                + arguments.values().map(count_lingua_nodes).sum::<usize>()
        }
        LinguaExpression::ApplyConcept { bindings, .. } => {
            1 + bindings.values().map(count_lingua_nodes).sum::<usize>()
        }
        LinguaExpression::Satisfies { subject, concept }
        | LinguaExpression::Equals {
            left: subject,
            right: concept,
        } => 1 + count_lingua_nodes(subject) + count_lingua_nodes(concept),
        LinguaExpression::And(items) | LinguaExpression::Or(items) => {
            1 + items.iter().map(count_lingua_nodes).sum::<usize>()
        }
        LinguaExpression::Let { value, body, .. } => {
            1 + count_lingua_nodes(value) + count_lingua_nodes(body)
        }
    }
}

pub(crate) fn instantiate_meaning(
    seed: &str,
    category: &SyntacticCategory,
    sense: &CompiledLexicalSense,
) -> Result<(SyntacticCategory, MeaningInstance), ParseError> {
    let mut freshener = Freshener::new(seed, sense.meaning.query_variables.keys().cloned());
    let category = freshener.fresh_category(category);
    let expression = freshener.fresh_expression(&sense.meaning.expression)?;

    for declared in sense.meaning.query_variables.keys() {
        if !freshener.used_declared_queries.contains(declared) {
            return Err(ParseError::MeaningFreshening(format!(
                "declared query variable is unused: {}",
                declared
            )));
        }
    }

    let mut query_variables = BTreeMap::new();
    for (original, category_type) in &sense.meaning.query_variables {
        let fresh_variable = freshener
            .declared_query_map
            .get(original)
            .cloned()
            .ok_or_else(|| {
                ParseError::MeaningFreshening(format!(
                    "missing fresh query variable for {}",
                    original
                ))
            })?;
        let category_type = freshener.fresh_category_type(category_type);
        query_variables.insert(fresh_variable, category_type);
    }

    let semantic_nodes = count_lingua_nodes(&expression);
    Ok((
        category,
        MeaningInstance {
            expression,
            query_variables,
            semantic_nodes,
        },
    ))
}
