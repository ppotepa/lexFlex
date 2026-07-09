# Quantification — Quantifiers, Scope, and Logical Form

Quantification allows lexFlex to represent statements about quantities and distributions: "all students", "some books", "no cats", "three people". This document covers quantifier types, scope resolution, and how quantification maps across languages.

---

## Quantifier Types

```rust
pub enum Quantifier {
    /// Universal: "all", "every", "each", "wszyscy", "każdy"
    /// ∀x P(x) — for all x, P(x) holds
    Universal,
    
    /// Existential: "some", "a", "at least one", "niektórzy", "jakiś"
    /// ∃x P(x) — there exists x such that P(x)
    Existential,
    
    /// Negated existential: "none", "no", "neither", "żaden", "nikt"
    /// ¬∃x P(x) — there does not exist x such that P(x)
    NegatedExistential,
    
    /// Unique existential: "exactly one", "the", "dokładnie jeden"
    /// ∃!x P(x) — there exists exactly one x such that P(x)
    Unique,
    
    /// Numerical: "three", "five students", "trzech", "pięciu"
    /// |{x : P(x)}| ≥ n
    Numerical(u32),
    
    /// Proportional: "most", "many", "few", "większość", "wielu"
    Proportional(Proportion),
}

pub enum Proportion {
    /// >50% of the domain
    Most,
    
    /// Significant portion (context-dependent, usually >20%)
    Many,
    
    /// Small portion (context-dependent, usually <20%)
    Few,
    
    /// More than two, not many (usually 3-5)
    Several,
}
```

## Quantified Expression

```rust
pub struct QuantifiedExpression {
    /// The quantifier
    pub quantifier: Quantifier,
    
    /// The variable being quantified
    pub variable: Variable,
    
    /// Optional domain restriction
    /// "all students" → domain = STUDENT
    /// "all red books" → domain = BOOK, restriction = RED
    pub domain: Option<Box<InterlinguaNode>>,
    
    /// The body: what is being said about the variable
    pub body: Box<InterlinguaNode>,
    
    /// Optional restrictions (relative clauses)
    /// "all students who passed" → restriction = PASS(x, exam)
    pub restrictions: Vec<Box<InterlinguaNode>>,
}

### Variable

See [INTERLINGUA.md](./INTERLINGUA.md#quantification) for the canonical `Variable` definition:
- `name: String` - variable name (e.g., "x", "y")
- `var_type: Option<Type>` - optional semantic type (e.g., Person, Object, Event)

## Polish Quantifiers

### Universal

| Polish | English | Logical Form |
|--------|---------|--------------|
| wszyscy studenci | all students | ∀x (Student(x) → ...) |
| każdy student | every student | ∀x (Student(x) → ...) |
| każda książka | every book | ∀x (Book(x) → ...) |
| wszystko | everything | ∀x (...) |

**Grammar note:** Polish universal quantifiers agree in gender/number/case with the noun:
- "wszyscy studenci" (virile plural)
- "wszystkie książki" (non-virile plural)
- "każdy student" (masculine singular)
- "każda studentka" (feminine singular)

### Existential

| Polish | English | Logical Form |
|--------|---------|--------------|
| niektórzy studenci | some students | ∃x (Student(x) ∧ ...) |
| jakiś student | some/a student | ∃x (Student(x) ∧ ...) |
| kilka książek | several books | ∃≥3 x (Book(x) ∧ ...) |
| coś | something | ∃x (...) |
| ktoś | someone | ∃x (Person(x) ∧ ...) |

### Negated Existential

| Polish | English | Logical Form |
|--------|---------|--------------|
| żaden student | no student | ¬∃x (Student(x) ∧ ...) |
| nikt | nobody | ¬∃x (Person(x) ∧ ...) |
| nic | nothing | ¬∃x (...) |

**Grammar note:** Polish uses double negation with negated existential:
- "Żaden student **nie** przyszedł" (literally: "No student didn't come")
- "Nikt **nie** widział" (literally: "Nobody didn't see")

### Numerical

| Polish | English | Logical Form |
|--------|---------|--------------|
| trzech studentów | three students | ∃≥3 x (Student(x) ∧ ...) |
| pięć książek | five books | ∃≥5 x (Book(x) ∧ ...) |
| dwoje dzieci | two children | ∃≥2 x (Child(x) ∧ ...) |

**Grammar note:** Polish numerals ≥5 require genitive plural:
- "trzech studentów" (GEN.PL) vs "dwaj studenci" (NOM.PL)
- This affects verb agreement: "trzech studentów czytało" (neuter sg verb)

### Proportional

| Polish | English | Logical Form |
|--------|---------|--------------|
| większość studentów | most students | >50% x (Student(x) ∧ ...) |
| wielu studentów | many students | many x (Student(x) ∧ ...) |
| kilku studentów | several students | several x (Student(x) ∧ ...) |
| mało studentów | few students | few x (Student(x) ∧ ...) |

## English Quantifiers

### Universal

| English | Polish | Logical Form |
|---------|--------|--------------|
| all students | wszyscy studenci | ∀x (Student(x) → ...) |
| every student | każdy student | ∀x (Student(x) → ...) |
| each student | każdy student | ∀x (Student(x) → ...) |
| everything | wszystko | ∀x (...) |

### Existential

| English | Polish | Logical Form |
|---------|--------|--------------|
| some students | niektórzy studenci | ∃x (Student(x) ∧ ...) |
| a student | jakiś student | ∃x (Student(x) ∧ ...) |
| at least one | przynajmniej jeden | ∃x (...) |
| something | coś | ∃x (...) |

### Negated Existential

| English | Polish | Logical Form |
|---------|--------|--------------|
| no students | żadni studenci | ¬∃x (Student(x) ∧ ...) |
| nobody | nikt | ¬∃x (Person(x) ∧ ...) |
| nothing | nic | ¬∃x (...) |

## Logical Form Conversion

### Universal Quantifier

```
"Wszyscy studenci zdali egzamin."

QuantifiedExpression {
    quantifier: Universal,
    variable: Variable { name: "x" },
    domain: Some(STUDENT),
    body: PASS(x, exam),
}

Logical form: ∀x (Student(x) → Pass(x, exam))

Note: Universal quantifier uses IMPLICATION (→), not conjunction (∧).
      This means: "For all x, if x is a student, then x passed the exam."
      If x is not a student, the statement is vacuously true.
```

### Existential Quantifier

```
"Niektórzy studenci zdali egzamin."

QuantifiedExpression {
    quantifier: Existential,
    variable: Variable { name: "x" },
    domain: Some(STUDENT),
    body: PASS(x, exam),
}

Logical form: ∃x (Student(x) ∧ Pass(x, exam))

Note: Existential quantifier uses CONJUNCTION (∧), not implication (→).
      This means: "There exists x such that x is a student AND x passed."
```

### Negated Existential

```
"Żaden student nie przyszedł."

QuantifiedExpression {
    quantifier: NegatedExistential,
    variable: Variable { name: "x" },
    domain: Some(STUDENT),
    body: COME(x),
}

Logical form: ¬∃x (Student(x) ∧ Come(x))
Equivalent:   ∀x (Student(x) → ¬Come(x))
```

### Numerical Quantifier

```
"Trzech studentów czytało książkę."

QuantifiedExpression {
    quantifier: Numerical(3),
    variable: Variable { name: "x" },
    domain: Some(STUDENT),
    body: READ(x, book),
}

Logical form: |{x : Student(x) ∧ Read(x, book)}| ≥ 3

Expanded: ∃x₁ ∃x₂ ∃x₃ (
    Student(x₁) ∧ Student(x₂) ∧ Student(x₃) ∧
    x₁ ≠ x₂ ∧ x₁ ≠ x₃ ∧ x₂ ≠ x₃ ∧
    Read(x₁, book) ∧ Read(x₂, book) ∧ Read(x₃, book)
)
```

## Scope

When a sentence contains multiple quantifiers, their **scope** (which takes precedence) can create different meanings.

### Scope Ambiguity

```
"Każdy student przeczytał jakąś książkę."

Reading 1 (∀ > ∃ — surface scope):
  ∀x ∃y (Student(x) → (Book(y) ∧ Read(x, y)))
  → For each student, there exists a (possibly different) book they read.
  → Each student read their own book.

Reading 2 (∃ > ∀ — inverse scope):
  ∃y ∀x (Book(y) ∧ (Student(x) → Read(x, y)))
  → There exists a single book that every student read.
  → All students read the same book.
```

### Scope Resolution

```rust
pub enum Scope {
    /// Quantifier has wide scope (takes precedence)
    Wide,
    
    /// Quantifier has narrow scope
    Narrow,
    
    /// Scope is ambiguous
    Ambiguous,
}

pub struct ScopeResolution {
    /// Default scope preference (surface order)
    pub default: Vec<(String, Scope)>,
    
    /// Alternative readings
    pub alternatives: Vec<Vec<(String, Scope)>>,
    
    /// Is the scope unambiguous?
    pub is_unambiguous: bool,
}
```

### Scope Heuristics

```
Rule 1: Surface scope (default)
  Quantifiers are interpreted in the order they appear.
  "Every student read some book" → ∀ > ∃

Rule 2: Subject > Object
  Subject quantifier tends to take wide scope.
  "A student saw every cat" → ∃ > ∀ (subject wide)

Rule 3: Specific > Non-specific
  Definite NPs take wide scope over indefinites.
  "The teacher graded every student" → The > ∀

Rule 4: Negation interaction
  "Every student didn't pass" → ambiguous:
    ∀ > ¬ : For each student, they didn't pass (none passed)
    ¬ > ∀ : Not every student passed (some passed, some didn't)
```

## Quantifier Raising

In generative grammar, quantifiers undergo **Quantifier Raising (QR)** — they move to a higher position in the syntactic tree for semantic interpretation.

```
Surface: "Tomek dał każdemu studentowi książkę"
         (Tomek gave every student a book)

Logical Form (after QR):
  [∀x: Student(x)] [∃y: Book(y)] Gave(Tomek, x, y)

→ For every student x, there exists a book y such that Tomek gave y to x.
```

## Cross-Language Quantifier Mapping

### PL → EN

```
wszyscy studenci    → all students
każdy student       → every student
niektórzy studenci  → some students
żaden student       → no student
trzech studentów    → three students
większość studentów → most students
```

### EN → PL

```
all students        → wszyscy studenci
every student       → każdy student
some students       → niektórzy studenci
no student          → żaden student
three students      → trzech studentów
most students       → większość studentów
```

### Generation Notes

When generating quantified expressions:

1. **Polish**: Quantifier agrees with noun in gender, number, case
   - "wszyscy studenci" (NOM, virile)
   - "wszystkich studentów" (ACC/GEN, virile)
   - "wszystkie książki" (NOM/ACC, non-virile)

2. **Polish numerals ≥5**: Require genitive plural + neuter singular verb
   - "pięciu studentów czytało" (GEN.PL + 3SG.N verb)
   - NOT: *"pięciu studentów czytali"

3. **Polish negated existential**: Requires double negation
   - "żaden student nie przyszedł"
   - NOT: *"żaden student przyszedł"

4. **English**: Quantifiers are invariant
   - "all students", "all books" (same form)
   - "every student", "every book" (same form)

## Quantifier Detection During Parsing

```rust
pub fn detect_quantifier(
    token: &str,
    pos: PartOfSpeech,
    context: &ParseContext,
) -> Option<Quantifier> {
    match token {
        // Polish universal
        "wszyscy" | "wszystkie" | "wszystko" => Some(Quantifier::Universal),
        "każdy" | "każda" | "każde" => Some(Quantifier::Universal),
        
        // Polish existential
        "niektórzy" | "niektóre" => Some(Quantifier::Existential),
        "jakiś" | "jakaś" | "jakieś" => Some(Quantifier::Existential),
        "kilka" => Some(Quantifier::Proportional(Proportion::Several)),
        
        // Polish negated existential
        "żaden" | "żadna" | "żadne" => Some(Quantifier::NegatedExistential),
        "nikt" => Some(Quantifier::NegatedExistential),
        "nic" => Some(Quantifier::NegatedExistential),
        
        // Polish proportional
        "większość" => Some(Quantifier::Proportional(Proportion::Most)),
        "wielu" | "wiele" => Some(Quantifier::Proportional(Proportion::Many)),
        "mało" => Some(Quantifier::Proportional(Proportion::Few)),
        
        // English universal
        "all" | "every" | "each" => Some(Quantifier::Universal),
        
        // English existential
        "some" | "a" | "an" => Some(Quantifier::Existential),
        
        // English negated existential
        "no" | "none" | "nobody" | "nothing" | "neither" => Some(Quantifier::NegatedExistential),
        
        // English proportional
        "most" => Some(Quantifier::Proportional(Proportion::Most)),
        "many" => Some(Quantifier::Proportional(Proportion::Many)),
        "few" => Some(Quantifier::Proportional(Proportion::Few)),
        "several" => Some(Quantifier::Proportional(Proportion::Several)),
        
        _ => None,
    }
}
```

## Summary

Quantification in lexFlex:

1. **Captures quantity and scope** — universal, existential, negated, numerical, proportional
2. **Converts to logical form** — ∀, ∃, ¬∃ with proper domain restrictions
3. **Handles scope ambiguity** — multiple quantifiers can have different readings
4. **Maps across languages** — PL ↔ EN quantifier translation
5. **Respects grammar** — Polish agreement, double negation, numeral government
6. **Integrates with Interlingua** — stored as `InterlinguaNode::QuantifiedStatement`
