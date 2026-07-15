use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use crate::document::knowledge::{
    CivilDate, DecimalValue, KnowledgeObjectRef, KnowledgePredicateRef, KnowledgeSubjectRef,
    KnowledgeValue, PropositionOccurrence,
};
use crate::runtime::DocumentArtifactBundle;

use super::{
    BundleId, ConversationWorkspace, DebugCard, DebugCommand, DebugEntityMatch, DebugEvidence,
    DebugFact, DebugFactRole, DebugInspectTarget, DebugPresentation, DebugPresentationCategory,
    DebugSelectorOutcome, DebugSourceSpan, DebugTurnContext, EngineSession, EvidenceDebugQuery,
    FactsDebugQuery, SessionStore, TraceEvent,
};

pub(super) struct FactsDebugResult {
    pub outcome: DebugSelectorOutcome,
    pub total_facts: usize,
    pub truncated: bool,
    pub facts: Vec<DebugFact>,
    pub page: usize,
    pub page_size: Option<usize>,
    pub offset: usize,
    pub claims_scanned: usize,
    pub omitted_without_evidence: usize,
}

pub(super) fn collect_facts(workspace: &ConversationWorkspace, query: &FactsDebugQuery) -> FactsDebugResult {
    let selector = normalize(&query.selector);
    let mut matches_by_bundle = BTreeMap::<BundleId, Vec<DebugEntityMatch>>::new();
    for (bundle_id, bundle) in &workspace.bundles {
        for cluster in bundle.entity_resolution.clusters.values() {
            let is_match = cluster
                .canonical_name
                .iter()
                .chain(cluster.aliases.iter())
                .any(|name| normalize(name) == selector);
            if is_match {
                matches_by_bundle.entry(bundle_id.clone()).or_default().push(DebugEntityMatch {
                    bundle_id: bundle_id.clone(),
                    entity_cluster_id: cluster.id.to_string(),
                    canonical_name: cluster.canonical_name.clone(),
                    aliases: cluster.aliases.clone(),
                });
            }
        }
    }
    for matches in matches_by_bundle.values_mut() {
        matches.sort_by(|left, right| left.entity_cluster_id.cmp(&right.entity_cluster_id));
    }
    let all_matches = matches_by_bundle.values().flatten().cloned().collect::<Vec<_>>();
    if all_matches.is_empty() {
        return FactsDebugResult {
            outcome: DebugSelectorOutcome::NotFound,
            total_facts: 0,
            truncated: false,
            facts: Vec::new(),
            page: query.page.max(1),
            page_size: query.limit,
            offset: 0,
            claims_scanned: workspace.claims.len(),
            omitted_without_evidence: 0,
        };
    }
    let ambiguous = matches_by_bundle.values().any(|matches| {
        let canonical_names = matches
            .iter()
            .filter_map(|candidate| candidate.canonical_name.as_deref())
            .map(normalize)
            .collect::<BTreeSet<_>>();
        canonical_names.len() > 1
    });
    if ambiguous {
        return FactsDebugResult {
            outcome: DebugSelectorOutcome::Ambiguous {
                candidates: all_matches,
            },
            total_facts: 0,
            truncated: false,
            facts: Vec::new(),
            page: query.page.max(1),
            page_size: query.limit,
            offset: 0,
            claims_scanned: workspace.claims.len(),
            omitted_without_evidence: 0,
        };
    }

    let mut facts = Vec::new();
    let mut omitted_without_evidence = 0;
    for (bundle_id, entity_matches) in &matches_by_bundle {
        let Some(bundle) = workspace.bundles.get(bundle_id) else { continue };
        let entity_ids = entity_matches
            .iter()
            .map(|value| value.entity_cluster_id.as_str())
            .collect::<BTreeSet<_>>();
        for claim_id in &bundle.knowledge.claim_order {
            let Some(claim) = bundle.knowledge.claims.get(claim_id) else { continue };
            let subject_match = matches!(&claim.canonical_subject, KnowledgeSubjectRef::EntityCluster(id) if entity_ids.contains(id.as_str()));
            let object_match = matches!(&claim.canonical_object, KnowledgeObjectRef::EntityCluster(id) if entity_ids.contains(id.as_str()));
            let role_match = match query.role {
                DebugFactRole::Any => subject_match || object_match,
                DebugFactRole::Subject => subject_match,
                DebugFactRole::Object => object_match,
            };
            if !role_match {
                continue;
            }
            let evidence = claim
                .occurrence_ids
                .iter()
                .filter_map(|id| bundle.knowledge.proposition_occurrences.get(id))
                .filter_map(|occurrence| debug_evidence(workspace, bundle, occurrence))
                .collect::<Vec<_>>();
            if evidence.is_empty() {
                omitted_without_evidence += 1;
                continue;
            }
            facts.push(DebugFact {
                claim_id: claim.id.to_string(),
                bundle_id: bundle_id.clone(),
                subject: render_subject(bundle, &claim.canonical_subject),
                predicate: render_predicate(&claim.canonical_predicate),
                object: render_object(bundle, &claim.canonical_object),
                selector_role: selector_role(subject_match, object_match).into(),
                quality: fact_quality(
                    &render_predicate(&claim.canonical_predicate),
                    &format!("{:?}", claim.status),
                    &format!("{:?}", claim.factuality),
                    &format!("{:?}", claim.world),
                    claim.confidence_milli,
                )
                .into(),
                claim_status: format!("{:?}", claim.status),
                factuality: format!("{:?}", claim.factuality),
                world: format!("{:?}", claim.world),
                confidence_milli: claim.confidence_milli,
                evidence,
            });
        }
    }
    facts.sort_by(|left, right| {
        (
            &left.subject,
            &left.predicate,
            &left.object,
            &left.bundle_id,
            &left.claim_id,
        )
            .cmp(&(
                &right.subject,
                &right.predicate,
                &right.object,
                &right.bundle_id,
                &right.claim_id,
            ))
    });
    facts.sort_by(|left, right| {
        fact_group_rank(&left.quality, &left.selector_role)
            .cmp(&fact_group_rank(&right.quality, &right.selector_role))
            .then_with(|| left.predicate.cmp(&right.predicate))
            .then_with(|| left.subject.cmp(&right.subject))
            .then_with(|| left.object.cmp(&right.object))
            .then_with(|| left.bundle_id.cmp(&right.bundle_id))
            .then_with(|| left.claim_id.cmp(&right.claim_id))
    });
    let total_facts = facts.len();
    let page = query.page.max(1);
    let page_size = query.limit;
    let offset = query
        .limit
        .map(|limit| limit.saturating_mul(page.saturating_sub(1)))
        .unwrap_or(0);
    if let Some(limit) = query.limit {
        facts = facts.into_iter().skip(offset).take(limit).collect();
    }
    FactsDebugResult {
        outcome: DebugSelectorOutcome::Matched {
            entities: all_matches,
        },
        total_facts,
        truncated: offset.saturating_add(facts.len()) < total_facts,
        facts,
        page,
        page_size,
        offset,
        claims_scanned: workspace.claims.len(),
        omitted_without_evidence,
    }
}

pub(super) fn collect_evidence(
    workspace: &ConversationWorkspace,
    query: &EvidenceDebugQuery,
) -> Option<DebugFact> {
    let claim_ref = workspace.claims.get(&query.claim_id)?;
    let bundle = workspace.bundles.get(&claim_ref.bundle_id)?;
    let claim = bundle
        .knowledge
        .claims
        .values()
        .find(|claim| claim.id.to_string() == query.claim_id)?;
    let evidence = claim
        .occurrence_ids
        .iter()
        .filter_map(|id| bundle.knowledge.proposition_occurrences.get(id))
        .filter_map(|occurrence| debug_evidence(workspace, bundle, occurrence))
        .collect::<Vec<_>>();
    if evidence.is_empty() {
        return None;
    }
    Some(DebugFact {
        claim_id: claim.id.to_string(),
        bundle_id: claim_ref.bundle_id.clone(),
        subject: render_subject(bundle, &claim.canonical_subject),
        predicate: render_predicate(&claim.canonical_predicate),
        object: render_object(bundle, &claim.canonical_object),
        selector_role: "selected".into(),
        quality: fact_quality(
            &render_predicate(&claim.canonical_predicate),
            &format!("{:?}", claim.status),
            &format!("{:?}", claim.factuality),
            &format!("{:?}", claim.world),
            claim.confidence_milli,
        )
        .into(),
        claim_status: format!("{:?}", claim.status),
        factuality: format!("{:?}", claim.factuality),
        world: format!("{:?}", claim.world),
        confidence_milli: claim.confidence_milli,
        evidence,
    })
}

pub(super) fn facts_presentation(
    command: &DebugCommand,
    result: &FactsDebugResult,
) -> DebugPresentation {
    let mut cards = Vec::new();
    let summary = match &result.outcome {
        DebugSelectorOutcome::NotFound => {
            "Entity not found in the current session. Ingest or translate a source first.".into()
        }
        DebugSelectorOutcome::Ambiguous { candidates } => format!(
            "Selector is ambiguous across {} entity candidates. No facts were selected.",
            candidates.len()
        ),
        DebugSelectorOutcome::Matched { .. } if result.facts.is_empty() => {
            "No evidence-backed facts matched this selector.".into()
        }
        DebugSelectorOutcome::Matched { .. } => format!(
            "Found {} evidence-backed facts; showing {} on page {}.",
            result.total_facts,
            result.facts.len(),
            result.page
        ),
    };
    let mut hints = Vec::new();
    match &result.outcome {
        DebugSelectorOutcome::Matched { .. } => {
            hints.push("Use /evidence <number> to inspect one fact in detail.".into());
        }
        DebugSelectorOutcome::NotFound => {
            hints.push("Use /ingest <title> or ask a factual question first.".into());
        }
        DebugSelectorOutcome::Ambiguous { .. } => {}
    }
    let mut last_group = None::<String>;
    for (index, fact) in result.facts.iter().enumerate() {
        let group = fact_group_label(&fact.quality, &fact.selector_role).to_string();
        if last_group.as_deref() != Some(group.as_str()) {
            cards.push(DebugCard {
                kind: "section".into(),
                title: group.clone(),
                body: Vec::new(),
                fields: Vec::new(),
                evidence: Vec::new(),
                status: None,
                confidence: None,
                artifact_refs: Vec::new(),
            });
            last_group = Some(group);
        }
        cards.push(fact_card(result.offset + index + 1, fact, true));
    }
    if result.truncated {
        cards.push(DebugCard {
            kind: "truncation".into(),
            title: "More facts available".into(),
            body: vec![format!(
                "{} additional facts are available after this page.",
                result
                    .total_facts
                    .saturating_sub(result.offset.saturating_add(result.facts.len()))
            )],
            fields: Vec::new(),
            evidence: Vec::new(),
            status: None,
            confidence: None,
            artifact_refs: Vec::new(),
        });
        let next_page = result.page + 1;
        hints.push(format!("Use /facts <entity> --page {next_page} to continue browsing."));
    }
    if let Some(page_size) = result.page_size {
        hints.push(format!("Use /facts <entity> --limit {page_size} --page N to browse deterministic pages."));
    }
    DebugPresentation {
        category: DebugPresentationCategory::Facts,
        title: match command {
            DebugCommand::Facts(query) => format!("Facts · {}", query.selector),
            _ => "Facts".into(),
        },
        summary,
        status_line: format!(
            "facts={} page={} returned={} scanned={}",
            result.total_facts,
            result.page,
            result.facts.len(),
            result.claims_scanned
        ),
        cards,
        hints,
    }
}

pub(super) fn evidence_presentation(fact: Option<&DebugFact>, claim_id: &str) -> DebugPresentation {
    match fact {
        Some(fact) => DebugPresentation {
            category: DebugPresentationCategory::Evidence,
            title: format!("Evidence · {}", fact.claim_id),
            summary: humanize_fact(fact),
            status_line: format!("evidence={} claim={}", fact.evidence.len(), fact.claim_id),
            cards: vec![fact_card(1, fact, false)],
            hints: vec!["Use /facts <entity> to browse related facts.".into()],
        },
        None => DebugPresentation {
            category: DebugPresentationCategory::Evidence,
            title: format!("Evidence · {claim_id}"),
            summary: "No evidence-backed claim matched this identifier.".into(),
            status_line: "evidence=0".into(),
            cards: Vec::new(),
            hints: vec!["Use /facts <entity> to discover valid claim IDs.".into()],
        },
    }
}

pub(super) fn inspect_presentation(
    session: &EngineSession,
    store: Option<&SessionStore>,
    target: &DebugInspectTarget,
) -> Result<DebugPresentation, String> {
    match target {
        DebugInspectTarget::Interlingua => Ok(interlingua_presentation(session.debug.last_user_turn.as_ref())),
        DebugInspectTarget::Entities => Ok(entities_presentation(session.debug.last_user_turn.as_ref())),
        DebugInspectTarget::Query => Ok(query_presentation(session.debug.last_user_turn.as_ref())),
        DebugInspectTarget::Pipeline => Ok(pipeline_presentation(session.debug.last_user_turn.as_ref())),
        DebugInspectTarget::Sources => Ok(sources_presentation(session)),
        DebugInspectTarget::Snapshot => Ok(snapshot_presentation(session)),
        DebugInspectTarget::Trace { run_id } => trace_presentation(session, store, run_id.as_deref()),
    }
}

fn interlingua_presentation(ctx: Option<&DebugTurnContext>) -> DebugPresentation {
    match ctx {
        Some(ctx) => DebugPresentation {
            category: DebugPresentationCategory::Interlingua,
            title: "Interlingua".into(),
            summary: format!(
                "Last user turn parsed as {} with {} sentence(s).",
                ctx.detected_language.as_deref().unwrap_or("unknown language"),
                ctx.sentence_count
            ),
            status_line: format!("parse={}", ctx.parse_status),
            cards: vec![DebugCard {
                kind: "interlingua".into(),
                title: "Question semantics".into(),
                body: vec![format!("input: {}", ctx.input_text)],
                fields: vec![
                    (
                        "language".into(),
                        ctx.detected_language.clone().unwrap_or_else(|| "unknown".into()),
                    ),
                    ("parse".into(), ctx.parse_status.clone()),
                    (
                        "question_kind".into(),
                        ctx.question_kind.clone().unwrap_or_else(|| "none".into()),
                    ),
                    (
                        "predicate".into(),
                        ctx.predicate.clone().unwrap_or_else(|| "none".into()),
                    ),
                    (
                        "projection".into(),
                        ctx.projection.clone().unwrap_or_else(|| "none".into()),
                    ),
                ],
                evidence: Vec::new(),
                status: None,
                confidence: None,
                artifact_refs: Vec::new(),
            }],
            hints: vec!["Use /debug query to inspect the structured query and plan.".into()],
        },
        None => empty_presentation(
            DebugPresentationCategory::Interlingua,
            "Interlingua",
            "No parsed user turn is available yet.",
        ),
    }
}

fn entities_presentation(ctx: Option<&DebugTurnContext>) -> DebugPresentation {
    match ctx {
        Some(ctx) => DebugPresentation {
            category: DebugPresentationCategory::Entities,
            title: "Entities".into(),
            summary: if ctx.source_candidates.is_empty() {
                "No source candidates or direct entity anchors were captured for the last turn.".into()
            } else {
                format!(
                    "Last turn produced {} source/entity candidate(s).",
                    ctx.source_candidates.len()
                )
            },
            status_line: format!("candidates={}", ctx.source_candidates.len()),
            cards: vec![DebugCard {
                kind: "entities".into(),
                title: "Resolved candidates".into(),
                body: if ctx.source_candidates.is_empty() {
                    vec!["No direct candidates.".into()]
                } else {
                    ctx.source_candidates
                        .iter()
                        .enumerate()
                        .map(|(index, value)| format!("{}. {}", index + 1, value))
                        .collect()
                },
                fields: vec![(
                    "selected_bundle".into(),
                    ctx.selected_bundle_id
                        .clone()
                        .unwrap_or_else(|| "none".into()),
                )],
                evidence: Vec::new(),
                status: None,
                confidence: None,
                artifact_refs: Vec::new(),
            }],
            hints: vec!["Use /facts <entity> to inspect extracted claims for a candidate.".into()],
        },
        None => empty_presentation(
            DebugPresentationCategory::Entities,
            "Entities",
            "No entity resolution context is available yet.",
        ),
    }
}

fn query_presentation(ctx: Option<&DebugTurnContext>) -> DebugPresentation {
    match ctx {
        Some(ctx) => {
            let mut cards = Vec::new();
            cards.push(DebugCard {
                kind: "query".into(),
                title: "Structured query".into(),
                body: describe_structured_query(ctx.query_json.as_deref()),
                fields: vec![(
                    "answer_status".into(),
                    ctx.answer_status.clone().unwrap_or_else(|| "unknown".into()),
                )],
                evidence: Vec::new(),
                status: None,
                confidence: None,
                artifact_refs: Vec::new(),
            });
            if !ctx.plan_operators.is_empty() || !ctx.plan_steps.is_empty() {
                cards.push(DebugCard {
                    kind: "plan".into(),
                    title: "Execution plan".into(),
                    body: ctx
                        .plan_steps
                        .iter()
                        .enumerate()
                        .map(|(index, step)| format!("{}. {}", index + 1, step))
                        .collect(),
                    fields: vec![("operators".into(), ctx.plan_operators.join(", "))],
                    evidence: ctx.execution_rows.clone(),
                    status: None,
                    confidence: None,
                    artifact_refs: Vec::new(),
                });
            }
            DebugPresentation {
                category: DebugPresentationCategory::Query,
                title: "Query".into(),
                summary: "Structured query, plan and execution summary for the last turn.".into(),
                status_line: format!("rows={}", ctx.execution_rows.len()),
                cards,
                hints: vec!["Use /debug pipeline to see the full stage sequence.".into()],
            }
        }
        None => empty_presentation(
            DebugPresentationCategory::Query,
            "Query",
            "No query execution context is available yet.",
        ),
    }
}

fn pipeline_presentation(ctx: Option<&DebugTurnContext>) -> DebugPresentation {
    match ctx {
        Some(ctx) => DebugPresentation {
            category: DebugPresentationCategory::Pipeline,
            title: "Pipeline".into(),
            summary: "Last turn pipeline summary.".into(),
            status_line: format!(
                "parse={} answer={}",
                ctx.parse_status,
                ctx.answer_status.clone().unwrap_or_else(|| "unknown".into())
            ),
            cards: vec![DebugCard {
                kind: "pipeline".into(),
                title: "Stage timeline".into(),
                body: vec![
                    format!(
                        "language.detected -> {}",
                        ctx.detected_language.as_deref().unwrap_or("unknown")
                    ),
                    format!("parse -> {}", ctx.parse_status),
                    format!(
                        "query -> {}",
                        if ctx.query_json.is_some() { "built" } else { "none" }
                    ),
                    format!(
                        "answer -> {}",
                        ctx.answer_status.clone().unwrap_or_else(|| "unknown".into())
                    ),
                ],
                fields: vec![(
                    "run_id".into(),
                    ctx.run_id.clone().unwrap_or_else(|| "unknown".into()),
                )],
                evidence: ctx.diagnostics.clone(),
                status: None,
                confidence: None,
                artifact_refs: ctx
                    .selected_bundle_id
                    .iter()
                    .map(|id| format!("bundle:{id}"))
                    .collect(),
            }],
            hints: vec!["Use /debug trace for persisted raw stage events.".into()],
        },
        None => empty_presentation(
            DebugPresentationCategory::Pipeline,
            "Pipeline",
            "No pipeline context is available yet.",
        ),
    }
}

fn sources_presentation(session: &EngineSession) -> DebugPresentation {
    let mut cards = Vec::new();
    for source in session.conversation.sources.values() {
        cards.push(DebugCard {
            kind: "source".into(),
            title: source.title.clone(),
            body: vec![format!(
                "{} snapshot with sha256 {}.",
                source.language, source.content_sha256
            )],
            fields: vec![
                ("source_id".into(), source.source_id.clone()),
                (
                    "revision".into(),
                    source.revision.clone().unwrap_or_else(|| "none".into()),
                ),
            ],
            evidence: source
                .uri
                .clone()
                .into_iter()
                .map(|uri| format!("uri: {uri}"))
                .collect(),
            status: None,
            confidence: None,
            artifact_refs: vec![source.content_sha256.clone()],
        });
    }
    DebugPresentation {
        category: DebugPresentationCategory::Sources,
        title: "Sources".into(),
        summary: format!(
            "Current conversation workspace has {} active source snapshot(s).",
            session.conversation.sources.len()
        ),
        status_line: format!(
            "sources={} bundles={}",
            session.conversation.sources.len(),
            session.conversation.bundles.len()
        ),
        cards,
        hints: vec!["Use /inspect or /debug snapshot for snapshot-level details.".into()],
    }
}

fn snapshot_presentation(session: &EngineSession) -> DebugPresentation {
    DebugPresentation {
        category: DebugPresentationCategory::Snapshot,
        title: "Snapshot".into(),
        summary: "Current immutable engine session snapshot.".into(),
        status_line: format!("snapshot={}", session.snapshot_id),
        cards: vec![DebugCard {
            kind: "snapshot".into(),
            title: "Workspace summary".into(),
            body: Vec::new(),
            fields: vec![
                ("session_id".into(), session.session_id.clone()),
                ("session_snapshot".into(), session.snapshot_id.clone()),
                (
                    "conversation_snapshot".into(),
                    session.conversation.snapshot_id.clone(),
                ),
                (
                    "translation_snapshot".into(),
                    session.translation.snapshot_id.clone(),
                ),
                ("sources".into(), session.conversation.sources.len().to_string()),
                ("bundles".into(), session.conversation.bundles.len().to_string()),
                (
                    "translation_turns".into(),
                    session.translation.turn_order.len().to_string(),
                ),
            ],
            evidence: Vec::new(),
            status: None,
            confidence: None,
            artifact_refs: vec![
                session.snapshot_sha256.clone(),
                session.conversation.snapshot_sha256.clone(),
                session.translation.snapshot_sha256.clone(),
            ],
        }],
        hints: vec!["Use /debug sources to inspect current source snapshots.".into()],
    }
}

fn trace_presentation(
    session: &EngineSession,
    store: Option<&SessionStore>,
    run_id: Option<&str>,
) -> Result<DebugPresentation, String> {
    let store = store.ok_or_else(|| "session store disabled".to_string())?;
    let (resolved_run_id, trace_text) = match run_id {
        Some(run_id) => (
            run_id.to_string(),
            store
                .load_trace(&session.session_id, run_id)
                .map_err(|error| format!("{error:?}"))?,
        ),
        None => store
            .latest_trace(&session.session_id)
            .map_err(|error| format!("{error:?}"))?,
    };
    let mut cards = Vec::new();
    let mut corrupt = Vec::new();
    let mut count = 0usize;
    for (index, line) in trace_text.lines().enumerate() {
        match serde_json::from_str::<TraceEvent>(line) {
            Ok(event) => {
                count += 1;
                cards.push(DebugCard {
                    kind: "trace_event".into(),
                    title: humanize_trace_stage(&event.stage),
                    body: humanize_trace_payload(&event.stage, event.payload.as_ref()),
                    fields: vec![("sequence".into(), event.sequence.to_string())],
                    evidence: Vec::new(),
                    status: None,
                    confidence: None,
                    artifact_refs: vec![event.request_id],
                });
            }
            Err(error) => corrupt.push(format!("line {}: {}", index + 1, error)),
        }
    }
    if !corrupt.is_empty() {
        cards.push(DebugCard {
            kind: "trace_corrupt".into(),
            title: "Corrupt trace lines".into(),
            body: corrupt.clone(),
            fields: Vec::new(),
            evidence: Vec::new(),
            status: Some("warning".into()),
            confidence: None,
            artifact_refs: Vec::new(),
        });
    }
    Ok(DebugPresentation {
        category: DebugPresentationCategory::Trace,
        title: format!("Trace · {resolved_run_id}"),
        summary: format!("Loaded {} persisted trace event(s).", count),
        status_line: format!("run_id={resolved_run_id}"),
        cards,
        hints: vec!["Alt+V cycles transcript verbosity during live runs.".into()],
    })
}

fn empty_presentation(
    category: DebugPresentationCategory,
    title: &str,
    summary: &str,
) -> DebugPresentation {
    DebugPresentation {
        category,
        title: title.into(),
        summary: summary.into(),
        status_line: "status=unknown".into(),
        cards: Vec::new(),
        hints: Vec::new(),
    }
}

fn fact_card(index: usize, fact: &DebugFact, include_humanized_title: bool) -> DebugCard {
    let title = if include_humanized_title {
        format!("{}. {}", index, humanize_fact(fact))
    } else {
        humanize_fact(fact)
    };
    let mut evidence = Vec::new();
    for item in &fact.evidence {
        evidence.push(format!("Source: {} ({})", item.source_title, item.source_language));
        evidence.push(format!("Sentence: {}", item.sentence_text));
        if !item.source_spans.is_empty() {
            evidence.push(format!(
                "Spans: {}",
                item.source_spans
                    .iter()
                    .map(|span| format!("{}..{} '{}'", span.start, span.end, span.text))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }
    DebugCard {
        kind: "fact".into(),
        title,
        body: vec![
            format!("Claim: {} {} {}", fact.subject, fact.predicate, fact.object),
            format!("Role: {}", prettify_selector_role(&fact.selector_role)),
        ],
        fields: vec![
            ("claim_id".into(), fact.claim_id.clone()),
            ("bundle_id".into(), fact.bundle_id.clone()),
            ("quality".into(), fact.quality.clone()),
            ("status".into(), fact.claim_status.clone()),
            ("factuality".into(), fact.factuality.clone()),
            ("world".into(), fact.world.clone()),
        ],
        evidence,
        status: Some(fact.quality.clone()),
        confidence: Some(format!("{}/1000", fact.confidence_milli)),
        artifact_refs: vec![fact.claim_id.clone(), fact.bundle_id.clone()],
    }
}

fn humanize_fact(fact: &DebugFact) -> String {
    let predicate = normalize_predicate(&fact.predicate);
    match predicate.as_str() {
        "capitalof" => format!("{} is the capital of {}.", fact.subject, fact.object),
        "locatedin" => format!("{} is located in {}.", fact.subject, fact.object),
        "population" => format!("{} has population {}.", fact.subject, fact.object),
        "is_a" | "isa" => format!("{} is {}.", fact.subject, fact.object),
        "flowsthrough" => format!("{} flows through {}.", fact.subject, fact.object),
        "dateof" => format!("{} happened on {}.", fact.subject, fact.object),
        _ => format!("{} {} {}.", fact.subject, prettify_predicate(&fact.predicate), fact.object),
    }
}

fn describe_structured_query(query_json: Option<&str>) -> Vec<String> {
    let Some(query_json) = query_json else {
        return vec!["No structured query was recorded.".into()];
    };
    let Ok(value) = serde_json::from_str::<Value>(query_json) else {
        return vec![query_json.into()];
    };
    let mut lines = Vec::new();
    if let Some(kind) = value.get("kind").and_then(compact_query_value) {
        lines.push(format!("Kind: {kind}"));
    }
    if let Some(predicate) = value.get("predicate").and_then(compact_query_value) {
        lines.push(format!("Predicate: {predicate}"));
    }
    if let Some(subject) = value.get("subject").and_then(compact_query_value) {
        lines.push(format!("Subject: {subject}"));
    }
    if let Some(object) = value.get("object").and_then(compact_query_value) {
        lines.push(format!("Object: {object}"));
    }
    if let Some(projection) = value.get("projection").and_then(compact_query_value) {
        lines.push(format!("Projection: {projection}"));
    }
    if lines.is_empty() {
        lines.push(compact_json(&value));
    }
    lines
}

fn compact_query_value(value: &Value) -> Option<String> {
    match value {
        Value::Null => None,
        Value::String(value) => Some(value.clone()),
        _ => Some(compact_json(value)),
    }
}

fn selector_role(subject_match: bool, object_match: bool) -> &'static str {
    match (subject_match, object_match) {
        (true, true) => "subject+object",
        (true, false) => "subject",
        (false, true) => "object",
        (false, false) => "context",
    }
}

fn fact_quality(
    predicate: &str,
    claim_status: &str,
    factuality: &str,
    world: &str,
    confidence_milli: u16,
) -> &'static str {
    let status = normalize(claim_status);
    let factuality = normalize(factuality);
    let world = normalize(world);
    let predicate = normalize_predicate(predicate);
    if status.contains("conflict") || status.contains("contrad") || status.contains("rejected") {
        return "Contested";
    }
    if factuality != "fact" || world != "actual" {
        return "NonFactual";
    }
    if confidence_milli < 700 || !is_well_known_predicate(&predicate) {
        return "NeedsReview";
    }
    "Supported"
}

fn fact_group_rank(quality: &str, selector_role: &str) -> (u8, u8) {
    let quality_rank = match quality {
        "Supported" => 0,
        "Contested" => 1,
        "NonFactual" => 2,
        _ => 3,
    };
    let role_rank = match selector_role {
        "subject" => 0,
        "object" => 1,
        "subject+object" => 2,
        _ => 3,
    };
    (quality_rank, role_rank)
}

fn fact_group_label(quality: &str, selector_role: &str) -> &'static str {
    match (quality, selector_role) {
        ("Supported", "subject") => "Supported facts where the entity is the subject",
        ("Supported", "object") => "Supported facts where the entity is the object",
        ("Supported", "subject+object") => "Supported self-referential or reciprocal facts",
        ("Contested", _) => "Contested facts",
        ("NonFactual", _) => "Reported, conditional or non-actual facts",
        _ => "Facts that need review",
    }
}

fn prettify_selector_role(value: &str) -> &'static str {
    match value {
        "subject" => "entity is the subject",
        "object" => "entity is the object",
        "subject+object" => "entity appears on both sides",
        "selected" => "selected claim",
        _ => "contextual relation",
    }
}

fn is_well_known_predicate(predicate: &str) -> bool {
    matches!(
        predicate,
        "capitalof"
            | "locatedin"
            | "population"
            | "is_a"
            | "isa"
            | "flowsthrough"
            | "dateof"
            | "hasproperty"
            | "hasvalue"
            | "bornin"
            | "worksfor"
            | "createdby"
    )
}

pub(crate) fn humanize_trace_stage(stage: &str) -> String {
    match stage {
        "request" => "Request accepted".into(),
        "runtime.data_root" => "Runtime configuration".into(),
        "language.detected" => "Language detected".into(),
        "language.parse_failed" => "Language parse failed".into(),
        "source.discovery" => "Source discovery".into(),
        "source.selected" => "Source selected".into(),
        "source.auto.failed" => "Source fetch failed".into(),
        "ingest.start" => "Source ingest started".into(),
        "ingest.bundle" => "Bundle produced".into(),
        "query.plan" => "Query plan built".into(),
        "query.execution" => "Query executed".into(),
        "answer.unknown" => "Answer unresolved".into(),
        "answer.selected" => "Answer selected".into(),
        "debug.selector" => "Debug selector parsed".into(),
        "debug.entity_matches" => "Entity matches resolved".into(),
        "debug.claims.scanned" => "Claims scanned".into(),
        "debug.facts.selected" => "Facts selected".into(),
        "debug.response" => "Debug response prepared".into(),
        "trace.persist_error" => "Trace persistence error".into(),
        other => other.replace('.', " "),
    }
}

pub(crate) fn humanize_trace_payload(stage: &str, payload: Option<&Value>) -> Vec<String> {
    let Some(payload) = payload else { return vec!["No additional details.".into()]; };
    match stage {
        "request" => payload
            .get("request")
            .map(compact_json)
            .map(|value| vec![format!("Request: {value}")])
            .unwrap_or_else(|| vec![compact_json(payload)]),
        "runtime.data_root" => {
            let data_dir = payload.get("data_dir").and_then(Value::as_str).unwrap_or("unknown");
            let offline = payload.get("offline").and_then(Value::as_bool).unwrap_or(false);
            let policy = payload
                .get("source_policy")
                .map(compact_json)
                .unwrap_or_else(|| "unknown".into());
            vec![format!("Data root: {data_dir}"), format!("Offline: {offline}"), format!("Source policy: {policy}")]
        }
        "language.detected" => payload
            .get("language")
            .and_then(Value::as_str)
            .map(|value| vec![format!("Detected language: {value}.")])
            .unwrap_or_else(|| vec![compact_json(payload)]),
        "language.parse_failed" => vec![field_sentence(payload, "error", "Parse error")],
        "source.discovery" => {
            let candidates = payload
                .get("candidates")
                .and_then(Value::as_array)
                .map(|values| values.iter().filter_map(Value::as_str).collect::<Vec<_>>())
                .unwrap_or_default();
            if candidates.is_empty() {
                vec!["No source candidates were derived from the turn.".into()]
            } else {
                vec![format!("Candidate sources: {}.", candidates.join(", "))]
            }
        }
        "source.selected" => {
            let mut lines = Vec::new();
            if let Some(title) = payload.get("title").and_then(Value::as_str) {
                lines.push(format!("Selected source: {title}."));
            }
            if let Some(origin) = payload.get("origin").and_then(Value::as_str) {
                lines.push(format!("Origin: {origin}."));
            }
            if lines.is_empty() { vec![compact_json(payload)] } else { lines }
        }
        "query.plan" => {
            let operators = payload
                .get("operators")
                .and_then(Value::as_array)
                .map(|values| {
                    values
                        .iter()
                        .map(compact_json)
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_else(|| "none".into());
            vec![format!("Operators: {operators}.")]
        }
        "query.execution" => {
            let rows = payload.get("row_count").and_then(Value::as_u64).unwrap_or(0);
            vec![format!("Execution produced {rows} row(s).")]
        }
        "debug.facts.selected" => {
            let total = payload.get("total").and_then(Value::as_u64).unwrap_or(0);
            let returned = payload.get("returned").and_then(Value::as_u64).unwrap_or(0);
            vec![format!("Selected {returned} fact(s) from {total} matching fact(s).")]
        }
        "debug.selector" => {
            let selector = payload.get("selector").and_then(Value::as_str).unwrap_or("unknown");
            let role = payload.get("role").map(compact_json).unwrap_or_else(|| "unknown".into());
            let limit = payload.get("limit").map(compact_json).unwrap_or_else(|| "all".into());
            vec![format!("Selector: {selector}"), format!("Role: {role}"), format!("Limit: {limit}")]
        }
        "debug.entity_matches" => payload
            .get("outcome")
            .and_then(|value| value.get("Matched"))
            .and_then(|value| value.get("entities"))
            .and_then(Value::as_array)
            .map(|entities| vec![format!("Matched {} entity candidate(s).", entities.len())])
            .unwrap_or_else(|| vec![compact_json(payload)]),
        "debug.claims.scanned" => {
            let count = payload.get("count").and_then(Value::as_u64).unwrap_or(0);
            let omitted = payload
                .get("omitted_without_evidence")
                .and_then(Value::as_u64)
                .unwrap_or(0);
            vec![
                format!("Claims scanned: {count}"),
                format!("Without evidence: {omitted}"),
            ]
        }
        "debug.response" => {
            let status = payload.get("status").map(compact_json).unwrap_or_else(|| "unknown".into());
            let diagnostics = payload
                .get("diagnostics")
                .and_then(Value::as_array)
                .map(|values| values.iter().map(compact_json).collect::<Vec<_>>().join(", "))
                .unwrap_or_else(|| "none".into());
            vec![format!("Status: {status}"), format!("Diagnostics: {diagnostics}")]
        }
        _ => vec![compact_json(payload)],
    }
}

fn field_sentence(payload: &Value, key: &str, fallback_label: &str) -> String {
    payload
        .get(key)
        .map(compact_json)
        .map(|value| format!("{fallback_label}: {value}."))
        .unwrap_or_else(|| compact_json(payload))
}

fn debug_evidence(
    workspace: &ConversationWorkspace,
    bundle: &DocumentArtifactBundle,
    occurrence: &PropositionOccurrence,
) -> Option<DebugEvidence> {
    let document = &bundle.compilation.document;
    let sentence_text = document.sentence_text(&occurrence.source_sentence_id)?.trim().to_string();
    if sentence_text.is_empty() {
        return None;
    }
    let source = workspace
        .sources
        .values()
        .find(|source| source.content_sha256 == bundle.source_sha256);
    let mut source_spans = occurrence
        .source_spans
        .iter()
        .filter_map(|span| {
            span.validate_for(&document.source).ok()?;
            Some(DebugSourceSpan {
                start: span.start,
                end: span.end,
                text: document.source.get(span.start..span.end)?.to_string(),
            })
        })
        .collect::<Vec<_>>();
    source_spans.sort_by_key(|span| (span.start, span.end));
    source_spans.dedup_by_key(|span| (span.start, span.end));
    if source_spans.is_empty() {
        return None;
    }
    Some(DebugEvidence {
        source_id: source
            .map(|value| value.source_id.clone())
            .unwrap_or_else(|| bundle.source_document_id.clone()),
        source_title: bundle.source_metadata.title.clone(),
        source_language: bundle.source_metadata.language.clone(),
        source_uri: bundle.source_metadata.uri.clone(),
        source_revision: bundle.source_metadata.revision.clone(),
        source_sha256: bundle.source_sha256.clone(),
        occurrence_id: occurrence.id.to_string(),
        sentence_id: occurrence.source_sentence_id.to_string(),
        sentence_text,
        source_spans,
        derivation: occurrence.evidence.clone(),
    })
}

fn normalize(value: &str) -> String {
    value.trim().to_lowercase()
}

fn normalize_predicate(value: &str) -> String {
    value
        .chars()
        .filter(|char| char.is_ascii_alphanumeric() || *char == '_')
        .flat_map(|char| char.to_lowercase())
        .collect()
}

fn prettify_predicate(value: &str) -> String {
    let mut output = String::new();
    let mut last_lower = false;
    for ch in value.chars() {
        if ch == '_' {
            output.push(' ');
            last_lower = false;
            continue;
        }
        if ch.is_ascii_uppercase() && last_lower {
            output.push(' ');
        }
        output.push(ch.to_ascii_lowercase());
        last_lower = ch.is_ascii_lowercase();
    }
    output
}

fn render_subject(bundle: &DocumentArtifactBundle, value: &KnowledgeSubjectRef) -> String {
    match value {
        KnowledgeSubjectRef::EntityCluster(id) => entity_name(bundle, id.as_str()),
        KnowledgeSubjectRef::EventCluster(id) => format!("event:{id}"),
        KnowledgeSubjectRef::Document(id) => format!("document:{id}"),
        KnowledgeSubjectRef::Generic => "generic".into(),
        KnowledgeSubjectRef::Unresolved => "unknown".into(),
    }
}

fn render_object(bundle: &DocumentArtifactBundle, value: &KnowledgeObjectRef) -> String {
    match value {
        KnowledgeObjectRef::EntityCluster(id) => entity_name(bundle, id.as_str()),
        KnowledgeObjectRef::EventCluster(id) => format!("event:{id}"),
        KnowledgeObjectRef::Value(id) => bundle
            .knowledge
            .values
            .get(id)
            .map(render_value)
            .unwrap_or_else(|| format!("value:{id}")),
        KnowledgeObjectRef::Concept(value) => value.clone(),
        KnowledgeObjectRef::TextLiteral(value) => value.clone(),
        KnowledgeObjectRef::Boolean(value) => value.to_string(),
        KnowledgeObjectRef::Unknown => "unknown".into(),
    }
}

fn entity_name(bundle: &DocumentArtifactBundle, id: &str) -> String {
    bundle
        .entity_resolution
        .clusters
        .values()
        .find(|cluster| cluster.id.as_str() == id)
        .and_then(|cluster| cluster.canonical_name.clone())
        .unwrap_or_else(|| format!("entity:{id}"))
}

fn render_predicate(value: &KnowledgePredicateRef) -> String {
    match value {
        KnowledgePredicateRef::Relation(value) => format!("{:?}", value),
        KnowledgePredicateRef::Property(value)
        | KnowledgePredicateRef::Concept(value)
        | KnowledgePredicateRef::Custom(value) => value.clone(),
    }
}

fn render_value(value: &KnowledgeValue) -> String {
    match value {
        KnowledgeValue::Integer(value) => value.to_string(),
        KnowledgeValue::Decimal(value) => render_decimal(value),
        KnowledgeValue::Range { minimum, maximum } => format!(
            "{}..{}",
            minimum.as_ref().map(render_decimal).unwrap_or_default(),
            maximum.as_ref().map(render_decimal).unwrap_or_default()
        ),
        KnowledgeValue::Approximate(value) => format!("approximately {}", render_value(value)),
        KnowledgeValue::Quantity(value) => format!(
            "{}{}",
            render_decimal(&value.amount),
            value.unit
                .as_ref()
                .map(|unit| format!(" {unit}"))
                .unwrap_or_default()
        ),
        KnowledgeValue::Date(value) => render_date(value),
        KnowledgeValue::Duration { days } => format!("{days} days"),
        KnowledgeValue::Frequency { times, period } => format!("{times} per {period}"),
        KnowledgeValue::Text(value) => value.clone(),
        KnowledgeValue::Boolean(value) => value.to_string(),
        KnowledgeValue::Unknown => "unknown".into(),
    }
}

fn render_decimal(value: &DecimalValue) -> String {
    let sign = if value.sign < 0 { "-" } else { "" };
    if value.scale == 0 {
        return format!("{sign}{}", value.mantissa);
    }
    let scale = value.scale as usize;
    let digits = format!("{:0width$}", value.mantissa, width = scale + 1);
    let split = digits.len() - scale;
    format!("{sign}{}.{}", &digits[..split], &digits[split..])
}

fn render_date(value: &CivilDate) -> String {
    match (value.month, value.day) {
        (Some(month), Some(day)) => format!("{:04}-{month:02}-{day:02}", value.year),
        (Some(month), None) => format!("{:04}-{month:02}", value.year),
        _ => value.year.to_string(),
    }
}

pub(super) fn compact_json(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| value.to_string())
}
