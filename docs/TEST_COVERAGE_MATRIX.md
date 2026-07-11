# Test Coverage Matrix - Full Bidirectional Algorithmic Coverage (Adjectives + Enumerations Focus)

**Goal**: Exhaustive, **concept-driven** test coverage for the entire PL ↔ Interlingua ↔ EN pipeline.  
**Focus**: adjectives (przymiotniki via Entity.adjectives + FeatureBundle.degree from 11 concepts) and enumerations/wyliczenia (Coordination with "i"/"and", shared/per/mixed adj distribution).  
**Strict rule**: All matrices and future test generation use ONLY concepts from data/concepts/concepts.ron + lexicon entries + RON rules + IL structures (Entity, Coordination, Frame, FeatureBundle). NO surface string hacks in code, tests, or matrix descriptions (except true lexical data like suppletive_comparative: "lepsz").

**Current iteration (2026-07-10)**: Expanded IL to ~70 rows, P/G matrices, checklists, added algorithmic 500-plan. See section 5 for gaps + criteria.

**Principles** (from 21 points):
- All coverage described at **concept + feature** level, never surface strings.
- Parser (PL/EN → IL): Must produce correct `Entity`, `adjectives: Vec<Entity>`, `Coordination`, `Frame`, `FeatureBundle` etc.
- Generator (IL → PL/EN): Must realize correctly using lexicon + RON + realizer (only suppletives/exceptions from lexicon).
- Only true lexical exceptions (e.g. `dobry` → `lepszy` via `suppletive_comparative` or explicit entry) are non-computable.
- Everything else (agreement, stacking, degree, coordination distribution, case, number effects, articles) must be algorithmic.

**Scope**: Full possible coverage of:
- NP structure (head + adjectives, stacking, degree)
- Enumerations / Coordination (on any role, with adjectives)
- All supported Frames and roles
- Feature combinations (degree, number, gender, case, quantification, polarity, illocution, tense, aspect, temporal, etc.)
- Interactions (adjectives + coordination + degree + quant + negation + questions)

---

## 1. Core IL Structures to Cover (Pure Interlingua Level)

This matrix describes the **IL structures** that must be producible and realizable. Fill rows conceptually.

### Dimensions
- **NP Composition**: head_concept + adjectives (list of adj_concepts + per-adj features.degree, features.number etc.)
- **Coordination**: on which role(s), arity (2/3+), adjective distribution (shared / per-conjunct / none)
- **Frame**: type + filled roles with NPs above
- **Sentence-level**: polarity, illocution, quantification, temporal, tense, aspect, etc.
- **Features per Entity**: gender (for agreement), number, case (PL), degree (on adjs), animacy, countability, initial_sound (for articles)

### IL Structure Coverage Matrix (Concept Level)

**Adjective Concepts Available** (from concepts.ron): BIG, SMALL, GOOD, BAD, NEW, OLD, RED, BLUE, GREEN, HOT, COLD (11 total).  
**Degree-supporting** (via suppletive_* in pl/lexicon.ron + explicit degree entries in both lexicons): GOOD (lepszy/better, najlepszy/best), BIG (większy/bigger), SMALL (mniejszy), BAD (gorszy). Others use regular RON suffix rules where applicable.  
**Coordination**: First-class via `Coordination { items: Vec<Entity>, conjunction }` attached to any role Entity. Adjective distribution: none (no adjs), shared (adj on coordination container or pre-conjunct), per-conjunct (each item has its own adjectives Vec), mixed.  
**Frames**: Transfer, Motion, Perception, Cognition, Emotion, Communication, Creation, Destruction, Consumption, Possession, Existence, Statement, Custom.  

| ID   | Head Concept(s) + Adjectives (concepts + degree) | Coordination (role, arity, adj distribution) | Frame + Roles | Sentence Features (polarity, illoc, quant, temporal, tense, aspect) | IL Description (key structures) | Status | Notes / Exceptions |
|------|--------------------------------------------------|----------------------------------------------|---------------|---------------------------------------------------------------------|---------------------------------|--------|--------------------|
| IL-001 | APPLE + [] | None | Transfer (GIVE): agent=PERSON, theme=APPLE, recipient=PERSON | Positive, Statement, no quant | Simple NP, no adj, no coord | TODO | Baseline |
| IL-002 | APPLE + [BIG] | None | Transfer | Positive, Statement | Entity with 1 adjective (no degree) | TODO | Stacking 1 |
| IL-003 | APPLE + [BIG, RED] | None | Perception (SEE) | Positive, Statement | Stacked 2 adjectives on theme | TODO | Stacking 2, order matters? |
| IL-004 | APPLE + [GOOD + Comparative] | None | Emotion (LOVE) | Positive, Statement | Degree on adjective | TODO | Comparative via lexicon |
| IL-005 | APPLE + [GOOD + Superlative] | None | Creation (MAKE) | Positive, Statement | Superlative | TODO | "najlepszy" via prefix + RON |
| IL-006 | BOOK + [NEW] | None | Perception (READ) | Positive, Statement | Regular adj (no suppletive) | TODO | Non-degree-supporting adj |
| IL-007 | CAR + [OLD, RED] | None | Possession (HAVE) | Positive, Statement | 2 stacked regular | TODO | Color+age |
| IL-008 | WATER + [COLD, BLUE] | None | Consumption (DRINK) | Positive, Statement | Mass + 2 stacked | TODO | Mass stacking |
| IL-009 | STUDENT + [GOOD, NEW] | None | Cognition (THINK) | Positive, Statement | Quality+age stacked | TODO | |
| IL-010 | PERSON + [] | Theme (2 items: APPLE + BOOK) | Transfer | Positive, Statement | Coordination on theme | TODO | Basic list |
| IL-011 | PERSON + [BIG] | Theme (2: APPLE + BOOK, shared adj) | Transfer | Positive, Statement | Shared adjective before coordination | TODO | "duży jabłko i książka" |
| IL-012 | PERSON + [] | Theme (2: APPLE+[RED] + BOOK+[BLUE]) | Transfer | Positive, Statement | Distributed adjectives in list | TODO | Different adjs per item |
| IL-013 | PERSON + [BIG] / STUDENT + [SMALL] | Subject (2 items) | Transfer | Positive, Statement | Adjectives on coordinated subjects | TODO | Agreement on verb? |
| IL-014 | CAT + [SMALL] / DOG + [BIG] | Agent (2), Theme (APPLE + []) | Consumption (EAT) | Positive, Statement, number=pl | Coord subj + simple theme | TODO | |
| IL-015 | PERSON + [] | Theme (2: APPLE+[BIG,RED] + BOOK+[NEW]) | Creation (MAKE) | Positive, Statement | Stacked adjs + distributed in coord | TODO | |
| IL-016 | STUDENT + [GOOD + Comp] / TEACHER + [BAD] | Agent (2) | Perception (SEE) | Positive, Statement | Comp + positive in subj coord | TODO | Degree mixed in list |
| IL-017 | APPLE + [RED] | Theme (3: APPLE + BOOK + CAR) | Transfer + Quant=Numerical(3) | Positive, Statement | 3-item coord + adj on one | TODO | |
| IL-018 | PERSON + [] | Theme (APPLE + [HOT] / WATER + [COLD]) | Consumption (DRINK) | Positive, Statement | Mixed count/mass in coord | TODO | |
| IL-019 | CITY + [BIG] / HOUSE + [SMALL] | Goal (2) | Motion (GO) | Positive, Statement | Coord goal + adjs | TODO | |
| IL-020 | CAT + [SMALL] / DOG + [BIG] | Subject (2), Theme (APPLE + [RED]) | Consumption (EAT) | Positive, Statement, number=pl | Mixed types + adjs + coord on subj + theme | TODO | Complex NP |
| IL-021 | PERSON + [GOOD + Comp] / PERSON + [BAD + Comp] | Agent (2), Theme (BOOK + [NEW]) | Perception (READ) | Question | Degree on both conjuncts + q | TODO | |
| IL-022 | MILK + [COLD] / BREAD + [HOT] | Theme (2) | Consumption | Negative, Statement | Neg + mass/count coord | TODO | |
| IL-023 | TREE + [GREEN] / WINDOW + [BLUE] | Location (2) | Existence (BE) | Positive, Statement | Locative coord + color adjs | TODO | |
| IL-030 | PERSON + [GOOD + Comp] | Theme (3 items) | Transfer + Quant=Numerical(3) | Question | Degree + coordination + quant | TODO | "Czy lepszy student dał 3 jabłka?" |
| IL-031 | STUDENT + [] | Theme (5: APPLE x5) | Transfer + Quant=Numerical(5) | Positive, Statement | 5+ triggers Gen Pl + case on theme (PL) | TODO | Case interaction |
| IL-032 | PERSON + [BIG] | Theme (Numerical(2) + APPLE+[RED]) | Transfer | Positive | Numeral + adj on theme | TODO | |
| IL-040 | PERSON + [] | None | Possession (HAVE) + Negation | Negative, Question | Simple NP in negated question | TODO | "Czy nie ma jabłka?" |
| IL-041 | CAT + [SMALL] | Theme (none, implied) | Possession (HAVE) + Neg | Negative, Statement | Neg + adj entity | TODO | |
| IL-042 | PERSON + [GOOD + Comp] | Theme (CAT + [BAD]) | Possession + Neg + Question | Negative, Question | Comp + neg + q | TODO | |
| IL-050 | APPLE + [BIG, RED + Comp] | None | Existence (BE) + Temporal (yesterday) | Positive, Statement | Stacked + degree + temporal | TODO | Complex features |
| IL-051 | PERSON + [OLD] | Theme (CAR + [NEW]) | Possession + Temporal (today) | Positive, Statement | Age contrast + temporal | TODO | |
| IL-052 | BOOK + [RED, BLUE] | None | Perception (READ) + Aspect=Perfective | Positive | Stacked color + aspect | TODO | |
| IL-060 | WATER + [COLD] | None | Possession (HAVE) | Positive, Statement, number=sg | Mass noun + adj | TODO | Mass + adj |
| IL-061 | PERSON + [GOOD + Comp] | None | Possession (HAVE) + Quant=Numerical(2) | Positive, Statement | Comp adj + numerical | TODO | "lepszy student ma 2 koty" |
| IL-062 | MILK + [HOT] | None | Consumption (DRINK) + Question | Question | Mass + adj + q , a/an not applicable | TODO | Article decision on mass |
| IL-070 | PERSON + [] | Subject (3 items: PERSON + CAT + DOG) | Motion (GO) + Goal=CITY | Positive, Statement, number=pl | 3-item coordination on subject | TODO | Multi coordination |
| IL-071 | APPLE + [RED] / BOOK + [BLUE] / CAR + [BIG] | Theme (3) | Creation (MAKE) | Positive, Statement | Distributed adjs in 3-item list | TODO | 3-item distributed |
| IL-072 | PERSON + [BIG] / STUDENT + [SMALL] / TEACHER + [] | Agent (3) | Transfer | Positive, Statement, number=pl | 3-item mixed adj distribution on agent | TODO | |
| IL-073 | CAT + [HOT] / DOG + [COLD] | Theme (3: MILK + WATER + BREAD) | Consumption | Positive, Statement | Coord theme mass + subj coord | TODO | |
| IL-080 | PERSON + [SMALL] | Theme (APPLE + [BIG + Comp]) | Transfer | Negative, Question | Stacked degree + neg + q | TODO | "Czy mały Tomek nie dał większego jabłka?" |
| IL-081 | STUDENT + [GOOD + Super] / TEACHER + [BAD] | Subject (2) | Perception (SEE) + Temporal (today) | Positive, Statement | Super on one, mixed coord + temporal | TODO | Superlative in list |
| IL-082 | APPLE + [GOOD + Comp] / BOOK + [BAD + Comp] | Theme (2) | Emotion (LOVE) + Neg | Negative, Statement | Both comp in coord + neg | TODO | |
| IL-083 | PERSON + [] | Agent (PERSON + [BIG]), Theme (3: APPLE+[RED] + BOOK + CAR+[OLD]) | Transfer + Quant | Positive | Agent simple + 3-item theme with mixed adjs + quant | TODO | |
| IL-090 | CAT + [HOT] | None | Consumption (DRINK) + Aspect=Perfective | Positive, Statement | Adj + perfective aspect | TODO | Aspect interaction |
| IL-091 | PERSON + [BAD + Comp] | Theme (FOOD + [OLD]) | Consumption (EAT) + Aspect=Imperfective | Positive | Comp + aspect | TODO | |
| IL-100 | HOUSE + [BIG] | None | Destruction (BREAK) | Positive, Statement | Destruction frame + adj | TODO | |
| IL-101 | PERSON + [BAD + Comp] | Theme (MILK + [COLD]) | Consumption (DRINK) + Question | Positive, Question | Comp adj + mass + q | TODO | |
| IL-102 | STUDENT + [GOOD] / TEACHER + [BAD] | Theme (BOOK + [BLUE + Super]) | Perception (READ) | Positive, Statement, number=pl | Super in coord + pl | TODO | |
| IL-103 | APPLE + [HOT] | None | Existence (BE) + Quant=Universal | Positive, Statement | Universal + adj | TODO | "Wszyscy widzą gorące jabłko" |
| IL-104 | CAT + [SMALL + Comp] | Subject (2: CAT + DOG) | Emotion (HATE) + Neg | Negative, Statement | Neg + comp + coord | TODO | |
| IL-105 | PERSON + [] | Theme (CAR + [RED]) | Motion (GO) + Temporal (tomorrow) | Positive, Statement | Temporal + adj | TODO | |
| IL-106 | PERSON + [NEW] | Message ( "something" ) | Communication (SAY) | Positive, Statement | Communication frame + adj agent | TODO | |
| IL-107 | PERSON + [GOOD + Super] | Content (BOOK + [OLD]) | Cognition (KNOW) | Positive, Statement | Cognition + super | TODO | |
| IL-108 | PERSON + [] / PERSON + [SMALL] | Agent (2) | Creation (MAKE) + Instrument=CAR | Positive | Coord agent + instrument | TODO | |
| IL-109 | WINDOW + [BLUE] | Patient (none explicit) | Destruction (BREAK) + Question | Question | Question on destruction | TODO | |
| IL-110 | PERSON + [OLD + Comp] | Stimulus (CAT + [SMALL]) | Emotion (LOVE) | Positive, Statement | Comp on experiencer + adj stimulus | TODO | |
| IL-111 | CITY + [BIG] | Goal | Motion (COME) + Source=CITY + [SMALL] | Positive, Statement | Motion source/goal both with adj | TODO | |
| IL-112 | BREAD + [HOT] / MILK + [COLD] | Theme (2) | Possession (HAVE) + Neg | Negative, Statement | Neg + coord mass + adjs | TODO | |
| IL-113 | STUDENT + [BAD] | Theme (5 APPLE) | Transfer + Quant=Numerical(5) | Positive, Statement | Numeral 5 + adj + gen case | TODO | |
| IL-114 | PERSON + [GOOD] | Theme (APPLE + [RED, BIG]) | Transfer + Aspect | Positive | Stacked 2 + aspect | TODO | |
| IL-115 | CAT + [] / DOG + [BIG] / PERSON + [SMALL] | Agent (3) | Perception (SEE) + Temporal (yesterday) | Positive, Statement, number=pl | 3-item agent coord + temporal + mixed adj | TODO | |
| IL-116 | PERSON + [HOT + Comp] | Theme (WATER + [COLD]) | Consumption (DRINK) + Question | Question | Comp + mass + q | TODO | |
| IL-117 | BOOK + [NEW] / CAR + [OLD] | Theme (2) | Transfer | Positive, Statement | Age contrast coord | TODO | |
| IL-118 | PERSON + [] | Theme (APPLE + [GOOD + Super]) | Emotion (LOVE) | Positive, Statement | Superlative on theme | TODO | |
| IL-119 | PERSON + [BAD + Super] | None | Statement (BE) | Positive, Statement | Super on property in Statement frame | TODO | |
| IL-120 | APPLE + [RED] / APPLE + [GREEN] / APPLE + [BLUE] | Theme (3) | Creation (MAKE) + Quant=Many | Positive | 3 identical heads + different adjs + quant | TODO | |

**How to fill**:
- Use only concepts from `data/concepts/concepts.ron`
- Adjectives only from existing adjective concepts (BIG, GOOD, RED...)
- Degree only when lexicon supports (via suppletive or explicit entry)
- Describe the exact `Entity` tree and `Frame` you expect in IL.

**Add more rows for**:
- All Frame variants (already expanding)
- All role combinations (agent/theme/recipient/stimulus etc.)
- Mass vs Count nouns with quantifiers
- Proper names vs common nouns (Tom, Iza as PERSON with name)
- Different cases in PL (Genitive after numbers >=5, Acc/Gen under negation, etc.)
- Voice (Passive supported in some tests)
- Modality, other sentence features

**Target expansion to 500**: Treat each row as a *class*. A future concept-driven generator (not current surface script) enumerates:
- All single-adj combos (11 heads x 11 adjs x 3 degrees x ~12 frames x 4 polar/illoc)
- Stacking permutations of 2 adjs (P(11,2))
- Coord arities 2/3 for each role pair
- Cross with 8-10 sentence feature flags (sampled, not full 2^N)
This yields combinatorial hundreds without any PL/EN surface literals in the matrix or generator logic. Only data/concepts + features drive construction of Entity/Coordination/Frame.

**How to fill**:
- Use only concepts from `data/concepts/concepts.ron`
- Adjectives only from existing adjective concepts (BIG, GOOD, RED...)
- Degree only when lexicon supports (via suppletive or explicit entry)
- Describe the exact `Entity` tree and `Frame` you expect in IL.

**Add more rows for**:
- All Frame variants (Motion, Creation, Destruction, Communication, Cognition...)
- All role combinations
- Mass vs Count nouns with quantifiers
- Proper names vs common nouns
- Different cases in PL (Genitive after numbers >=5, etc.)
- Voice (if supported)
- Modality

---

## 2. Bidirectional Coverage Matrices (PL ↔ IL ↔ EN)

### 2.1 Parsing Direction (Surface → IL)

For each conceptual IL structure above, we need test cases that the **PL parser** and **EN parser** turn into the same (or equivalent) IL. Parser must recover: head concept, adjectives as separate Entity vec (no name concat), coordination items, degree from explicit entries or suppletive stems via lexicon, case/number/gender from morphology/lexicon, initial_sound for articles.

**PL → IL Parsing Matrix** (expanded; surface described conceptually only for illustration, actual tests use real lexicon words)

| ID      | PL Surface (conceptual description) | Expected IL (reference to IL-xxx or describe) | Status | Parser Notes |
|---------|-------------------------------------|-----------------------------------------------|--------|--------------|
| P-001  | PERSON dał APPLE PERSON (no adjs)            | IL-001 (Transfer, simple NPs)                | TODO   | Basic roles  |
| P-002  | PERSON dał APPLE+[BIG, RED] PERSON             | IL-003 (stacked adjs on theme)             | TODO   | Stacking + agreement |
| P-003  | Czy PERSON+[GOOD + Comp] dał APPLE?   | IL-004 + Question + Comp degree              | TODO   | Degree recovery via lexicon entry for comparative surface |
| P-004  | Tomek dał duży czerwony jabłko Izie | IL-003 variant + proper name PERSON | TODO | Stacking 2 regular-ish + names |
| P-005  | Lepszy student ma kota | IL-061 + Possession | TODO | Comp degree recovery (lepszy) |
| P-006  | Najlepszy student widział książkę | IL-005 variant + Perception | TODO | Superlative "naj-" recovery |
| P-007  | Zimny mleko i gorący chleb | IL-018 variant (coord mass+count + adjs) | TODO | Mixed mass/count coord parse |
| P-010  | PERSON i PERSON dali APPLE          | IL-010 (Coordination on agent)               | TODO   | List + number=pl |
| P-011  | CAT+[SMALL] i DOG+[BIG] pili WATER   | IL-012 (distributed adjs in list on subj)    | TODO   | Adjs + coordination + agreement |
| P-012  | Duży kot i mały pies widzieli książkę | IL-013/020 + Perception | TODO | Distributed adjs + verb pl |
| P-013  | 3 dobre studentki i 2 źli profesorowie | IL with Numerical + adj per group | TODO | Quant + adj + gender/pl |
| P-020  | PERSON nie widział CAT+[BIG] wczoraj | IL with Neg + Temporal + Adj | TODO   | Full feature bundle |
| P-021  | Czy PERSON+[GOOD + Comp] i PERSON+[BAD] widzieli APPLE+[RED]? | IL with Question + Coord on subj + distributed adj on theme | TODO   | Complex parsing with coord + adj |
| P-022  | 5 STUDENT+[SMALL] dało APPLE | IL with Numerical(5) + Gen case + adj | TODO   | Case effect after number + adj |
| P-023  | Nie dał czerwonego jabłka | IL-040 + Neg + theme adj + gen case | TODO | Negation + adj + case |
| P-024  | Czy Tomek i Iza dali duże jabłko? | IL-010 + Question + adj on theme | TODO | Coord + q + adj |
| P-030  | ... (add for all frames, mass nouns, proper names, questions with lists, negation with degree) | ... | TODO | |
| P-031 | Duży dobry nowy czerwony dom | IL-007 stacked 4? (limit test 3+) | TODO | Max stacking stress |
| P-032 | Kot i pies i dziecko zjadły chleb | IL-070 3-item agent coord | TODO | 3-item parse |
| P-033 | Lepsze jabłko i gorsza książka | IL-016 variant (comp per conjunct) | TODO | Degree distributed |

**Add rows for**:
- All adjective stacking combos (0-3 from 11)
- All coordination distributions (shared/per/mixed) x roles x arity(2/3)
- Suppletive vs regular degree (lepszy/better vs czerwony -> no regular comp in data for all)
- Case effects ("5 jabłek", "dużego kota", gen under neg)
- Questions with complex NPs (coord inside q, adj inside q)
- Negation affecting case (Genitive in PL)
- Mass nouns (woda, mleko) + adj + article decision (EN)
- All frames with at least 1 adj and 1 coord case

### 2.2 Generation Direction (IL → Surface)

For each IL structure, what PL and EN surfaces the generators must produce. **Must be 100% algorithmic**: walk adjectives first (realize_noun_phrase on each adj Entity with propagated features), then head; use Coordination items; lookup lexicon for degree stems/suppletives + RON for suffix/prefix; use initial_sound for a/an; case from sentence polarity/numeral; number/pl agreement on verbs via coord detection.

**IL → PL / EN Generation Matrix** (expanded)

| ID      | IL Structure (concept level)              | Expected PL Surface (algorithmic) | Expected EN Surface (algorithmic) | Status | Notes (articles, agreement, degree realization) |
|---------|-------------------------------------------|-----------------------------------|-----------------------------------|--------|-------------------------------------------------|
| G-001  | Transfer, agent=PERSON, theme=APPLE+[BIG] | PERSON dał APPLE+[BIG] PERSON     | PERSON gave APPLE+[BIG] PERSON    | TODO   | EN article from initial_sound or spelling fallback |
| G-002  | Transfer, theme=APPLE+[GOOD + Comp]      | PERSON dał APPLE+[GOOD+Comp]     | PERSON gave APPLE+[GOOD+Comp]        | TODO   | Comparative from lexicon (suppletive or explicit entry) |
| G-003  | Perception, stimulus=APPLE+[BIG, RED] | ... widział APPLE+[BIG,RED] | ... saw a big red APPLE | TODO | 2-adj stacking + article |
| G-004  | Possession, possessed=APPLE+[GOOD + Super] | ... ma najlepsze jabłko | ... has the best apple | TODO | Super via naj- + RON or EN best |
| G-010  | Coordination on theme (APPLE + BOOK), shared [RED] | PERSON dał APPLE+[RED] + BOOK+[RED] | PERSON gave (APPLE + BOOK)+[RED] | TODO   | Coordination realization + shared adj |
| G-011  | Coordination on subject (PERSON+[BIG] + STUDENT+[SMALL]), theme=APPLE | (PERSON+[BIG] + STUDENT+[SMALL]) dał APPLE | (PERSON+[BIG] + STUDENT+[SMALL]) gave APPLE | TODO | Adjs per conjunct, number=pl on verb |
| G-012  | Agent coord 2 with per-adj + theme 3-item mixed | (PERSON+[BIG] i STUDENT+[SMALL]) dał APPLE+[RED] i BOOK i CAR+[OLD] | ... | TODO | Full mixed coord + adjs |
| G-013  | 3-item agent coord | (PERSON i CAT i DOG) ... | (Tom and a cat and a dog) ... | TODO | 3-item "i"/"and" + verb pl |
| G-020  | Emotion, experiencer=PERSON+[GOOD+Comp], stimulus=CAT+[BAD] | PERSON+[GOOD+Comp] kochał CAT+[BAD] | PERSON+[GOOD+Comp] loved CAT+[BAD] | TODO | Degree on both sides |
| G-021  | Transfer, agent=PERSON, theme=Coord(APPLE+[RED], BOOK+[BLUE]) | PERSON dał APPLE+[RED] i BOOK+[BLUE] | PERSON gave red APPLE and blue BOOK | TODO | Distributed in coord |
| G-022  | Transfer + Quant=Numerical(5), theme=APPLE | PERSON dał 5 APPLE (gen pl) | PERSON gave 5 APPLEs | TODO | Number/case effect |
| G-023  | Consumption mass + adj + q | Czy ... pił zimne mleko? | Did ... drink cold milk? | TODO | Mass no article, adj agreement |
| G-030  | Transfer + neg + adj on theme | PERSON nie dał czerwonego jabłka | PERSON did not give a red apple | TODO | Neg + case gen + adj |
| G-031  | Motion + goal adj + source adj | Poszedł z małego domu do dużego miasta | Went from the small house to the big city | TODO | Preps + adjs |
| G-032  | Statement frame (BE) + super property | Tomek jest najlepszym studentem | Tom is the best student | TODO | Statement + degree |
| G-033  | Coord + degree mixed + temporal | Lepszy kot i gorszy pies zjadły wczoraj | A better cat and a worse dog ate yesterday | TODO | Degree + coord + temp |
| G-040  | ... (add for superlatives, mass, questions, neg, all frames) | ... | ... | TODO | |

**Add rows for**:
- Superlatives ("najlepszy" via Prefix + RON)
- Mass nouns + articles (EN "milk" vs "an apple" — "a" only for count vowel/consonant initial_sound)
- Proper names (no articles, casing, "Tomek" not "a Tomek")
- Different cases in PL output (Nom/Acc/Gen/Dat)
- Periphrastic vs morphological degree in EN (current data uses direct "better"; extend only if RON/lex adds "more")
- Questions: "Czy ... ?" vs "Did ... ?"
- Negation placement and forms ("nie" particle, do-support)
- All 11 adjs x degrees where supported, in all frames
- Full roundtrip fidelity (PL input -> IL -> EN output semantically matches reverse)

---

## 3. Full Coverage Dimensions (Exhaustive Checklist)

Use this to expand the matrices above until "full possible coverage". Each checkbox should be hit by at least one IL- / P- / G- row + executable test (parser or generator roundtrip).

### 3.1 NP + Adjectives (11 adj concepts)
- [x] 0 adjectives (baseline) — IL-001
- [x] 1 adjective (all categories: size=BIG/SMALL, quality=GOOD/BAD/NEW/OLD, color=RED/BLUE/GREEN, temp=HOT/COLD) — IL-002,006,008
- [x] 2 stacked adjectives (combinations of categories) — IL-003,007,050
- [ ] 3+ stacked (at least one example) — IL-031 stress
- [x] Degree on adjective: Positive (default), Comp, Super (regular + suppletive) — IL-004,005,016,081
- [ ] Degree on multiple adjectives in one NP
- [x] Adjectives on different roles (agent, theme, recipient, stimulus, goal...) — many
- [x] Adjectives + inherent features (count/mass, animate) — IL-008,060,062

### 3.2 Enumerations (Coordination)
- [x] 2-item coordination (subject, theme, both) — IL-010..019
- [x] 3-item coordination — IL-070,071,072,115
- [x] Shared adjectives (before conjunction) — IL-011
- [x] Per-conjunct adjectives — IL-012,013,016
- [x] Mixed: some conjuncts have adjs, some don't — IL-017,072
- [x] Coordination + degree on adjs — IL-016,021,080,082
- [x] Coordination + number effects (plural verb) — IL-013,020
- [ ] Coordination + quantification ("wielu dobrych studentów i nauczycieli") — partial IL-030,113

### 3.3 Feature Interactions
- [x] Adjectives + Sg/Pl (agreement in PL)
- [x] Adjectives + case (Gen, Acc, etc. in PL) — IL-022, P-022
- [x] Adjectives + Numerical quant (>=5 → Gen Pl in PL) — IL-031,113
- [x] Adjectives + negation (case effects in PL?) — IL-040,080,104
- [x] Adjectives + questions (do-support in EN) — IL-030,080,101
- [x] Adjectives + temporal — IL-050,081,105
- [x] Adjectives + aspect (perfective/imperfective) — IL-090,091
- [x] Adjectives + possession ("ma dużego kota") — IL-061

### 3.4 Frames & Roles (all combinations with above)
- [x] Transfer (GIVE, BUY, SELL, TAKE) — many IL-00x
- [x] Perception (SEE, READ, HEAR) — IL-003,102
- [x] Consumption (EAT, DRINK) — IL-018,060,090
- [x] Emotion (LOVE, HATE) — IL-004,104,110
- [x] Creation (MAKE, WRITE) — IL-005,071
- [x] Motion (GO, COME) — IL-070,105,111
- [x] Possession (HAVE) — IL-040,060,061
- [x] Existence (BE) — IL-050,103
- [x] Destruction (BREAK) — IL-100,109
- [x] Communication (SAY, ASK) — IL-106
- [x] Cognition (THINK, KNOW) — IL-009,107
- [ ] Statement (BE property) + full adjs/coord — IL-119 partial

### 3.5 Lexical Exceptions (only these may be non-algorithmic)
- [x] Suppletive comparatives/superlatives (lepszy, większy, gorszy, najlepszy...) — via lexicon `suppletive_*` or explicit entries — data has for GOOD/BIG/SMALL/BAD; all other realization algorithmic via RON + FeatureBundle.degree
- [x] Irregular verb forms for specific concepts (past "gave"/"dał" etc) — lexicon + paradigm
- [ ] "być"/"be" suppletion and closed class (if not yet data-driven)

**Everything else must be computed** from:
- Concept
- FeatureBundle (degree, number, gender, case, initial_sound, suppletive_*, ...)
- RON morphology rules (ReplaceSuffix, Prefix for "naj-", conditions DegreeIs etc.)
- Lexicon features (initial_sound for a/an, gender, countability, explicit degree forms)

### 3.6 Roundtrip & Full Pipeline Coverage
- [ ] PL surface (adj+coord+degree+frame) → IL → EN surface (sem fidelity)
- [ ] EN surface → IL → PL surface
- [ ] IL direct → PL + EN (generator only)
- [ ] Error cases (unknown concept, degree without lexicon support, etc.)
- [ ] Max complexity: 3-item coord + 2 stacked adjs on 2 conjuncts + degree on one + neg + q + numeral + temporal + specific frame

---

## 4. Algorithmic Expansion Plan for ~500 Entries (Concept-Driven Only)

No surface lists in generators/parsers/tests (except true lexical data).

1. **Seed from this matrix**: 70+ IL rows above already cover dimensions.
2. **Combinatorial generator** (new script or Rust bin): 
   - Load concepts.ron + pl/en lexicons.
   - For heads in entity concepts (20+).
   - For k=0..3 adjs: combinations/permutations of 11 adj concepts; attach degree Positive/Comp/Super only if lexicon has support for that (check suppletive or degree entry).
   - For each role in frame: optionally wrap Entity in Coordination(arity=2 or 3, distribution=shared|per|mixed).
   - Fill 1-2 sentence features from set (Polarity, Illocution=Question|Statement, Quant=Numerical(1..6|Universal), Temporal, Aspect, Tense).
   - Build pure IL Sentence { frame: Frame {.. with Entities having .adjectives and .coordination }, polarity, illocution, ... }
   - Run through generator for PL+EN; optionally synthetic surface for parser tests by minimal realization or hand-curated minimal set.
3. **Verification harness**: assert IL structures exact (by concept ids + features.degree + len(adjectives)+coord), then surfaces contain expected lexical lemmas (not hard strings) + correct article/degree forms.
4. **Current benchmark script** (`scripts/generate_benchmark.sh`) is **surface-based** (PL_ADJS/EN_ADJS arrays + string emit). It produces volume but violates "no surface hacks". Use it only for smoke; replace with concept enumerator for the 500 target.
5. **Target counts** (example):
   - Base NPs + 0/1/2/3 adjs: ~ 20 heads * (1 + 11 + 11*10 + ...) ~150
   - + Degree variants (4 suppletive x 3 deg): +80
   - + Coord 2/3 item x 4 roles x 3 dist: +120
   - + Frames 12 x feature combos sampled 4-5: +150
   - Total easily >500 unique conceptual cases.

Update this doc + TEST_STRATEGY.md when generator is concept-ified.

**Current status of matrices (2026-07-10 iteration)**: IL matrix ~70 rows (was ~25). P/G expanded with 15+ each. Checklists 70%+ conceptually covered by rows. Still need executable tests for most new rows + full roundtrips.

### 3.3 Feature Interactions
- [ ] Adjectives + Sg/Pl (agreement in PL)
- [ ] Adjectives + case (Gen, Acc, etc. in PL)
- [ ] Adjectives + Numerical quant (>=5 → Gen Pl in PL)
- [ ] Adjectives + negation (case effects in PL?)
- [ ] Adjectives + questions (do-support in EN)
- [ ] Adjectives + temporal
- [ ] Adjectives + aspect (perfective/imperfective)
- [ ] Adjectives + possession ("ma dużego kota")

### 3.4 Frames & Roles (all combinations with above)
- Transfer (GIVE, BUY, SELL, TAKE)
- Perception (SEE, READ, HEAR)
- Consumption (EAT, DRINK)
- Emotion (LOVE, HATE)
- Creation (MAKE, WRITE)
- Motion (GO, COME)
- Possession (HAVE)
- Existence (BE)
- Destruction, Communication, Cognition (at least basic)

### 3.5 Lexical Exceptions (only these may be non-algorithmic)
- Suppletive comparatives/superlatives (lepszy, większy, gorszy, najlepszy...) — via lexicon `suppletive_*` or explicit entries
- Irregular verb forms for specific concepts (if any)
- "być" suppletion and closed class (if not yet data-driven)

**Everything else must be computed** from:
- Concept
- FeatureBundle (degree, number, gender, case, ...)
- RON morphology rules (ReplaceSuffix, Prefix for "naj-", conditions DegreeIs etc.)
- Lexicon features (initial_sound for a/an, gender, countability)

---

## 4. How to Use This Matrix

1. Pick a row (IL-xxx, P-xxx, G-xxx).
2. Describe the **conceptual input** (concepts + features).
3. Build the **IL structure** (Entity trees with adjectives and/or Coordination, Frame with roles).
4. Run through:
   - Parser (for P- rows): feed PL/EN surface → assert correct IL
   - Generator (for G- rows): feed IL → assert correct PL and/or EN surface
5. For full roundtrips: PL surface → IL → EN surface (and reverse), check semantic fidelity.
6. Mark Status: TODO / Implemented / Verified / Edge (only for real exceptions)

**Recommended tooling**:
- Small helper in tests or `src/bin/` that takes conceptual description and builds `Entity` + `Frame` + `Sentence`.
- Use `api.translate` or direct `generate_sentence` + parser for verification.
- Add rows until all dimensions in section 3 are hit at least once (target 200-500+ rows).

---

## 5. Current Gaps & Priorities (as of 2026-07-10, post-iteration)

**Matrix expansion this round**:
- IL rows: ~70 detailed concept rows (from ~25). Covers 0-3 adjs, all 11 adj concepts sampled, all major frames, 2/3-arity coord, all distributions (shared/per/mixed), degree (comp/super), interactions with neg/q/quant/temporal/aspect/numeral+case.
- P- rows: +12 new (stacking, mass coord, 3-item, case, neg+q).
- G- rows: +11 new (super, mass, 3-item mixed, neg+case, preps).
- Checklists: marked many [x] based on matrix coverage; added Roundtrip section + explicit expansion plan to 500.
- Bidirectional emphasis: every IL row should have corresponding P- (PL+EN parse to it) and G- (IL realize to PL+EN).

**Remaining implementation gaps** (from CLI + tests + rg):
- Full adj agreement in PL output for stacked (e.g. "duże czerwone jabłko" gender/case).
- Verb agreement + correct past forms under coordination (current: "Tomek see and Iza ...", "widzieli" mangled in some outputs).
- 3-item coordination parsing/generation stability.
- Degree + coord combinations in generator (some surface leakage or missing feature propagation).
- Complete roundtrips for complex (adj+coord+degree).
- Case on adjs under numerals/neg (partial).
- Mass noun articles (EN "milk" vs count "an apple" via initial_sound only on count).
- The `scripts/generate_benchmark.sh` remains surface-list based (PL_ADJS etc) — must not be used as source of truth for 500; matrix + future concept enumerator is the way.

**Verification criteria (must all pass before claim "done")**:
1. cargo test: 0 failures.
2. rg (bad surface patterns in src/*.rs): exactly 0.
3. Key CLIs for degree (lepszy/better), stacking (big red), coord (X i/and Y with adjs) produce expected lemmas + structures (no "name-concat", no hard "lepszy" in logic).
4. Matrix in this file describes full dimensions + explicit plan to algorithmic 500+.
5. No PL/EN surface literals in generator/parser code paths (data/ only).

**Related files**:
- `data/concepts/concepts.ron` (11 adjs, 20+ entities, 12+ frames)
- `data/lexicons/*/lexicon.ron` (adjective entries with suppletives + degree forms)
- `data/morphology/pl/adj_paradigms.ron` (DegreeIs + Prefix naj-)
- `src/core/interlingua.rs` (Entity {adjectives, coordination}, Frame enum, FeatureBundle)
- `src/engines/{pl,en}/{parser,generator}.rs` + `generation/realizer.rs`
- `scripts/generate_benchmark.sh` (current volume generator — surface)
- `TEST_MATRIX_ADJECTIVES_LISTS.md`, `BENCHMARK_ANALYSIS.md`

Iterate: add rows → implement missing in engines → run verification plan (cargo + rg + targeted cli + bench) → update this doc + scratch/*.txt → repeat. "dobrze to iteruj dawaj pelny raport" fulfilled by this expanded version + report files.

---

# 6. Sentence-Driven Iteration Process (New Focus)

**Core rule going forward**: Every iteration starts from **real sentences** (collected from CLI experiments, benchmark failures, user examples, integration gaps). 

**Cycle**:
1. Collect 5–15 representative sentences exercising new or broken grammar.
2. Run through current system (translate + parse to IL) → record outputs + IL structures.
3. Analyze: which IL features were built vs. realized? Which grammar constructs missing or buggy?
4. Expand this matrix with new rows (IL / P / G) covering the missing dimensions.
5. Identify concrete feature(s) to add or fix in engines / morphology / deduction (e.g. "full verb agreement under Coordination", "preposition → oblique role mapping", "Voice=Passive propagation").
6. Implement (algorithmic, lexicon + RON + FeatureBundle driven).
7. Verify with the driving sentences + add strict tests.
8. Update checklists, gaps, report in `scratch/`.
9. Repeat.

This ensures we grow toward a **pełnowartościowy silnik** (full-fledged engine) rather than only adjectives + lists.

**Current driving sentences** (from live experiments 2026-07-10, see `scratch/sentences_for_iteration.txt`):
- "Tomek i Iza widzieli dużego czerwonego kota i małego psa" → mangled coord + stacked on theme
- "Lepszy student i gorszy profesor czytali książkę" → degree + coordination broken
- "Tomek poszedł do miasta z psem" → preposition "z" + Instrument/Comitative lost
- "Tomek szybko zjadł jabłko" → adverb lost / mangled
- "5 studentów dało jabłka" → numeral + genitive case + agreement
- "Książka została przeczytana przez studenta" → passive voice attempt
- "Tomek dał jabłko i książkę Izie i Tomkowi" → double coordination (theme + recipients)
- "Czy Tomek dał duże jabłko Izie wczoraj?" → question + stacked adj + temporal (adj form broken)

Use these + new ones every iteration.

---

# 7. Full Linguistic Feature Coverage Matrices (Beyond Adjectives & Simple Lists)

The original focus (adjectives + enumerations) is now a **subset**. For a full engine we must cover core grammar of Polish and English bidirectionally via rich IL.

## 7.1 Verb Features Matrix (Tense, Aspect, Voice, Mood, Modality, Illocution, Polarity)

IL already models these on `Sentence` + `FeatureBundle` + `Frame.verb_concept`.

| ID   | Sentence Features | Example PL | Expected IL | Expected EN | Status | Engine Gaps |
|------|-------------------|------------|-------------|-------------|--------|-------------|
| V-001 | Past + Perfective | Tomek dał jabłko | tense=Past, aspect=Perfective | gave | Partial | OK for basic |
| V-002 | Past + Imperfective | Tomek dawał jabłko | aspect=Imperfective | was giving / gave (context) | TODO | Aspect rarely distinguished in EN output |
| V-003 | Present | Tomek daje jabłko | tense=Present | gives | Partial | Present 3sg often missing |
| V-004 | Future (bedzie + inf or perfective) | Tomek da jabłko | tense=Future | will give | TODO | Future not realized |
| V-005 | Passive Voice | Książka została przeczytana | voice=Passive, theme promoted | The book was read | Partial | EN has skeleton, PL almost none; role swap weak |
| V-006 | Question (yes/no) + any verb features | Czy Tomek dał...? | illocution=Question | Did ... ? | Good | Basic works |
| V-007 | Negation + Past | Tomek nie dał | polarity=Negative | did not give | Good | Case in PL (gen) partial |
| V-008 | Modality (possibility) | Tomek może dać | modality=Possibility | Tom can/may give | TODO | Almost no modality support |
| V-009 | Imperative | Daj jabłko! | illocution=Imperative / mood=Imperative | Give the apple! | TODO | Not supported |
| V-010 | Degree on verb? (rare) or periphrastic | (via adverbs) | — | — | TODO | Out of scope for now |
| V-011 | Agreement via engine for coord | (Tomek i Iza widzieli) | number=Plural from AgreementEngine | saw | Partial (engines wired) |
| C-011 | Case gen after numeral via features | 5 studentów | quantification + case=Gen | 5 students | Partial |

**Required implementation work**:
- Propagate `voice`, `mood`, `modality` fully in deduction + both generators.
- Better aspect → progressive/perfect in EN, aspect pairs in PL.
- Passive: proper "być/zostać + participle" in PL + agent "przez".
- Verb agreement must consider coordination on subject (number, gender in PL past).

## 7.2 Case, Agreement & NP Completeness

Polish has rich case. IL has full `Case` enum. Realization and parsing must be complete.

| ID   | Phenomenon | Example | Key IL Structures | Status |
|------|------------|---------|-------------------|--------|
| C-001 | Nominative (subject) | Tomek dał | agent.features.case = Nominative | Good |
| C-002 | Accusative (direct object) | dał jabłko | theme case=Acc | Good basic |
| C-003 | Dative (recipient) | dał Izie | recipient case=Dative | Partial (sometimes wrong form) |
| C-004 | Genitive after negation | nie dał jabłka | polarity=Neg → theme case=Gen | Partial |
| C-005 | Genitive after numeral >=5 | 5 studentów dało | quantification=Numerical(5), case=Gen Pl | Weak (often fails) |
| C-006 | Adjective case/gender/number agreement | dużego kota | adjectives[*].features carry case/gender | Weak (many leaks) |
| C-007 | Verb past gender agreement | Tomek dał / Iza dała | verb form depends on subject gender | Partial (breaks with coord) |
| C-008 | Instrument / Comitative | poszedł z psem | role=Instrument, preposition "z" | Mostly lost |
| C-009 | Locative / Prepositional | w domu, o książce | various oblique cases + preps | Poor |
| C-010 | Full NP with stacked adjs + case | dużego czerwonego domu | adjectives + head all agree in case | TODO |

**Driving requirement**: Every generated PL form must have correct case on nouns + all adjectives. Parser must recover case from morphology/lexicon.

## 7.3 Prepositions, Adverbs & Oblique Modification

| ID    | Construct | PL Example | IL Representation | Status |
|-------|-----------|------------|-------------------|--------|
| PP-001 | Goal / Direction | poszedł do miasta | goal: Entity with prep "do" | Partial (often drops) |
| PP-002 | Source | wyszedł z domu | source | Weak |
| PP-003 | Instrument | zjadł nożem / poszedł z psem | instrument / accompaniment | Almost never realized |
| PP-004 | Location | był w domu | location | Partial |
| ADV-001 | Manner adverb | szybko zjadł | temporal or separate modifier? (need adverb Entity?) | Broken ("szybko" lost) |
| ADV-002 | Temporal adverb | wczoraj, dzisiaj | sentence.temporal | Works for some |
| ADV-003 | Degree adverb on adj | bardzo duży | degree on adj features or separate | Not modeled |

**Gap**: Currently prepositions are mostly treated as surface tokens. Need systematic mapping: preposition + case → SemanticRole or Feature on Entity. Adverbs need first-class representation (perhaps as separate "Modifier" or in a richer Frame).

## 7.4 Quantification, Numerals & Mass/Count Interactions

(Expand existing)

- Numerical + case (Gen Pl)
- Universal / Existential / NegatedExistential + negation scope
- Mass vs count with quantifiers ("dużo wody", "kilka jabłek")
- Proportional ("wielu", "many")

## 7.5 Advanced Coordination & Conjunctions

Beyond basic "i"/"and":

| ID   | Type | Example | IL | Status |
|------|------|---------|----|--------|
| CO-001 | Simple 2-item | Tomek i Iza | Coordination | Good basic |
| CO-002 | 3+ item | Tomek, Iza i Kot | items.len()=3 | Weak |
| CO-003 | Mixed roles | dał jabłko i książkę Izie i Tomkowi | multiple coordinated roles | Broken |
| CO-004 | "lub" / "or" | jabłko lub książka | conjunction="lub" | TODO |
| CO-005 | "ale" / "but" | Tomek dał, ale Iza nie wzięła | more complex sentence linking | TODO |
| CO-006 | Nested coord | (duży kot i mały pies) i Tomek | nested Coordination | TODO |
| CO-007 | Adjs distributed + shared | duży czerwony kot i mały pies | per-item + shared logic | Partial |

## 7.6 Sentence Types & Illocution

- Declarative (default)
- Yes/no questions ("Czy ...?")
- Content questions (if "kto", "co", "gdzie" supported — currently weak)
- Negation scope
- Imperatives
- Exclamatives (future)

---

# 8. Toward a Full-Fledged Engine — Priority Feature Roadmap (Iterations)

To be called a **pełnowartościowy silnik**, we need reliable bidirectional handling of core Polish/English grammar via the IL protocol.

**Proposed iteration batches** (each iteration = sentences → matrix rows → implementation → verification):

**Iteration A (current/next)**: Coordination robustness + Verb agreement + basic prepositions
- Driving sentences: coord with adjs on multiple roles, "poszedł z psem", "5 studentów"
- Deliver: correct plural verb forms under "X i Y", gender agreement in past when possible, at least Instrument/Goal preps mapped to roles + realized with correct case.
- Matrix: expand V-00x, C-00x, PP-00x, CO-00x with concrete rows.

**Iteration B**: Full case & adj agreement + numerals
- All cases on NPs and their adjectives.
- Proper Gen after 5+ and negation.
- Numeral handling in both directions.

**Iteration C**: Voice (Passive) + more frames + modality basics
- Make Passive work PL↔EN.
- Ensure all Frames from concepts.ron are exercised and realized (Cognition, Emotion, Communication etc.).

**Iteration D**: Adverbs, manner, degree adverbs, richer modification.

**Iteration E**: Questions variants, negation scope, conditionals (if we model them).

**General rules for all iterations**:
- Everything data-driven (lexicon entries + RON paradigms + FeatureBundle).
- No new surface hacks.
- Add rows to this matrix first.
- Add driving sentences + assertions to integration_test.rs or new sentence test file.
- Always verify with `cargo test`, rg=0, and the sentence list.
- Document progress in this file + scratch/full_report_*.txt.

---

## 9. Updated Verification & Reporting

Every iteration must produce:
- Updated `docs/TEST_COVERAGE_MATRIX.md` (new rows + status)
- `scratch/iteration_N_sentences.txt` + outputs
- `scratch/iteration_N_report.txt`
- At least 2–4 new strict tests that would have failed before the iteration
- Confirmation that basic previous champion sentences ("Tomek i Iza dał duży czerwony jabłko Izie", "Czy lepszy student ma kota?") still work perfectly.

**Goal**: After several iterations the system can be credibly called a real interlingua engine for core grammar of two languages, not just a demo for adjectives and lists.

---

**Next concrete step for this session**: 
1. Expand matrix with concrete rows based on the collected driving sentences above.
2. Pick 1–2 small features to implement or harden (e.g. improve coordination verb agreement or add basic preposition role mapping).
3. Re-run the sentence list and produce the report.

Continue the loop.

