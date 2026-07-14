use std::collections::BTreeMap;

use serde::Serialize;

use crate::document::knowledge::{
    ClaimStatus, DocumentKnowledgeExtraction, KnowledgeObjectRef, KnowledgePredicateRef,
    KnowledgeSubjectRef, PropositionOccurrence,
};

use super::{
    AnswerId, AnswerKind, AnswerStatus, DocumentAnswer, QueryAggregation, QueryConstraint,
    QueryError, QueryExecutionPlan, QueryExecutionResult, QueryExecutionResultRow,
    QueryIntent, QueryInterlingua, QueryOrdering, QueryPlanId, QueryProjection, QueryResultId, QueryStep,
};

pub struct QueryService;

impl QueryService {
    pub fn compile_query(query: QueryInterlingua) -> Result<QueryInterlingua, QueryError> {
        let expected = hash_without_field(&query, "query_sha256")?;
        if expected != query.query_sha256 {
            return Err(QueryError("query hash mismatch".into()));
        }
        Ok(query)
    }

    pub fn plan_query(query: &QueryInterlingua) -> Result<QueryExecutionPlan, QueryError> {
        let mut steps = vec![QueryStep::Scan, QueryStep::Filter, QueryStep::Project];
        if matches!(query.aggregation, Some(QueryAggregation::Count | QueryAggregation::Distinct)) {
            steps.push(QueryStep::Aggregate);
        }
        if matches!(query.intent, QueryIntent::Explain) {
            steps.push(QueryStep::Explain);
        }
        let plan = QueryExecutionPlan {
            id: QueryPlanId(format!("{}:plan", query.id.0)),
            query_id: query.id.clone(),
            steps,
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
        let mut rows = Vec::new();
        for claim_id in &knowledge.claim_order {
            let Some(claim) = knowledge.claims.get(claim_id) else {
                continue;
            };
            let occurrences = claim_occurrences(knowledge, claim);
            if !query_matches(&query.constraints, claim, &occurrences, knowledge) {
                continue;
            }
            let mut columns = BTreeMap::new();
            columns.insert("claim".into(), claim.id.to_string());
            columns.insert("status".into(), render_claim_status(&claim.status));
            columns.insert("subject".into(), render_subject(&claim.canonical_subject));
            columns.insert("predicate".into(), render_predicate(&claim.canonical_predicate));
            columns.insert("object".into(), render_object(&claim.canonical_object));
            columns.insert("factuality".into(), render_factuality(&claim.factuality));
            columns.insert("world".into(), render_world(&claim.world));
            columns.insert("confidence_milli".into(), claim.confidence_milli.to_string());
            columns.insert(
                "occurrences".into(),
                occurrences
                    .iter()
                    .map(|occurrence| occurrence.id.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
            );
            columns.insert(
                "source_sentences".into(),
                occurrences
                    .iter()
                    .map(|occurrence| occurrence.source_sentence_id.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
            );
            columns.insert("summary".into(), claim_render_text(claim, &occurrences));
            columns.insert("evidence_count".into(), claim_evidence(claim, &occurrences).len().to_string());
            project_columns(&mut columns, &query.projection);
            rows.push(QueryExecutionResultRow {
                columns,
                evidence: claim_evidence(claim, &occurrences),
            });
        }
        if matches!(query.aggregation, Some(QueryAggregation::Distinct)) {
            deduplicate_rows(&mut rows);
        }
        if matches!(query.aggregation, Some(QueryAggregation::Count)) {
            let count = rows.len().to_string();
            let evidence = rows
                .iter()
                .flat_map(|row| row.evidence.clone())
                .collect::<Vec<_>>();
            rows = vec![QueryExecutionResultRow {
                columns: BTreeMap::from([
                    ("count".to_string(), count),
                    ("mode".to_string(), "count".to_string()),
                ]),
                evidence,
            }];
        } else {
            sort_rows(&mut rows, &query.ordering);
            if let Some(limit) = query.limit {
                rows.truncate(limit);
            }
        }
        let result = QueryExecutionResult {
            id: QueryResultId(format!("{}:result", plan.id.0)),
            query_id: plan.query_id.clone(),
            plan_id: plan.id.clone(),
            rows,
            diagnostics: Vec::new(),
            result_sha256: String::new(),
        };
        Ok(QueryExecutionResult {
            result_sha256: hash_without_field(&result, "result_sha256")?,
            ..result
        })
    }

    pub fn answer_document_query(
        query: QueryInterlingua,
        knowledge: &DocumentKnowledgeExtraction,
    ) -> Result<DocumentAnswer, QueryError> {
        let query = Self::compile_query(query)?;
        let plan = Self::plan_query(&query)?;
        let execution = Self::execute_query(&plan, &query, knowledge)?;
        let status = answer_status(&query, &execution);
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
        let text = answer_text(&query, &execution, status);
        let answer = DocumentAnswer {
            id: AnswerId(format!("{}:answer", query.id.0)),
            query_id: query.id.clone(),
            plan_id: plan.id.clone(),
            status,
            kind,
            text,
            rows: execution.rows,
            evidence,
            conflicts: Vec::new(),
            answer_sha256: String::new(),
        };
        Ok(DocumentAnswer {
            answer_sha256: hash_without_field(&answer, "answer_sha256")?,
            ..answer
        })
    }
}

fn query_matches(
    constraint: &QueryConstraint,
    claim: &crate::document::knowledge::DocumentClaim,
    occurrences: &[&PropositionOccurrence],
    knowledge: &DocumentKnowledgeExtraction,
) -> bool {
    match constraint {
        QueryConstraint::And(items) => items
            .iter()
            .all(|item| query_matches(item, claim, occurrences, knowledge)),
        QueryConstraint::Or(items) => items
            .iter()
            .any(|item| query_matches(item, claim, occurrences, knowledge)),
        QueryConstraint::Not(item) => !query_matches(item, claim, occurrences, knowledge),
        QueryConstraint::Subject(subject) => &claim.canonical_subject == subject,
        QueryConstraint::Predicate(predicate) => &claim.canonical_predicate == predicate,
        QueryConstraint::Object(object) => &claim.canonical_object == object,
        QueryConstraint::ClaimStatus(status) => &claim.status == status,
        QueryConstraint::Claim(id) => &claim.id == id,
        QueryConstraint::EvidenceRequired(required) => {
            !*required
                || occurrences
                    .iter()
                    .any(|occurrence| !claim_evidence_from_occurrence(occurrence).is_empty())
        }
        QueryConstraint::TextContains(text) => {
            let needle = text.to_lowercase();
            claim_render_text(claim, occurrences)
                .to_lowercase()
                .contains(&needle)
        }
    }
}

fn claim_occurrences<'a>(
    knowledge: &'a DocumentKnowledgeExtraction,
    claim: &crate::document::knowledge::DocumentClaim,
) -> Vec<&'a PropositionOccurrence> {
    claim
        .occurrence_ids
        .iter()
        .filter_map(|occurrence_id| knowledge.proposition_occurrences.get(occurrence_id))
        .collect()
}

fn claim_render_text(
    claim: &crate::document::knowledge::DocumentClaim,
    occurrences: &[&PropositionOccurrence],
) -> String {
    let mut parts = vec![
        render_subject(&claim.canonical_subject),
        render_predicate(&claim.canonical_predicate),
        render_object(&claim.canonical_object),
        render_claim_status(&claim.status),
        render_factuality(&claim.factuality),
        render_world(&claim.world),
    ];
    for occurrence in occurrences {
        parts.extend(occurrence.evidence.iter().cloned());
    }
    parts.join(" ")
}

fn claim_evidence(
    claim: &crate::document::knowledge::DocumentClaim,
    occurrences: &[&PropositionOccurrence],
) -> Vec<String> {
    let mut evidence = vec![format!("claim:{}", claim.id)];
    for occurrence in occurrences {
        evidence.push(format!("occurrence:{}", occurrence.id));
        evidence.push(format!("sentence:{}", occurrence.source_sentence_id));
        for item in &occurrence.evidence {
            evidence.push(item.clone());
        }
    }
    evidence.sort();
    evidence.dedup();
    evidence
}

fn claim_evidence_from_occurrence(occurrence: &PropositionOccurrence) -> Vec<String> {
    let mut evidence = vec![format!("occurrence:{}", occurrence.id)];
    evidence.extend(occurrence.evidence.iter().cloned());
    evidence
}

fn render_subject(subject: &KnowledgeSubjectRef) -> String {
    match subject {
        KnowledgeSubjectRef::EntityCluster(id) => format!("entity_cluster:{id}"),
        KnowledgeSubjectRef::EventCluster(id) => format!("event_cluster:{id}"),
        KnowledgeSubjectRef::Document(id) => format!("document:{id}"),
        KnowledgeSubjectRef::Generic => "generic".to_string(),
        KnowledgeSubjectRef::Unresolved => "unresolved".to_string(),
    }
}

fn render_predicate(predicate: &KnowledgePredicateRef) -> String {
    match predicate {
        KnowledgePredicateRef::Relation(kind) => format!("relation:{kind:?}"),
        KnowledgePredicateRef::Property(value) => format!("property:{value}"),
        KnowledgePredicateRef::Concept(value) => format!("concept:{value}"),
        KnowledgePredicateRef::Custom(value) => format!("custom:{value}"),
    }
}

fn render_object(object: &KnowledgeObjectRef) -> String {
    match object {
        KnowledgeObjectRef::EntityCluster(id) => format!("entity_cluster:{id}"),
        KnowledgeObjectRef::EventCluster(id) => format!("event_cluster:{id}"),
        KnowledgeObjectRef::Value(id) => format!("value:{id}"),
        KnowledgeObjectRef::Concept(value) => format!("concept:{value}"),
        KnowledgeObjectRef::TextLiteral(value) => format!("text:{value}"),
        KnowledgeObjectRef::Boolean(value) => format!("boolean:{value}"),
        KnowledgeObjectRef::Unknown => "unknown".to_string(),
    }
}

fn render_claim_status(status: &ClaimStatus) -> String {
    format!("{status:?}")
}

fn render_factuality(factuality: &crate::document::knowledge::ClaimFactuality) -> String {
    format!("{factuality:?}")
}

fn render_world(world: &crate::document::knowledge::ClaimWorldRef) -> String {
    format!("{world:?}")
}

fn sort_rows(rows: &mut [QueryExecutionResultRow], ordering: &[QueryOrdering]) {
    if ordering.is_empty() {
        rows.sort_by(|a, b| a.columns.get("claim").cmp(&b.columns.get("claim")));
        return;
    }
    rows.sort_by(|left, right| {
        for order in ordering {
            let (key, descending) = match order {
                QueryOrdering::Asc(key) => (key, false),
                QueryOrdering::Desc(key) => (key, true),
            };
            let comparison = left.columns.get(key).cmp(&right.columns.get(key));
            if !comparison.is_eq() {
                return if descending { comparison.reverse() } else { comparison };
            }
        }
        left.columns.get("claim").cmp(&right.columns.get("claim"))
    });
}

fn project_columns(columns: &mut BTreeMap<String, String>, projection: &[QueryProjection]) {
    if projection.is_empty() {
        return;
    }
    let mut keep = BTreeMap::new();
    for item in projection {
        match item {
            QueryProjection::Claim => retain_column(columns, &mut keep, "claim"),
            QueryProjection::Subject => retain_column(columns, &mut keep, "subject"),
            QueryProjection::Predicate => retain_column(columns, &mut keep, "predicate"),
            QueryProjection::Object => retain_column(columns, &mut keep, "object"),
            QueryProjection::Evidence => {
                retain_column(columns, &mut keep, "evidence_count");
                retain_column(columns, &mut keep, "occurrences");
            }
            QueryProjection::Text => retain_column(columns, &mut keep, "summary"),
        }
    }
    if keep.is_empty() {
        return;
    }
    *columns = keep;
}

fn retain_column(
    source: &BTreeMap<String, String>,
    target: &mut BTreeMap<String, String>,
    key: &str,
) {
    if let Some(value) = source.get(key) {
        target.insert(key.to_string(), value.clone());
    }
}

fn deduplicate_rows(rows: &mut Vec<QueryExecutionResultRow>) {
    let mut seen = BTreeMap::<String, QueryExecutionResultRow>::new();
    for row in rows.drain(..) {
        let key = serde_json::to_string(&row.columns).unwrap_or_default();
        seen.entry(key).or_insert(row);
    }
    *rows = seen.into_values().collect();
}

fn answer_status(query: &QueryInterlingua, execution: &QueryExecutionResult) -> AnswerStatus {
    if execution.rows.is_empty() {
        return AnswerStatus::Unknown;
    }
    match query.intent {
        QueryIntent::Count => AnswerStatus::Exact,
        QueryIntent::Boolean => AnswerStatus::Supported,
        QueryIntent::Explain => AnswerStatus::Supported,
        QueryIntent::Contradiction if execution.rows.len() > 1 => AnswerStatus::Conflicting,
        QueryIntent::LookupEntity
        | QueryIntent::LookupProperty
        | QueryIntent::LookupRelation
        | QueryIntent::Temporal
        | QueryIntent::Causal
        | QueryIntent::Comparison
        | QueryIntent::Provenance => {
            if execution.rows.len() == 1 {
                AnswerStatus::Exact
            } else {
                AnswerStatus::Supported
            }
        }
        _ => AnswerStatus::Supported,
    }
}

fn answer_text(query: &QueryInterlingua, execution: &QueryExecutionResult, status: AnswerStatus) -> Option<String> {
    let text = match query.intent {
        QueryIntent::Count => execution
            .rows
            .first()
            .and_then(|row| row.columns.get("count"))
            .cloned()
            .map(|count| format!("count={count}")),
        QueryIntent::Boolean => Some(if execution.rows.is_empty() {
            "unknown".to_string()
        } else {
            "true".to_string()
        }),
        QueryIntent::Explain => Some(format!("matched {} claim(s)", execution.rows.len())),
        QueryIntent::Contradiction => Some(format!("conflicts={}", execution.rows.len())),
        QueryIntent::Provenance => Some(format!("provenance rows={}", execution.rows.len())),
        _ => Some(format!("{} result(s)", execution.rows.len())),
    };
    text.or_else(|| Some(format!("{status:?}")))
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
