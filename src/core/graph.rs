use crate::core::interlingua::*;
use crate::core::ontology::Ontology;
use crate::data::lexicon::Lexicon;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Edge kinds ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeKind {
    Next,
    Prev,
    Realizes,
    HasRole(SemanticRole),
    CoordinatesWith,
    EvokesConcept,
    PartOfConstruction(String),
    SyntacticHead,
    Dependent,
    NextSentence,
    Corefers,
    InFocus,
    RecentMention,
    ContinuesTopic,
    ModifierOf,
    ComplementOf,
    MatchesPattern(String),
    ConceptRelation(String),
    Introduces,
    Answers,
    Elaborates,
    Contrasts,
}

/// Declarative construction pattern (registered at runtime or from defaults).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstructionPattern {
    pub name: String,
    pub prep_forms: Vec<String>,
    pub concept_filter: Vec<String>,
}

// ─── Node payloads ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WordNode {
    pub id: NodeId,
    pub form: String,
    pub lemma: String,
    pub pos: PartOfSpeech,
    pub features: FeatureBundle,
    pub span: (usize, usize),
    pub next: Option<NodeId>,
    pub prev: Option<NodeId>,
    pub evokes: Option<ConceptId>,
}

impl WordNode {
    /// Navigate to the next surface word (fluent API per graph.md §6).
    pub fn navig_next<'a>(&self, graph: &'a LinguisticGraph) -> Option<&'a WordNode> {
        self.next.and_then(|id| graph.get_word(id))
    }

    /// Navigate to the previous surface word.
    pub fn navig_prev<'a>(&self, graph: &'a LinguisticGraph) -> Option<&'a WordNode> {
        self.prev.and_then(|id| graph.get_word(id))
    }

    pub fn from_token(id: NodeId, token: &Token) -> Self {
        Self {
            id,
            form: token.form.clone(),
            lemma: token.lemma.clone().unwrap_or_else(|| token.form.clone()),
            pos: token.pos,
            features: token.features.clone(),
            span: token.span,
            next: None,
            prev: None,
            evokes: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityNode {
    pub id: NodeId,
    pub concept: ConceptId,
    pub features: FeatureBundle,
    pub name: Option<String>,
    pub realizing_words: Vec<NodeId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrameNode {
    pub id: NodeId,
    pub kind: String,
    pub verb_concept: ConceptId,
    pub roles: Vec<(SemanticRole, NodeId)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoordinationNode {
    pub id: NodeId,
    pub conjunction: String,
    pub member_entities: Vec<NodeId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhraseNode {
    pub id: NodeId,
    pub phrase_type: String,
    pub head: Option<NodeId>,
    pub members: Vec<NodeId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConceptNode {
    pub id: NodeId,
    pub concept: ConceptId,
    pub relations: Vec<(String, ConceptId)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SentenceNode {
    pub id: NodeId,
    pub frame_id: Option<NodeId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClauseNode {
    pub id: NodeId,
    pub sentence_id: Option<NodeId>,
    pub words: Vec<NodeId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UtteranceNode {
    pub id: NodeId,
    pub sentence_ids: Vec<NodeId>,
    pub discourse: Option<Discourse>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiscourseNode {
    pub id: NodeId,
    pub speaker: Option<Entity>,
    pub addressee: Option<Entity>,
    pub entities: Vec<NodeId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GraphNode {
    Word(WordNode),
    Phrase(PhraseNode),
    Clause(ClauseNode),
    Sentence(SentenceNode),
    Utterance(UtteranceNode),
    Discourse(DiscourseNode),
    Entity(EntityNode),
    Frame(FrameNode),
    Coordination(CoordinationNode),
    Concept(ConceptNode),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    pub id: EdgeId,
    pub from: NodeId,
    pub to: NodeId,
    pub kind: EdgeKind,
}

impl GraphNode {
    pub fn word_id(&self) -> Option<NodeId> {
        match self {
            GraphNode::Word(w) => Some(w.id),
            _ => None,
        }
    }
}

/// Direction-aware path builder for construction matching.
pub struct PathBuilder<'a> {
    graph: &'a LinguisticGraph,
    paths: Vec<Vec<NodeId>>,
}

impl<'a> PathBuilder<'a> {
    pub fn new(graph: &'a LinguisticGraph) -> Self {
        Self {
            graph,
            paths: vec![],
        }
    }

    pub fn starting_with_verb(mut self) -> Self {
        self.paths = self
            .graph
            .find_verbs()
            .into_iter()
            .map(|v| vec![v.id])
            .collect();
        self
    }

    pub fn then_preposition(mut self, forms: &[&str]) -> Self {
        let mut next = Vec::new();
        for path in &self.paths {
            if let Some(&vid) = path.last() {
                if let Some(v) = self.graph.get_word(vid) {
                    if let Some(nid) = v.next {
                        if let Some(w) = self.graph.get_word(nid) {
                            if w.pos == PartOfSpeech::Preposition
                                && forms.iter().any(|f| w.form == *f || w.lemma == *f)
                            {
                                let mut p = path.clone();
                                p.push(nid);
                                next.push(p);
                            }
                        }
                    }
                }
            }
        }
        self.paths = next;
        self
    }

    pub fn then_complementizer(mut self, forms: &[&str]) -> Self {
        let mut next = Vec::new();
        for path in &self.paths {
            if let Some(&vid) = path.last() {
                if let Some(v) = self.graph.get_word(vid) {
                    if let Some(nid) = v.next {
                        if let Some(w) = self.graph.get_word(nid) {
                            if matches!(w.pos, PartOfSpeech::Conjunction | PartOfSpeech::Particle)
                                && forms.iter().any(|f| w.form == *f || w.lemma == *f)
                            {
                                let mut p = path.clone();
                                p.push(nid);
                                next.push(p);
                            }
                        }
                    }
                }
            }
        }
        self.paths = next;
        self
    }

    pub fn matching_concept(mut self, concept: &str) -> Self {
        let c = concept.to_uppercase();
        self.paths.retain(|path| {
            path.iter().any(|&nid| {
                self.graph.get_word(nid).and_then(|w| w.evokes.as_ref()).map_or(false, |id| {
                    id.0 == c
                })
            })
        });
        self
    }

    pub fn then_noun_phrase(mut self) -> Self {
        let mut next = Vec::new();
        for path in &self.paths {
            if let Some(&pid) = path.last() {
                if let Some(prep) = self.graph.get_word(pid) {
                    let mut cur = prep.next;
                    let mut p = path.clone();
                    while let Some(nid) = cur {
                        if let Some(w) = self.graph.get_word(nid) {
                            if matches!(
                                w.pos,
                                PartOfSpeech::Noun | PartOfSpeech::Pronoun | PartOfSpeech::Adjective
                            ) {
                                p.push(nid);
                                cur = w.next;
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    if p.len() > path.len() {
                        next.push(p);
                    }
                }
            }
        }
        self.paths = next;
        self
    }

    pub fn paths(self) -> Vec<Vec<NodeId>> {
        self.paths
    }
}

/// Multi-utterance dialogue substrate with cross-utterance edges.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct DialogueGraph {
    pub utterances: Vec<Utterance>,
    pub cross_edges: Vec<Edge>,
    pub utterance_node_ids: Vec<NodeId>,
}

// ─── Tracked entity (parser helper) ─────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TrackedEntity {
    pub entity: Entity,
    pub token_indices: Vec<usize>,
}

impl TrackedEntity {
    pub fn new(entity: Entity, token_indices: Vec<usize>) -> Self {
        Self { entity, token_indices }
    }

    pub fn bare(entity: Entity) -> Self {
        Self { entity, token_indices: vec![] }
    }
}

// ─── LinguisticGraph ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct LinguisticGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<Edge>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub constructions: HashMap<String, ConstructionPattern>,
    #[serde(skip)]
    node_index: HashMap<NodeId, usize>,
    #[serde(skip)]
    next_node: u32,
    #[serde(skip)]
    next_edge: u32,
}

impl LinguisticGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn alloc_node_id(&mut self) -> NodeId {
        let id = NodeId(self.next_node);
        self.next_node += 1;
        id
    }

    fn alloc_edge_id(&mut self) -> EdgeId {
        let id = EdgeId(self.next_edge);
        self.next_edge += 1;
        id
    }

    pub fn add_edge(&mut self, from: NodeId, to: NodeId, kind: EdgeKind) -> EdgeId {
        let id = self.alloc_edge_id();
        self.edges.push(Edge { id, from, to, kind });
        id
    }

    fn rebuild_node_index(&mut self) {
        self.node_index.clear();
        for (i, node) in self.nodes.iter().enumerate() {
            let id = match node {
                GraphNode::Word(w) => w.id,
                GraphNode::Phrase(p) => p.id,
                GraphNode::Clause(c) => c.id,
                GraphNode::Sentence(s) => s.id,
                GraphNode::Utterance(u) => u.id,
                GraphNode::Discourse(d) => d.id,
                GraphNode::Entity(e) => e.id,
                GraphNode::Frame(f) => f.id,
                GraphNode::Coordination(c) => c.id,
                GraphNode::Concept(c) => c.id,
            };
            self.node_index.insert(id, i);
        }
    }

    pub fn node_at(&self, id: NodeId) -> Option<&GraphNode> {
        self.node_index
            .get(&id)
            .and_then(|&i| self.nodes.get(i))
    }

    /// Register a named construction pattern for discovery queries.
    pub fn register_construction(&mut self, pattern: ConstructionPattern) {
        self.constructions.insert(pattern.name.clone(), pattern);
    }

    /// Register built-in construction patterns (accompaniment, age idiom, etc.).
    pub fn register_default_constructions(&mut self) {
        if !self.constructions.is_empty() {
            return;
        }
        self.register_construction(ConstructionPattern {
            name: "ACCOMPANIMENT".into(),
            prep_forms: vec!["z".into(), "with".into(), "razem z".into()],
            concept_filter: vec![],
        });
        self.register_construction(ConstructionPattern {
            name: "AGE_IDIOM".into(),
            prep_forms: vec![],
            concept_filter: vec!["YEAR".into(), "BE".into()],
        });
        // Plan enrichment: lightweight named constructions for demonstratives and copulas.
        // These allow declarative handling instead of many special cases in parser/generator.
        self.register_construction(ConstructionPattern {
            name: "DEMONSTRATIVE_REFERENCE".into(),
            prep_forms: vec![],
            concept_filter: vec!["THIS".into()],
        });
        self.register_construction(ConstructionPattern {
            name: "IDENTIFICATION".into(),
            prep_forms: vec![],
            concept_filter: vec!["BE".into()],
        });
    }

    /// Find nodes participating in a named construction anchored at a word.
    pub fn find_construction(&self, word_id: NodeId, name: &str) -> Option<Vec<NodeId>> {
        if name == "ACCOMPANIMENT" {
            return self
                .find_accompaniment_paths()
                .into_iter()
                .find(|p| p.contains(&word_id))
                .or_else(|| self.find_accompaniment_paths().into_iter().next());
        }
        let pattern = self.constructions.get(name)?;
        if !pattern.prep_forms.is_empty() {
            return PathBuilder::new(self)
                .starting_with_verb()
                .then_preposition(&pattern.prep_forms.iter().map(|s| s.as_str()).collect::<Vec<_>>())
                .then_noun_phrase()
                .paths()
                .into_iter()
                .find(|p| p.contains(&word_id))
                .or_else(|| {
                    PathBuilder::new(self)
                        .starting_with_verb()
                        .then_preposition(&pattern.prep_forms.iter().map(|s| s.as_str()).collect::<Vec<_>>())
                        .then_noun_phrase()
                        .paths()
                        .into_iter()
                        .next()
                });
        }
        if !pattern.concept_filter.is_empty() {
            let paths: Vec<Vec<NodeId>> = self
                .word_nodes()
                .filter(|w| {
                    w.evokes.as_ref().map_or(false, |c| {
                        pattern.concept_filter.iter().any(|f| f == &c.0)
                    })
                })
                .map(|w| vec![w.id])
                .collect();
            return paths.iter().find(|p| p.contains(&word_id)).cloned()
                .or_else(|| paths.first().cloned());
        }
        None
    }

    /// Fluent entry for path queries (graph.md §6).
    pub fn find_paths(&self) -> PathBuilder<'_> {
        PathBuilder::new(self)
    }

    /// Coreference chain for an entity node (bidirectional Corefers edges).
    pub fn coreference_chain(&self, entity_id: NodeId) -> Vec<NodeId> {
        let mut chain = vec![entity_id];
        let mut seen = std::collections::HashSet::from([entity_id]);
        let mut queue = vec![entity_id];
        while let Some(cur) = queue.pop() {
            for e in &self.edges {
                if e.kind == EdgeKind::Corefers && (e.from == cur || e.to == cur) {
                    let other = if e.from == cur { e.to } else { e.from };
                    if seen.insert(other) {
                        chain.push(other);
                        queue.push(other);
                    }
                }
            }
        }
        chain
    }

    /// Entities currently in focus (InFocus edges).
    pub fn in_focus_entities(&self) -> Vec<NodeId> {
        self.edges
            .iter()
            .filter(|e| e.kind == EdgeKind::InFocus)
            .map(|e| e.from)
            .collect()
    }

    /// Recent mention entity ids ordered by edge presence.
    pub fn recent_mention_entities(&self) -> Vec<NodeId> {
        self.edges
            .iter()
            .filter(|e| e.kind == EdgeKind::RecentMention)
            .map(|e| e.from)
            .collect()
    }

    pub fn from_tokens(tokens: &[Token]) -> (Self, Vec<NodeId>) {
        let mut graph = Self::new();
        let mut word_ids = Vec::with_capacity(tokens.len());
        for token in tokens {
            let id = graph.alloc_node_id();
            graph.nodes.push(GraphNode::Word(WordNode::from_token(id, token)));
            word_ids.push(id);
        }
        graph.wire_linear_chain(&word_ids);
        (graph, word_ids)
    }

    pub fn wire_linear_chain(&mut self, word_ids: &[NodeId]) {
        for pair in word_ids.windows(2) {
            let a = pair[0];
            let b = pair[1];
            self.set_word_links(a, Some(b), None);
            self.set_word_links(b, None, Some(a));
            self.add_edge(a, b, EdgeKind::Next);
            self.add_edge(b, a, EdgeKind::Prev);
        }
    }

    fn set_word_links(&mut self, id: NodeId, next: Option<NodeId>, prev: Option<NodeId>) {
        if let Some(GraphNode::Word(w)) = self.nodes.get_mut(id.0 as usize) {
            if next.is_some() {
                w.next = next;
            }
            if prev.is_some() {
                w.prev = prev;
            }
        }
    }

    pub fn word_nodes(&self) -> impl Iterator<Item = &WordNode> {
        self.nodes.iter().filter_map(|n| match n {
            GraphNode::Word(w) => Some(w),
            GraphNode::Clause(_) | GraphNode::Utterance(_) | GraphNode::Discourse(_) | _ => None,
        })
    }

    pub fn word_count(&self) -> usize {
        self.word_nodes().count()
    }

    pub fn get_word(&self, id: NodeId) -> Option<&WordNode> {
        self.nodes.get(id.0 as usize).and_then(|n| match n {
            GraphNode::Word(w) => Some(w),
            GraphNode::Clause(_) | GraphNode::Utterance(_) | GraphNode::Discourse(_) | _ => None,
        })
    }

    pub fn get_entity(&self, id: NodeId) -> Option<&EntityNode> {
        self.nodes.get(id.0 as usize).and_then(|n| match n {
            GraphNode::Entity(e) => Some(e),
            GraphNode::Clause(_) | GraphNode::Utterance(_) | GraphNode::Discourse(_) | _ => None,
        })
    }

    pub fn next_word(&self, id: NodeId, steps: usize) -> Option<&WordNode> {
        let mut cur = id;
        for _ in 0..steps {
            let next = self.get_word(cur)?.next?;
            cur = next;
        }
        self.get_word(cur)
    }

    pub fn prev_word(&self, id: NodeId, steps: usize) -> Option<&WordNode> {
        let mut cur = id;
        for _ in 0..steps {
            let prev = self.get_word(cur)?.prev?;
            cur = prev;
        }
        self.get_word(cur)
    }

    pub fn realizing_words_for_entity(&self, entity_id: NodeId) -> Vec<&WordNode> {
        self.edges
            .iter()
            .filter(|e| e.kind == EdgeKind::Realizes && e.to == entity_id)
            .filter_map(|e| self.get_word(e.from))
            .collect()
    }

    pub fn find_words_by_concept(&self, concept: &str) -> Vec<&WordNode> {
        let c = concept.to_uppercase();
        self.word_nodes()
            .filter(|w| {
                w.evokes.as_ref().map_or(false, |id| id.0 == c)
                    || w.lemma.eq_ignore_ascii_case(concept)
                    || w.form.eq_ignore_ascii_case(concept)
            })
            .collect()
    }

    pub fn find_verbs(&self) -> Vec<&WordNode> {
        self.word_nodes()
            .filter(|w| w.pos == PartOfSpeech::Verb)
            .collect()
    }

    pub fn find_words_with_feature<F>(&self, pred: F) -> Vec<&WordNode>
    where
        F: Fn(&FeatureBundle) -> bool,
    {
        self.word_nodes()
            .filter(|w| pred(&w.features))
            .collect()
    }

    pub fn has_edge_kind(&self, kind: &EdgeKind) -> bool {
        self.edges.iter().any(|e| &e.kind == kind)
    }

    pub fn edges_of_kind(&self, kind: &EdgeKind) -> Vec<&Edge> {
        self.edges.iter().filter(|e| &e.kind == kind).collect()
    }

    pub fn validate_linear_chain(&self, expected_count: usize) -> bool {
        if self.word_count() != expected_count {
            return false;
        }
        let words: Vec<&WordNode> = self.word_nodes().collect();
        if words.is_empty() {
            return expected_count == 0;
        }
        // Walk next chain from first word
        let mut visited = 0;
        let mut cur = words[0].id;
        loop {
            visited += 1;
            if let Some(w) = self.get_word(cur) {
                if let Some(next) = w.next {
                    cur = next;
                } else {
                    break;
                }
            } else {
                return false;
            }
        }
        visited == expected_count
    }

    /// True when any edge marks participation in a canonical construction concept.
    pub fn has_construction(&self, name: &str) -> bool {
        crate::core::constructions::graph_has_construction(self, name)
    }

    /// True when any edge marks participation in a construction concept ID.
    pub fn has_construction_concept(&self, concept: &str) -> bool {
        self.edges.iter().any(|e| {
            matches!(
                &e.kind,
                EdgeKind::PartOfConstruction(c) if c == concept
            )
        })
    }

    pub fn location_uses_instrumental(&self, location_entity_id: NodeId) -> bool {
        self.realizing_words_for_entity(location_entity_id)
            .iter()
            .any(|w| w.features.case == Some(Case::Instrumental))
    }

    pub fn entity_ids(&self) -> Vec<NodeId> {
        self.nodes
            .iter()
            .filter_map(|n| match n {
                GraphNode::Entity(e) => Some(e.id),
                _ => None,
            })
            .collect()
    }

    pub fn frame_ids(&self) -> Vec<NodeId> {
        self.nodes
            .iter()
            .filter_map(|n| match n {
                GraphNode::Frame(f) => Some(f.id),
                _ => None,
            })
            .collect()
    }

    pub fn phrase_nodes(&self) -> impl Iterator<Item = &PhraseNode> {
        self.nodes.iter().filter_map(|n| match n {
            GraphNode::Phrase(p) => Some(p),
            _ => None,
        })
    }

    pub fn concept_nodes(&self) -> impl Iterator<Item = &ConceptNode> {
        self.nodes.iter().filter_map(|n| match n {
            GraphNode::Concept(c) => Some(c),
            _ => None,
        })
    }

    pub fn find_coordination_for_entity(&self, entity_id: NodeId) -> Option<&CoordinationNode> {
        self.nodes.iter().find_map(|n| match n {
            GraphNode::Coordination(c) if c.member_entities.contains(&entity_id) => Some(c),
            _ => None,
        })
    }

    pub fn get_clause(&self, id: NodeId) -> Option<&ClauseNode> {
        self.nodes.get(id.0 as usize).and_then(|n| match n {
            GraphNode::Clause(c) => Some(c),
            _ => None,
        })
    }

    pub fn get_utterance(&self, id: NodeId) -> Option<&UtteranceNode> {
        self.nodes.get(id.0 as usize).and_then(|n| match n {
            GraphNode::Utterance(u) => Some(u),
            _ => None,
        })
    }

    pub fn get_discourse(&self, id: NodeId) -> Option<&DiscourseNode> {
        self.nodes.get(id.0 as usize).and_then(|n| match n {
            GraphNode::Discourse(d) => Some(d),
            _ => None,
        })
    }

    // find_accompaniment_paths already exists via PathBuilder; removed duplicate to avoid E0592.

    pub fn concept_related(&self, concept: &str, relation: &str) -> Vec<ConceptId> {
        let c = concept.to_uppercase();
        self.concept_nodes()
            .filter(|n| n.concept.0 == c)
            .flat_map(|n| {
                n.relations
                    .iter()
                    .filter(|(r, _)| r == relation)
                    .map(|(_, t)| t.clone())
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    /// Light syntactic phrase layer from token groups (NP/PP/VP).
    pub fn materialize_phrases(&mut self, tokens: &[Token], word_ids: &[NodeId]) {
        for (i, token) in tokens.iter().enumerate() {
            if token.pos == PartOfSpeech::Verb {
                if let Some(&wid) = word_ids.get(i) {
                    let pid = self.alloc_node_id();
                    self.nodes.push(GraphNode::Phrase(PhraseNode {
                        id: pid,
                        phrase_type: "VP".into(),
                        head: Some(wid),
                        members: vec![wid],
                    }));
                    self.add_edge(wid, pid, EdgeKind::SyntacticHead);
                    self.add_edge(pid, wid, EdgeKind::Dependent);
                }
            }
        }

        let mut i = 0;
        while i < tokens.len() {
            if matches!(
                tokens[i].pos,
                PartOfSpeech::Noun
                    | PartOfSpeech::Adjective
                    | PartOfSpeech::Determiner
                    | PartOfSpeech::Pronoun
            ) {
                let start = i;
                while i < tokens.len()
                    && matches!(
                        tokens[i].pos,
                        PartOfSpeech::Noun
                            | PartOfSpeech::Adjective
                            | PartOfSpeech::Determiner
                            | PartOfSpeech::Pronoun
                    )
                {
                    i += 1;
                }
                let members: Vec<NodeId> =
                    (start..i).filter_map(|j| word_ids.get(j).copied()).collect();
                if !members.is_empty() {
                    let head = *members.last().unwrap();
                    let pid = self.alloc_node_id();
                    self.nodes.push(GraphNode::Phrase(PhraseNode {
                        id: pid,
                        phrase_type: "NP".into(),
                        head: Some(head),
                        members: members.clone(),
                    }));
                    for &m in &members {
                        if m != head {
                            self.add_edge(m, head, EdgeKind::SyntacticHead);
                            self.add_edge(head, m, EdgeKind::Dependent);
                        }
                    }
                }
            } else if tokens[i].pos == PartOfSpeech::Preposition {
                let prep_i = i;
                i += 1;
                let mut members: Vec<NodeId> = word_ids.get(prep_i).into_iter().copied().collect();
                while i < tokens.len()
                    && matches!(
                        tokens[i].pos,
                        PartOfSpeech::Noun | PartOfSpeech::Adjective | PartOfSpeech::Pronoun
                    )
                {
                    if let Some(&wid) = word_ids.get(i) {
                        members.push(wid);
                    }
                    i += 1;
                }
                if members.len() > 1 {
                    let head = members[0];
                    let pid = self.alloc_node_id();
                    self.nodes.push(GraphNode::Phrase(PhraseNode {
                        id: pid,
                        phrase_type: "PP".into(),
                        head: Some(head),
                        members: members.clone(),
                    }));
                    for &m in &members[1..] {
                        self.add_edge(m, head, EdgeKind::SyntacticHead);
                    }
                    let construction = if tokens[prep_i].form == "z"
                        || tokens[prep_i].form == "with"
                        || tokens[prep_i].form == "razem z"
                    {
                        crate::core::constructions::ACCOMPANIMENT
                    } else {
                        "PrepositionalPhrase"
                    };
                    self.add_edge(
                        *members.last().unwrap(),
                        pid,
                        EdgeKind::PartOfConstruction(construction.to_string()),
                    );
                }
            } else {
                i += 1;
            }
        }
    }

    /// Attach ConceptNodes from ontology and populate WordNode.evokes from lexicon.
    pub fn attach_concept_layer(&mut self, lexicon: &Lexicon, ontology: &Ontology) {
        self.rebuild_node_index();
        let mut concept_index: HashMap<String, NodeId> = HashMap::new();

        for entry in ontology.all_entries() {
            let cid = self.alloc_node_id();
            let mut relations = Vec::new();
            if let Some(parent) = &entry.parent {
                relations.push(("is_a".to_string(), parent.clone()));
            }
            let concept_id = entry.id.clone();
            self.nodes.push(GraphNode::Concept(ConceptNode {
                id: cid,
                concept: concept_id.clone(),
                relations,
            }));
            concept_index.insert(concept_id.0.clone(), cid);
        }

        let word_count = self.nodes.len();
        for i in 0..word_count {
            let (wid, concept) = match &self.nodes[i] {
                GraphNode::Word(w) => {
                    let concept = w.evokes.clone().or_else(|| {
                        lexicon
                            .lookup_by_form(&w.form)
                            .or_else(|| lexicon.lookup_by_lemma(&w.lemma))
                            .map(|e| ConceptId::new(&e.concept))
                    });
                    (w.id, concept)
                }
                _ => continue,
            };
            if let Some(cid) = concept {
                if let GraphNode::Word(wmut) = &mut self.nodes[i] {
                    wmut.evokes = Some(cid.clone());
                }
                if let Some(&cnid) = concept_index.get(&cid.0) {
                    self.add_edge(wid, cnid, EdgeKind::EvokesConcept);
                }
            }
        }

        for entry in ontology.all_entries() {
            if let Some(parent) = &entry.parent {
                if let (Some(&child_id), Some(&parent_id)) = (
                    concept_index.get(&entry.id.0),
                    concept_index.get(&parent.0),
                ) {
                    self.add_edge(child_id, parent_id, EdgeKind::ConceptRelation("is_a".into()));
                }
            }
        }
        self.rebuild_node_index();
    }

    /// Path-based construction matching (accompaniment: verb → prep → noun+).
    pub fn find_accompaniment_paths(&self) -> Vec<Vec<NodeId>> {
        PathBuilder::new(self)
            .starting_with_verb()
            .then_preposition(&["z", "with", "razem z"])
            .then_noun_phrase()
            .paths()
    }

    pub fn path_builder(&self) -> PathBuilder<'_> {
        PathBuilder::new(self)
    }

    /// Materialize semantic layer from frame + tracked entities.
    pub fn materialize_semantic(
        &mut self,
        frame: &Frame,
        tracked: &[TrackedEntity],
        word_ids: &[NodeId],
        verb_token_idx: Option<usize>,
    ) {
        let entity_key = |e: &Entity| -> String {
            format!(
                "{}:{}",
                e.concept.0,
                e.name.as_deref().unwrap_or("")
            )
        };

        let mut entity_node_map: HashMap<String, NodeId> = HashMap::new();

        for te in tracked {
            let eid = self.alloc_node_id();
            let mut realizing = Vec::new();
            for &ti in &te.token_indices {
                if let Some(&wid) = word_ids.get(ti) {
                    realizing.push(wid);
                    self.add_edge(wid, eid, EdgeKind::Realizes);
                }
            }
            // Fallback: match by name/lemma on surface
            if realizing.is_empty() {
                if te.entity.name.is_some() {
                    let matches: Vec<NodeId> = self
                        .word_nodes()
                        .filter(|w| {
                            te.entity.name.as_ref().map_or(false, |n| w.lemma == *n || w.form == *n)
                        })
                        .map(|w| w.id)
                        .collect();
                    for wid in matches {
                        realizing.push(wid);
                        self.add_edge(wid, eid, EdgeKind::Realizes);
                    }
                }
            }
            self.nodes.push(GraphNode::Entity(EntityNode {
                id: eid,
                concept: te.entity.concept.clone(),
                features: te.entity.features.clone(),
                name: te.entity.name.clone(),
                realizing_words: realizing,
            }));
            entity_node_map.insert(entity_key(&te.entity), eid);

            if let Some(coord) = &te.entity.coordination {
                self.materialize_coordination(coord, &entity_key, &mut entity_node_map);
            }
        }

        let frame_id = self.alloc_node_id();
        let verb_concept = frame_verb_concept(frame);
        let mut role_pairs = Vec::new();

        for (role, entity) in frame_role_entities(frame) {
            let key = entity_key(entity);
            if let Some(&enid) = entity_node_map.get(&key) {
                self.add_edge(enid, frame_id, EdgeKind::HasRole(role));
                role_pairs.push((role, enid));
            }
        }

        self.nodes.push(GraphNode::Frame(FrameNode {
            id: frame_id,
            kind: frame.frame_type_name().to_string(),
            verb_concept: ConceptId::new(verb_concept),
            roles: role_pairs,
        }));

        if let Some(vi) = verb_token_idx {
            if let Some(&wid) = word_ids.get(vi) {
                for node in &mut self.nodes {
                    if let GraphNode::Word(w) = node {
                        if w.id == wid {
                            w.evokes = Some(ConceptId::new(verb_concept));
                            self.add_edge(wid, frame_id, EdgeKind::EvokesConcept);
                            break;
                        }
                    }
                }
            }
        }
        self.rebuild_node_index();
        self.register_default_constructions();
    }

    fn materialize_coordination(
        &mut self,
        coord: &Coordination,
        entity_key: &dyn Fn(&Entity) -> String,
        entity_node_map: &mut HashMap<String, NodeId>,
    ) {
        let mut members = Vec::new();
        for item in &coord.items {
            let key = entity_key(item);
            if let Some(&eid) = entity_node_map.get(&key) {
                members.push(eid);
            } else {
                let eid = self.alloc_node_id();
                self.nodes.push(GraphNode::Entity(EntityNode {
                    id: eid,
                    concept: item.concept.clone(),
                    features: item.features.clone(),
                    name: item.name.clone(),
                    realizing_words: vec![],
                }));
                entity_node_map.insert(key, eid);
                members.push(eid);
            }
        }
        if members.len() < 2 {
            return;
        }
        let cid = self.alloc_node_id();
        for i in 0..members.len() - 1 {
            self.add_edge(members[i], members[i + 1], EdgeKind::CoordinatesWith);
        }
        self.nodes.push(GraphNode::Coordination(CoordinationNode {
            id: cid,
            conjunction: coord.conjunction.clone(),
            member_entities: members,
        }));
    }
}

pub fn frame_verb_concept(frame: &Frame) -> &str {
    match frame {
        Frame::Transfer { verb_concept, .. }
        | Frame::Motion { verb_concept, .. }
        | Frame::Creation { verb_concept, .. }
        | Frame::Destruction { verb_concept, .. }
        | Frame::Perception { verb_concept, .. }
        | Frame::Cognition { verb_concept, .. }
        | Frame::Emotion { verb_concept, .. }
        | Frame::Communication { verb_concept, .. }
        | Frame::Statement { verb_concept, .. }
        | Frame::Existence { verb_concept, .. }
        | Frame::Possession { verb_concept, .. }
        | Frame::Consumption { verb_concept, .. } => verb_concept,
        Frame::Custom { .. } => "CUSTOM",
    }
}

pub fn frame_role_entities(frame: &Frame) -> Vec<(SemanticRole, &Entity)> {
    match frame {
        Frame::Transfer { agent, recipient, theme, .. } => vec![
            (SemanticRole::Agent, agent),
            (SemanticRole::Recipient, recipient),
            (SemanticRole::Theme, theme),
        ],
        Frame::Motion { mover, source, goal, path, .. } => {
            let mut v = vec![(SemanticRole::Agent, mover)];
            if let Some(s) = source {
                v.push((SemanticRole::Source, s));
            }
            if let Some(g) = goal {
                v.push((SemanticRole::Goal, g));
            }
            if let Some(p) = path {
                v.push((SemanticRole::Location, p));
            }
            v
        }
        Frame::Creation { creator, created, material, .. } => {
            let mut v = vec![
                (SemanticRole::Creator, creator),
                (SemanticRole::Created, created),
            ];
            if let Some(m) = material {
                v.push((SemanticRole::Instrument, m));
            }
            v
        }
        Frame::Destruction { agent, patient, instrument, .. } => {
            let mut v = vec![
                (SemanticRole::Agent, agent),
                (SemanticRole::Patient, patient),
            ];
            if let Some(i) = instrument {
                v.push((SemanticRole::Instrument, i));
            }
            v
        }
        Frame::Perception { experiencer, stimulus, .. } => vec![
            (SemanticRole::Experiencer, experiencer),
            (SemanticRole::Stimulus, stimulus),
        ],
        Frame::Cognition { cognizer, content, .. } => vec![
            (SemanticRole::Cognizer, cognizer),
            (SemanticRole::Content, content),
        ],
        Frame::Emotion { experiencer, stimulus, .. } => vec![
            (SemanticRole::Experiencer, experiencer),
            (SemanticRole::Stimulus, stimulus),
        ],
        Frame::Communication { speaker, addressee, message, .. } => {
            let mut v = vec![
                (SemanticRole::Speaker, speaker),
                (SemanticRole::Message, message),
            ];
            if let Some(a) = addressee {
                v.push((SemanticRole::Recipient, a));
            }
            v
        }
        Frame::Statement { subject, property, .. } => vec![
            (SemanticRole::Topic, subject),
            (SemanticRole::Theme, property),
        ],
        Frame::Existence { entity, location, .. } => {
            let mut v = vec![(SemanticRole::Theme, entity)];
            if let Some(l) = location {
                v.push((SemanticRole::Location, l));
            }
            v
        }
        Frame::Possession { possessor, possessed, .. } => vec![
            (SemanticRole::Agent, possessor),
            (SemanticRole::Theme, possessed),
        ],
        Frame::Consumption { agent, patient, .. } => vec![
            (SemanticRole::Agent, agent),
            (SemanticRole::Patient, patient),
        ],
        Frame::Custom { roles, .. } => roles.iter().map(|(r, e)| (*r, e)).collect(),
    }
}

/// Locate the EntityNode id matching an IL entity (concept + name).
pub fn find_entity_node_id(graph: &LinguisticGraph, entity: &Entity) -> Option<NodeId> {
    graph.nodes.iter().find_map(|n| match n {
        GraphNode::Entity(e) if e.concept == entity.concept && e.name == entity.name => Some(e.id),
        _ => None,
    })
}

/// True when graph has a CoordinationNode linking this entity.
pub fn has_coordination_topology(graph: &LinguisticGraph, entity: &Entity) -> bool {
    find_entity_node_id(graph, entity)
        .and_then(|id| graph.find_coordination_for_entity(id))
        .is_some()
}

/// Reconstruct coordination members from graph topology (CoordinatesWith chain).
pub fn coordination_from_graph(
    graph: &LinguisticGraph,
    entity: &Entity,
) -> Option<(String, Vec<Entity>)> {
    let eid = find_entity_node_id(graph, entity)?;
    let coord = graph.find_coordination_for_entity(eid)?;
    let items: Vec<Entity> = coord
        .member_entities
        .iter()
        .filter_map(|mid| entity_node_to_il(graph, *mid))
        .collect();
    if items.len() < 2 {
        return None;
    }
    Some((coord.conjunction.clone(), items))
}

fn entity_node_to_il(graph: &LinguisticGraph, id: NodeId) -> Option<Entity> {
    graph.nodes.iter().find_map(|n| match n {
        GraphNode::Entity(e) if e.id == id => Some(Entity {
            concept: e.concept.clone(),
            name: e.name.clone(),
            features: e.features.clone(),
            reference: Reference::Direct,
            id: None,
            coordination: None,
            adjectives: vec![],
        }),
        _ => None,
    })
}

/// Subject/agent uses plural agreement when graph shows coordination or plural features.
pub fn entity_needs_plural_agreement(entity: &Entity, graph: Option<&LinguisticGraph>) -> bool {
    if entity.features.number == Some(Number::Plural) {
        return true;
    }
    if let Some(g) = graph {
        if has_coordination_topology(g, entity) {
            return true;
        }
    }
    false
}

/// Match an entity to contributing token indices by name/lemma/form.
pub fn match_entity_to_tokens(entity: &Entity, tokens: &[Token]) -> Vec<usize> {
    let name = entity.name.as_deref().unwrap_or("");
    if name.is_empty() {
        return vec![];
    }
    tokens
        .iter()
        .enumerate()
        .filter(|(_, t)| {
            let lemma = t.lemma.as_deref().unwrap_or(&t.form);
            lemma == name || t.form == name || lemma.starts_with(name) || t.form.starts_with(name)
        })
        .map(|(i, _)| i)
        .collect()
}

/// Collect all entities from a frame plus nested coordination items.
pub fn collect_frame_entities(frame: &Frame) -> Vec<Entity> {
    let mut out = Vec::new();
    for e in frame.entities() {
        out.push((*e).clone());
        if let Some(coord) = &e.coordination {
            for item in &coord.items {
                out.push(item.clone());
            }
        }
    }
    out
}

/// Serializable snapshot for benchmark traces.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphSnapshot {
    pub words: Vec<WordNode>,
    pub phrases: Vec<PhraseNode>,
    pub entities: Vec<EntityNode>,
    pub frames: Vec<FrameNode>,
    pub coordinations: Vec<CoordinationNode>,
    pub concepts: Vec<ConceptNode>,
    pub edges: Vec<Edge>,
}

impl From<&LinguisticGraph> for GraphSnapshot {
    fn from(g: &LinguisticGraph) -> Self {
        let mut words = Vec::new();
        let mut phrases = Vec::new();
        let mut entities = Vec::new();
        let mut frames = Vec::new();
        let mut coordinations = Vec::new();
        let mut concepts = Vec::new();
        for node in &g.nodes {
            match node {
                GraphNode::Word(w) => words.push(w.clone()),
                GraphNode::Phrase(p) => phrases.push(p.clone()),
                GraphNode::Clause(_) | GraphNode::Utterance(_) | GraphNode::Discourse(_) | GraphNode::Sentence(_) => {}
                GraphNode::Entity(e) => entities.push(e.clone()),
                GraphNode::Frame(f) => frames.push(f.clone()),
                GraphNode::Coordination(c) => coordinations.push(c.clone()),
                GraphNode::Concept(c) => concepts.push(c.clone()),
            }
        }
        Self {
            words,
            phrases,
            entities,
            frames,
            coordinations,
            concepts,
            edges: g.edges.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_chain_internal() {
        let tokens = vec![
            Token {
                form: "a".into(),
                lemma: Some("a".into()),
                pos: PartOfSpeech::Noun,
                features: FeatureBundle::default(),
                span: (0, 1),
                word_node_id: None,
            },
            Token {
                form: "b".into(),
                lemma: Some("b".into()),
                pos: PartOfSpeech::Verb,
                features: FeatureBundle::default(),
                span: (2, 3),
                word_node_id: None,
            },
        ];
        let (graph, ids) = LinguisticGraph::from_tokens(&tokens);
        assert_eq!(graph.word_count(), 2);
        assert!(graph.validate_linear_chain(2));
        assert!(graph.next_word(ids[0], 1).is_some());
        assert!(graph.prev_word(ids[1], 1).is_some());
    }
}
