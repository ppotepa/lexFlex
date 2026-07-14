mod execution;
mod rendering;

use serde::Serialize;

use crate::document::knowledge::DocumentKnowledgeExtraction;

use super::{
    AnswerId, AnswerKind, AnswerStatus, DocumentAnswer, QueryAggregation, QueryConstraint,
    QueryError, QueryExecutionPlan, QueryExecutionResult, QueryIntent, QueryInterlingua,
    QueryOperator, QueryPlanId, QueryStep,
};

pub struct QueryService;

impl QueryService {
    pub fn compile_query(query: QueryInterlingua) -> Result<QueryInterlingua, QueryError> {
        let expected = hash_without_field(&query, "query_sha256")?;
        if expected != query.query_sha256 {
            return Err(QueryError("query hash mismatch".into()));
        }
        validate_constraints(&query.constraints)?;
        Ok(query)
    }

    pub fn plan_query(query: &QueryInterlingua) -> Result<QueryExecutionPlan, QueryError> {
        let mut steps = vec![QueryStep::Scan, QueryStep::Filter, QueryStep::Project];
        let mut operators = Vec::new();
        collect_operators(&query.constraints, &mut operators);
        if operators.is_empty() {
            operators.push(QueryOperator::UnknownGuard);
        }
        operators.push(QueryOperator::EvidenceAttach);
        if matches!(
            query.aggregation,
            Some(QueryAggregation::Count | QueryAggregation::Distinct)
        ) {
            steps.push(QueryStep::Aggregate);
            operators.push(if matches!(query.aggregation, Some(QueryAggregation::Distinct)) {
                QueryOperator::Distinct
            } else {
                QueryOperator::Aggregate
            });
        }
        if !query.ordering.is_empty() {
            operators.push(QueryOperator::Sort);
        }
        if query.limit.is_some() {
            operators.push(QueryOperator::Limit);
        }
        if matches!(query.intent, QueryIntent::Explain) {
            steps.push(QueryStep::Explain);
        }
        let plan = QueryExecutionPlan {
            id: QueryPlanId(format!("{}:plan", query.id.0)),
            query_id: query.id.clone(),
            steps,
            operators,
            plan_sha256: String::new(),
        };
        Ok(QueryExecutionPlan {
            plan_sha256: hash_without_field(&plan, "plan_sha256")?,
            ..plan
        })
    }

    pub fn execute_query(
        plan: &QueryExecutionPlan,
        query: &QueryInterlingua,
        knowledge: &DocumentKnowledgeExtraction,
    ) -> Result<QueryExecutionResult, QueryError> {
        execution::execute_query(plan, query, knowledge)
    }

    pub fn answer_document_query(
        query: QueryInterlingua,
        knowledge: &DocumentKnowledgeExtraction,
    ) -> Result<DocumentAnswer, QueryError> {
        let query = Self::compile_query(query)?;
        let plan = Self::plan_query(&query)?;
        let execution = Self::execute_query(&plan, &query, knowledge)?;
        let status = rendering::answer_status(&query, &execution);
        let kind = match query.intent {
            QueryIntent::Count => AnswerKind::Count,
            QueryIntent::Boolean => AnswerKind::Boolean,
            QueryIntent::LookupEntity | QueryIntent::LookupRelation => AnswerKind::EntityList,
            QueryIntent::LookupProperty => AnswerKind::ValueList,
            QueryIntent::Explain => AnswerKind::Text,
            QueryIntent::Contradiction => AnswerKind::ConflictReport,
            QueryIntent::Provenance => AnswerKind::EvidenceReport,
            _ => AnswerKind::NoAnswer,
        };
        let evidence = execution
            .rows
            .iter()
            .flat_map(|row| row.evidence.clone())
            .collect::<Vec<_>>();
        let text = rendering::answer_text(&query, &execution, status, knowledge);
        let conflicts = if matches!(status, AnswerStatus::Conflicting) {
            execution
                .rows
                .iter()
                .filter_map(|row| row.columns.get("claim").cloned())
                .collect()
        } else {
            Vec::new()
        };
        let answer = DocumentAnswer {
            id: AnswerId(format!("{}:answer", query.id.0)),
            query_id: query.id.clone(),
            plan_id: plan.id.clone(),
            status,
            kind,
            text,
            rows: execution.rows,
            evidence,
            conflicts,
            answer_sha256: String::new(),
        };
        Ok(DocumentAnswer {
            answer_sha256: hash_without_field(&answer, "answer_sha256")?,
            ..answer
        })
    }
}

fn validate_constraints(constraint: &QueryConstraint) -> Result<(), QueryError> {
    match constraint {
        QueryConstraint::And(items) | QueryConstraint::Or(items) => {
            for item in items {
                validate_constraints(item)?;
            }
            Ok(())
        }
        QueryConstraint::Not(item) => validate_constraints(item),
        QueryConstraint::TextContains(_) => Err(QueryError(
            "TextContains is unsupported in the authoritative query pipeline".into(),
        )),
        _ => Ok(()),
    }
}

fn collect_operators(constraint: &QueryConstraint, operators: &mut Vec<QueryOperator>) {
    match constraint {
        QueryConstraint::And(items) | QueryConstraint::Or(items) => {
            for item in items {
                collect_operators(item, operators);
            }
        }
        QueryConstraint::Not(item) => collect_operators(item, operators),
        QueryConstraint::Subject(_) => operators.push(QueryOperator::ScanBySubject),
        QueryConstraint::Predicate(_) => operators.push(QueryOperator::ScanByPredicate),
        QueryConstraint::Object(_) => operators.push(QueryOperator::ScanByObject),
        QueryConstraint::TextContains(_)
        | QueryConstraint::ClaimStatus(_)
        | QueryConstraint::Claim(_)
        | QueryConstraint::EvidenceRequired(_) => operators.push(QueryOperator::Filter),
    }
}

fn hash_without_field<T: Serialize>(value: &T, field: &str) -> Result<String, QueryError> {
    let mut json = serde_json::to_value(value).map_err(|err| QueryError(err.to_string()))?;
    if let serde_json::Value::Object(map) = &mut json {
        map.remove(field);
    }
    let bytes = serde_json::to_vec(&json).map_err(|err| QueryError(err.to_string()))?;
    let mut hasher = sha2::Sha256::new();
    use sha2::Digest;
    hasher.update(bytes);
    Ok(format!("{:x}", hasher.finalize()))
}
