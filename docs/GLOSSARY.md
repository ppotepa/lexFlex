# Glossary of Terms

This glossary defines key terms used throughout the lexFlex documentation.

**Note:** Terms with links to [DISCOURSE.md](./DISCOURSE.md) are **Feature v0.2+** (not in MVP v0.1). These terms relate to discourse context and multi-sentence processing.

---

## A

### Aktionsart
The inherent aspectual class of a verb, describing how the action unfolds over time. Categories: State, Activity, Accomplishment, Achievement, Semelfactive. See [LEXICON.md](./LEXICON.md#aktionsart-verb-aspectual-classes).

### Anaphora
A linguistic expression that refers to another expression (its antecedent). Example: "him" in "Tom saw him" refers to someone previously mentioned. See [DISCOURSE.md](./DISCOURSE.md#coreference-resolution).

### Anaphor
A type of pronoun that must be bound within its local domain (Principle A of Binding Theory). Examples: "himself", "herself", "siebie". See [DISCOURSE.md](./DISCOURSE.md#binding-theory-extended).

### Applicative
A voice transformation that adds a beneficiary or recipient argument to the verb. See [GRAMMAR_CASES.md](./GRAMMAR_CASES.md#voice).

### Aspect
Grammatical category indicating how an action unfolds over time: Perfective (completed), Imperfective (ongoing), Progressive, Habitual. See [INTERLINGUA.md](./INTERLINGUA.md).

### Antipassive
A voice transformation that demotes or omits the patient, focusing on the agent. Common in ergative languages. See [GRAMMAR_CASES.md](./GRAMMAR_CASES.md#voice).

---

## B

### Binding Theory
A set of principles (A, B, C) that constrain coreference relations between pronouns, anaphors, and R-expressions. See [DISCOURSE.md](./DISCOURSE.md#binding-theory-extended).

### Binding Domain
The syntactic domain within which binding principles apply. Typically the smallest clause containing the governor and a subject. See [DISCOURSE.md](./DISCOURSE.md#binding-domains).

---

## C

### Case
Grammatical category indicating the syntactic role of a noun phrase. Polish has 7 cases (NOM, GEN, DAT, ACC, INST, LOC, VOC). Finnish has 15. See [GRAMMAR_CASES.md](./GRAMMAR_CASES.md).

### Causative
A voice transformation that adds a causer argument. "X verb" → "Y make X verb". See [GRAMMAR_CASES.md](./GRAMMAR_CASES.md#voice).

### Cleft Sentence
A sentence structure that highlights a particular element. Examples: "It was Tom who gave the apple" (it-cleft), "What Tom gave was an apple" (wh-cleft). See [INTERLINGUA.md](./INTERLINGUA.md#cleft-sentences).

### Collocation
A combination of words that occur together more often than by chance. Examples: "strong coffee", "heavy rain". See [LEXICON.md](./LEXICON.md#collocations).

### Complement Clause
A clause that functions as a complement to a verb. Example: "Tom said [that he gave the apple]". See [INTERLINGUA.md](./INTERLINGUA.md#complement-clauses).

### Constituent
A word or group of words that function as a single unit within a hierarchical structure. See [INTERLINGUA.md](./INTERLINGUA.md#constituency-tree).

### Control Structure
A construction where the subject of an embedded clause is controlled by an argument of the matrix verb. Example: "Tom tried [PRO to give the apple]" (Tom controls PRO). See [INTERLINGUA.md](./INTERLINGUA.md#control-and-raising).

### Coordination
The joining of two or more constituents with a coordinator (and, but, or). See [INTERLINGUA.md](./INTERLINGUA.md#coordination).

### Coreference
When two or more expressions refer to the same entity. Example: "Tom" and "he" in "Tom came. He was tired." See [DISCOURSE.md](./DISCOURSE.md#coreference-resolution).

---

## D

### Deduction
The process of resolving ambiguity and filling in implicit information during parsing. Includes verb frame matching, case/role resolution, pronoun resolution (within sentence), temporal anchoring, ontology validation. See [DEDUCTION.md](./DEDUCTION.md).

### Definiteness
Grammatical category indicating whether a noun phrase refers to a specific, identifiable entity. English marks this with articles (a/an vs. the). See [INTERLINGUA.md](./INTERLINGUA.md#featurebundle).

### Dependency Tree
A syntactic representation where words are connected by dependency relations (subject, object, modifier). See [INTERLINGUA.md](./INTERLINGUA.md#dependency-tree).

### Discourse
**⚠️ FEATURE - v0.2+**

The context of a conversation, including participants, entities mentioned, topic, and shared knowledge. See [DISCOURSE.md](./DISCOURSE.md).

### Discourse Marker
**⚠️ FEATURE - v0.2+**

Words or phrases that signal relationships between utterances. Examples: "however", "therefore", "moreover". See [DISCOURSE.md](./DISCOURSE.md#discourse-markers).

---

## E

### Ellipsis
The omission of words that are recoverable from context. Types: Gapping, VP ellipsis, Sluicing, NP ellipsis. See [INTERLINGUA.md](./INTERLINGUA.md#ellipsis).

### Empty Category
A syntactic position that has no phonetic content but plays a grammatical role. Types: PRO (subject of infinitival), pro (dropped subject), trace (left by movement). See [INTERLINGUA.md](./INTERLINGUA.md#syntactic-movement).

### Entity
A thing that can participate in a semantic role. Can be concrete (person, object) or abstract (event, property). See [INTERLINGUA.md](./INTERLINGUA.md#entity).

### Evidentiality
Grammatical category indicating the source of information: direct (witnessed), reported (heard from others), inferred (deduced from evidence). Common in Turkish, Japanese, Korean. See [INTERLINGUA_UNIVERSALITY.md](./INTERLINGUA_UNIVERSALITY.md).

---

## F

### FeatureBundle
A collection of grammatical features (gender, number, case, tense, aspect, etc.) that describe an entity or predicate. See [INTERLINGUA.md](./INTERLINGUA.md#featurebundle).

### Focus
The part of a sentence that carries new or emphasized information. See [DISCOURSE.md](./DISCOURSE.md#information-structure).

### Frame
A predicate-argument structure that represents an event, state, or relationship. Examples: Transfer, Motion, Perception. See [INTERLINGUA.md](./INTERLINGUA.md#frame).

---

## G

### Gapping
A type of ellipsis in coordination where the verb is omitted from the second conjunct. Example: "Tom gave an apple to Iza, and Iza __ a book to Tom". See [INTERLINGUA.md](./INTERLINGUA.md#ellipsis).

### Given Information
Information that is already established in the discourse. Contrasts with new information. See [DISCOURSE.md](./DISCOURSE.md#information-structure).

---

## H

### Honorifics
Grammatical or lexical markers of social status, politeness, or respect. Common in Japanese, Korean, Javanese. See [INTERLINGUA_UNIVERSALITY.md](./INTERLINGUA_UNIVERSALITY.md).

---

## I

### Idiom
A fixed expression whose meaning cannot be derived from its parts. Examples: "kick the bucket" (die), "dać plamę" (fail). See [LEXICON.md](./LEXICON.md#idioms).

### Illocution
The communicative function of an utterance: Statement, Question, Command, Exclamation. See [INTERLINGUA.md](./INTERLINGUA.md).

### Information Structure
How information is organized in terms of topic, focus, given, and new. See [DISCOURSE.md](./DISCOURSE.md#information-structure).

### Interlingua
The language-neutral semantic core of lexFlex. A universal representation of meaning that all language plugins map to/from. See [INTERLINGUA.md](./INTERLINGUA.md).

### InterlinguaNode
The top-level enum representing different types of meaning: Natural (utterances), MathExpression, LogicalProposition, ProgramStmt. See [INTERLINGUA.md](./INTERLINGUA.md#interlinguanode--universal-meaning-representation).

---

## L

### Light Verb Construction
A construction where a verb with minimal semantic content combines with a noun to form a predicate. Examples: "make a decision", "podjąć decyzję". See [LEXICON.md](./LEXICON.md#light-verb-constructions).

---

## M

### Mention
A specific occurrence of an entity in text. Can be a proper name, pronoun, definite description, etc. See [DISCOURSE.md](./DISCOURSE.md).

### Middle Voice
A voice where the subject is affected by the action, but the agent is unspecified. Example: "The book sells well". See [GRAMMAR_CASES.md](./GRAMMAR_CASES.md#voice).

### Modality
Grammatical category indicating the speaker's attitude toward the proposition: Realis (factual), Irrealis (hypothetical), Epistemic (possibility), Deontic (obligation). See [INTERLINGUA.md](./INTERLINGUA.md).

### Mood
Grammatical category indicating the type of speech act: Indicative (statements), Subjunctive (hypothetical), Imperative (commands), Conditional. See [INTERLINGUA.md](./INTERLINGUA.md#conditional-and-subjunctive).

### Multi-Word Expression (MWE)
A fixed or semi-fixed phrase that functions as a single unit. Includes idioms, collocations, light verb constructions, phrasal verbs. See [LEXICON.md](./LEXICON.md#multi-word-expressions-mwe).

---

## N

### New Information
Information that is being introduced into the discourse. Contrasts with given information. See [DISCOURSE.md](./DISCOURSE.md#information-structure).

---

## O

### Ontology
A hierarchical organization of concepts using IS_A and PART_OF relations. Used for semantic validation and type checking. See [ONTOLOGY.md](./ONTOLOGY.md).

---

## P

### Passive Voice
A voice transformation where the patient/theme becomes the subject, and the agent is demoted to a by-phrase or omitted. See [GRAMMAR_CASES.md](./GRAMMAR_CASES.md#voice).

### Phrasal Verb
A multi-word verb consisting of a verb + particle that creates a new meaning. Examples: "give up", "look for". See [LEXICON.md](./LEXICON.md#phrasal-verbs-english).

### Polarity
Whether a sentence is positive or negative. See [INTERLINGUA.md](./INTERLINGUA.md).

### Polysemy
When a single word has multiple related meanings. Example: "bank" (financial institution vs. river bank). See [LEXICON.md](./LEXICON.md#polysemy-resolution).

### Pro-drop
The ability to omit the subject pronoun when it's recoverable from context. Common in Polish, Spanish, Japanese. See [LANGUAGE_DESCRIPTOR.md](./LANGUAGE_DESCRIPTOR.md).

### Pronoun
A word that refers to an entity without naming it. Types: personal, reflexive, possessive, demonstrative, relative, interrogative. See [PRONOUNS.md](./PRONOUNS.md).

---

## Q

### Quantifier
A word that indicates quantity or scope: Universal (all, every), Existential (some, a), Negated (none, no), Numerical (three, five). See [QUANTIFICATION.md](./QUANTIFICATION.md).

### Quantified Statement
A statement with a quantifier. Example: "All students passed the exam" (∀x (Student(x) → Pass(x))). See [QUANTIFICATION.md](./QUANTIFICATION.md).

### Question
An utterance that requests information. Types: Yes/No, Wh-questions, Alternative, Rhetorical. See [INTERLINGUA.md](./INTERLINGUA.md#wh-movement-and-questions).

---

## R

### R-expression
A referring expression (proper name, definite description) that must be free everywhere (Principle C of Binding Theory). Examples: "Tom", "Iza". See [DISCOURSE.md](./DISCOURSE.md#binding-theory-extended).

### Raising
A construction where the subject of an embedded clause is "raised" to become the subject of the matrix verb. Example: "Tom seems [__ to have given the apple]". See [INTERLINGUA.md](./INTERLINGUA.md#control-and-raising).

### Reciprocal
A voice transformation indicating that multiple agents are acting on each other. Example: "Tom and Iza love each other". See [GRAMMAR_CASES.md](./GRAMMAR_CASES.md#voice).

### Reflexive
A voice transformation where the agent and patient are the same entity. Example: "Tom washes himself". See [GRAMMAR_CASES.md](./GRAMMAR_CASES.md#voice).

### Relative Clause
A clause that modifies a noun. Example: "The apple [that Tom gave] was red". See [INTERLINGUA.md](./INTERLINGUA.md#relative-clauses).

### Reported Speech
Speech that reports what someone said. Types: Direct ("He said: 'I gave'"), Indirect ("He said that he gave"), Free Indirect. See [INTERLINGUA.md](./INTERLINGUA.md#reported-speech).

### Role (Semantic Role)
The function of an entity in a frame. Examples: Agent, Patient, Theme, Recipient, Experiencer. See [INTERLINGUA.md](./INTERLINGUA.md#semantic-roles).

---

## S

### Salience
How prominent an entity is in the discourse. Based on recency, frequency, grammatical role, topicality. See [DISCOURSE.md](./DISCOURSE.md).

### Scope
The range of a quantifier or operator. Scope ambiguity occurs when multiple quantifiers can have different relative scopes. See [QUANTIFICATION.md](./QUANTIFICATION.md#scope).

### Scrambling
Free word order variation in languages like Polish, German, Japanese. Used to mark information structure. See [INTERLINGUA.md](./INTERLINGUA.md#information-structure-extended).

### Semantic Role
See Role.

### Sentence
A unit of utterance containing one or more frames. See [INTERLINGUA.md](./INTERLINGUA.md).

### Sluicing
A type of ellipsis where everything except a wh-word is omitted. Example: "Someone gave an apple, but I don't know who __". See [INTERLINGUA.md](./INTERLINGUA.md#ellipsis).

### Speech Act
The communicative function of an utterance: Assert, Question, Request, Command, Offer, Promise, etc. See [SPEECH_ACTS.md](./SPEECH_ACTS.md).

### Subcategorization Frame
The syntactic requirements of a verb (what cases/positions it requires for its arguments). See [LEXICON.md](./LEXICON.md).

### Syntax Tree
A hierarchical representation of sentence structure. Can be constituency tree or dependency tree. See [INTERLINGUA.md](./INTERLINGUA.md#syntax-tree).

---

## T

### Tense
Grammatical category indicating when an action occurs: Past, Present, Future. See [INTERLINGUA.md](./INTERLINGUA.md).

### Temporal Reference
How time is expressed in language: absolute (specific date), relative (yesterday, tomorrow), deictic (now, then). See [TEMPORAL.md](./TEMPORAL.md).

### Topic
What a sentence is about. The entity that the sentence provides information about. See [DISCOURSE.md](./DISCOURSE.md#information-structure).

### Topicalization
Moving the topic to the front of the sentence. Example: "The apple, Tom gave to Iza". See [INTERLINGUA.md](./INTERLINGUA.md#information-structure-extended).

### Trace
An empty category left behind when a constituent moves. Example: "What did Tom give __?" (trace in object position). See [INTERLINGUA.md](./INTERLINGUA.md#syntactic-movement).

---

## U

### Utterance
A complete communicative act, containing one or more sentences and discourse context. See [INTERLINGUA.md](./INTERLINGUA.md).

---

## V

### Valency
The number and type of arguments a verb requires. See [INTERLINGUA.md](./INTERLINGUA.md#voice-and-valency).

### Voice
Grammatical category indicating the relationship between the action and participants. Types: Active, Passive, Middle, Reflexive, Causative, Applicative. See [GRAMMAR_CASES.md](./GRAMMAR_CASES.md#voice).

### VP Ellipsis
A type of ellipsis where the verb phrase is omitted. Example: "Tom gave an apple, and Iza did __ too". See [INTERLINGUA.md](./INTERLINGUA.md#ellipsis).

---

## W

### Wh-Movement
The syntactic movement of wh-words to the front of questions. Example: "What did Tom give?" (what moved from object position). See [INTERLINGUA.md](./INTERLINGUA.md#wh-movement-and-questions).

### Wh-Question
A question formed with a wh-word (who, what, where, when, why, how). See [INTERLINGUA.md](./INTERLINGUA.md#wh-movement-and-questions).

### Word Order
The arrangement of words in a sentence. Different languages have different default orders: SVO (English, Polish), SOV (Japanese, German subordinate), VSO (Arabic). See [INTERLINGUA_UNIVERSALITY.md](./INTERLINGUA_UNIVERSALITY.md).

---

## See Also

- [INTERLINGUA.md](./INTERLINGUA.md) - Core Interlingua specification
- [ARCHITECTURE.md](./ARCHITECTURE.md) - System architecture
- [ENGINE.md](./ENGINE.md) - Language engine architecture
- [LEXICON.md](./LEXICON.md) - Lexicon structure
- [MORPHOLOGY.md](./MORPHOLOGY.md) - Morphological system
- [GRAMMAR_CASES.md](./GRAMMAR_CASES.md) - Case system
- [DISCOURSE.md](./DISCOURSE.md) - Discourse management
- [TEMPORAL.md](./TEMPORAL.md) - Temporal reasoning
- [QUANTIFICATION.md](./QUANTIFICATION.md) - Quantification system
- [PRONOUNS.md](./PRONOUNS.md) - Pronoun system
- [ONTOLOGY.md](./ONTOLOGY.md) - Concept ontology
- [ERROR_HANDLING.md](./ERROR_HANDLING.md) - Error handling
- [LOGGING.md](./LOGGING.md) - Logging framework
- [LANGUAGE_DESCRIPTOR.md](./LANGUAGE_DESCRIPTOR.md) - Language descriptors
- [INTERLINGUA_UNIVERSALITY.md](./INTERLINGUA_UNIVERSALITY.md) - Universality principle
