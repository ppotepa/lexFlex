# Document Feature Matrix V1

| ID | Feature | Status | Blocking | CorpusTag | RequiredCases | CurrentEvidence | TargetChapter | Notes |
|---|---|---|---|---|---:|---|---:|---|
| DOC-STR-001 | Multi-sentence input | Required | Yes | multi_sentence | 3 | LexFlexAPI::parse_multi_sentence; tests/digest_tests.rs::test_digest_zero_anaphora_second_clause | 1 | Baseline adapter only |
| DOC-STR-002 | Multi-paragraph input | Required | Yes | multi_paragraph | 3 | LexFlexAPI::parse_multi_sentence; tests/graph_tests.rs::test_multi_sentence_corefers_and_next_sentence | 1 | Preserve blank lines |
| DOC-REF-001 | Repeated proper name | Required | Yes | proper_name_chain | 2 | tests/graph_tests.rs::test_multi_sentence_corefers_and_next_sentence | 1 | Identity chain |
| DOC-REF-002 | Personal pronoun chain | Required | Yes | pronoun_chain | 4 | tests/digest_tests.rs::test_digest_zero_anaphora_second_clause | 1 | Cross-sentence |
| DOC-REF-003 | Polish zero subject | Required | Yes | zero_anaphora | 4 | tests/digest_tests.rs::test_digest_zero_anaphora_second_clause; src/core/context.rs::resolve_discourse_context | 1 | Zero anaphora |
| DOC-TIM-001 | Narrative tense continuity | Required | Yes | tense_continuity | 3 | src/core/summary.rs::summarize_utterance | 1 | Inspect sentence tense |
| DOC-TIM-002 | Temporal sequencing | Required | Yes | temporal_sequence | 3 | tests/integration_test.rs::test_pl_temporal_wczoraj | 1 | yesterday/today/tomorrow |
| DOC-SEM-001 | Preserve negation | Required | Yes | negation | 3 | src/core/interlingua.rs::Sentence::new | 1 | Semantic invariant |
| DOC-SEM-002 | Preserve numbers | Required | Yes | number | 3 | tests/integration_test.rs::test_en_to_pl_age_idiom | 1 | Number exactness |
| DOC-SEM-003 | Preserve semantic roles | Required | Yes | semantic_roles | 4 | src/core/graph.rs::frame_role_entities | 1 | Agent/Patient/Recipient |
| DOC-LEX-001 | Terminology consistency | Required | Yes | terminology | 3 | tests/lexicon_lookup.rs::test_lookup_concept_store_deterministic_in_en_lexicon | 1 | Glossary lock |
| DOC-DIS-001 | Cause/result relation | Required | Yes | causal | 2 | src/core/context.rs::track_discourse | 1 | because/therefore |
| DOC-DIS-002 | Contrast relation | Required | Yes | contrast | 2 | tests/digest_tests.rs::test_digest_multi_clause_coordination_distinct_actors | 1 | contrast chain |
| DOC-CLA-001 | Complement clause | Tracked | No | complement_clause | 2 | LexFlexAPI::parse | 2 | Informational |
| DOC-CLA-002 | Relative clause | Tracked | No | relative_clause | 2 | LexFlexAPI::parse | 2 | Informational |
| DOC-MWE-001 | Multi-word expression | Tracked | No | mwe | 3 | LexFlexAPI::translate | 2 | Informational |
| DOC-FMT-001 | Paragraph preservation | Required | Yes | paragraph_structure | 3 | src/core/interlingua.rs::Sentence::new | 1 | Structure gate |
| DOC-FMT-002 | Rich text formatting | Deferred | No | rich_text | 0 | none | 3 | Deferred |
| DOC-SEM-004 | Quantifiers | Required | Yes | quantifier | 3 | src/core/interlingua.rs::Quantifier | 1 | Exact semantics |
| DOC-SEM-005 | Goal source location | Required | Yes | location_goal | 3 | src/core/graph.rs::frame_role_entities | 1 | Motion roles |
| DOC-SEM-006 | Coordination | Required | Yes | coordination | 3 | tests/graph_tests.rs::test_pl_accompaniment_coordination_in_graph | 1 | and/or |
| DOC-SEM-007 | Definiteness | Tracked | No | definiteness | 2 | tests/integration_test.rs::test_pl_to_en_transfer | 2 | Article choice |
| DOC-SEM-008 | Possession | Required | Yes | possession | 2 | tests/integration_test.rs::test_en_to_pl_transfer | 1 | Have / mieć |
| DOC-SEM-009 | Communication | Tracked | No | communication | 2 | tests/digest_tests.rs::test_digest_multi_clause_coordination_distinct_actors | 2 | Reporting |
| DOC-SEM-010 | Topic continuation | Required | Yes | topic_continuation | 2 | tests/graph_tests.rs::test_persistent_subject_zero_anaphora_translate | 1 | Discourse |
| DOC-SEM-011 | Topic shift | Required | Yes | topic_shift | 2 | tests/graph_tests.rs::test_dialogue_implicit_subject_cross_utterance | 1 | Paragraph boundary |
| DOC-SEM-012 | Instructional sequence | Tracked | No | instruction_sequence | 2 | LexFlexAPI::translate | 2 | Informational |
| DOC-SEM-013 | Factual description | Tracked | No | factual_description | 2 | LexFlexAPI::translate | 2 | Informational |
| DOC-SEM-014 | Numbers and quantities | Required | Yes | numbers_and_quantities | 2 | tests/integration_test.rs::test_empty_input | 1 | Baseline measurable |
| DOC-SEM-015 | Proper names and cities | Required | Yes | proper_names_cities | 2 | tests/integration_test.rs::test_weak_corpus_pl_to_en_proper_noun_locative_warsaw | 1 | City names |
| DOC-SEM-016 | Motion chain | Required | Yes | motion_chain | 2 | tests/graph_tests.rs::test_graph_frame_matches_il_after_age_idiom | 1 | Sequence of motion |
| DOC-SEM-017 | Reporting verb | Tracked | No | reporting_verb | 2 | LexFlexAPI::translate | 2 | Informational |
| DOC-SEM-018 | Standard mixed document | Deferred | No | standard_mixed | 1 | none | 3 | Informational |
