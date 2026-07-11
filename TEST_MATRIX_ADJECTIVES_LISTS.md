# Test Matrix: Adjectives + Enumerations (Concept-Driven)

**Cel**: Systematyczne pokrycie przymiotników i wyliczeń w sposób w pełni algorytmiczny na bazie konceptów.

**Zasada**:
- Opisujemy testy na poziomie **konceptów + cech**, nie surface strings.
- Generator / my ręcznie budujemy `Entity` z `adjectives: Vec<Entity>` i `coordination`.
- Tylko prawdziwe wyjątki leksykalne (suppletives) pochodzą z leksykonu.
- Wszystko inne (agreement, stopień, stacking, rozkład w koordynacji) musi działać algorytmicznie.

---

## Wymiary macierzy (do pokrycia)

### 1. Adjectives
- Liczba przymiotników na NP: 0, 1, 2, 3
- Rodzaje przymiotników: Size (BIG/SMALL), Quality (GOOD/BAD), Color (RED/BLUE/GREEN), Temperature (HOT/COLD)
- Stopień: Positive, Comparative, Superlative (na 0, 1 lub kilku przymiotnikach)

### 2. Enumerations / Coordination
- Arność: 2 lub 3 elementy
- Pozycja koordynacji: Subject, Theme, Both, None
- Rozkład przymiotników w koordynacji:
  - Shared (przed spójką)
  - Distributed (różne na różnych elementach)
  - Na konkretnym elemencie
  - Na wszystkich

### 3. Interakcje
- Adjectives + Number (Sg/Pl)
- Adjectives + Quantification
- Adjectives + Negation / Question
- Adjectives + Case effects (PL)
- Adjectives + Degree + Coordination

### 4. Frames
GIVE, SEE, EAT/DRINK, LOVE/HATE, MAKE, HAVE, GO, BE, ...

---

## Macierz testów (uzupełniamy ręcznie)

**Format opisu przypadku (zalecany):**

```
Frame: GIVE
Agent: PERSON + adjectives=[GOOD] + degree=Comparative
Theme: APPLE + adjectives=[RED, BIG]
Coordination: Theme (APPLE + BOOK)
Features: negation=false, question=true, number=sg
Direction: PL->EN
```

Następnie w kolumnie "Generated / Expected" wpisujemy co wyszło lub co powinno wyjść.

| ID     | Frame     | Agent (concept + adjs + degree)      | Theme (concept + adjs + degree)       | Coordination                  | Other Features                  | Direction | Status     | Generated Surface / Notes |
|--------|-----------|--------------------------------------|---------------------------------------|-------------------------------|---------------------------------|-----------|------------|---------------------------|
| A-001  | GIVE      | PERSON + []                          | APPLE + [BIG]                         | None                          | -                               | PL→EN    | TODO       |                           |
| A-002  | GIVE      | PERSON + [GOOD]                      | APPLE + [RED]                         | None                          | -                               | PL→EN    | TODO       |                           |
| A-003  | GIVE      | PERSON + [BIG, RED]                  | APPLE + []                            | None                          | -                               | PL→EN    | TODO       | stacked 2 adjs            |
| A-004  | GIVE      | PERSON + [GOOD, Comp]                | APPLE + [RED]                         | None                          | -                               | PL→EN    | TODO       | comparative               |
| A-005  | SEE       | CAT + [SMALL]                        | PERSON + []                           | None                          | -                               | EN→PL    | TODO       |                           |
| A-010  | GIVE      | PERSON + []                          | APPLE + [BIG]                         | Theme (2)                     | -                               | PL→EN    | TODO       | list + adj on theme       |
| A-011  | GIVE      | PERSON + [GOOD]                      | APPLE + []                            | Theme (2)                     | -                               | PL→EN    | TODO       | shared adj before list    |
| A-012  | GIVE      | PERSON + []                          | APPLE + [RED] / BOOK + [BLUE]         | Theme (2)                     | -                               | PL→EN    | TODO       | distributed adjs          |
| A-013  | GIVE      | PERSON + [BIG] / STUDENT + [SMALL]   | APPLE + []                            | Subject (2)                   | -                               | PL→EN    | TODO       | adjs on coordinated subj  |
| A-020  | LOVE      | PERSON + [GOOD, Comp]                | PERSON + [BAD]                        | Both                          | question=true                   | EN→PL    | TODO       | degree + coord + question |
| A-021  | EAT       | PERSON + []                          | APPLE + [RED, BIG]                    | None                          | negation=true, number=pl        | PL→EN    | TODO       | stacked + neg + plural    |
| A-030  | GIVE      | PERSON + [BIG]                       | APPLE + [GOOD, Super]                 | Theme (3)                     | quant=Many                      | PL→EN    | TODO       | 3 items + superlative     |
| A-031  | SEE       | CAT + [SMALL] / DOG + [BIG]          | PERSON + [GOOD]                       | Subject (2)                   | -                               | EN→PL    | TODO       | mixed types + adj         |
| A-040  | HAVE      | PERSON + [GOOD]                      | CAT + [RED]                           | None                          | -                               | PL→EN    | TODO       | possession + adj          |
| A-041  | MAKE      | PERSON + [SMALL]                     | HOUSE + [BIG]                         | None                          | degree on adj = Comp            | EN→PL    | TODO       | creation + comp           |

---

## Kategorie do uzupełnienia (cele pokrycia)

### Kategoria A: Stacked Adjectives (bez koordynacji)
- 1 adj
- 2 adj (różne typy: size+color, quality+color)
- 3 adj
- Degree na pierwszym / ostatnim / wszystkich

### Kategoria B: Coordination + Adjectives
- Shared adjective przed spójką
- Adjective tylko na pierwszym elemencie
- Adjective tylko na drugim
- Różne przymiotniki na różnych elementach
- Adjectives na coordinated subject (wpływ na verb)

### Kategoria C: Degree w wyliczeniach
- Comparative na jednym z elementów listy
- Superlative
- Stopień + koordynacja podmiotu

### Kategoria D: Interakcje z innymi cechami
- Adjectives + Quantification
- Adjectives + Negation
- Adjectives + Questions
- Adjectives + Temporals
- Adjectives + Number (Sg/Pl) + case effects (PL)

### Kategoria E: Różne role
- Adjectives na Agencie
- Adjectives na Theme
- Adjectives na Recipient (rzadziej)

---

## Jak korzystać z tej macierzy

1. Wypełniamy wiersze na poziomie konceptów.
2. Dla każdego wypełnionego wiersza:
   - Budujemy ręcznie lub przez mały helper `Entity`
   - Uruchamiamy przez prawdziwy pipeline
   - Wpisujemy wynik w kolumnie "Generated Surface"
3. Gdy macierz będzie gęsto wypełniona → generujemy z niej `benchmark_sentences.txt` lub testy.

**Zalecany format wpisu w kolumnie "Agent / Theme":**

```
PERSON + [GOOD, Comparative]
APPLE + [RED, BIG]
```

Lub bardziej szczegółowo:
```
Agent: concept=PERSON, adjs=[{concept=GOOD, degree=Comp}], number=Sg
```

---

## Status pokrycia (aktualizujemy)

- Stacked adjectives (1–2): TODO
- Stacked 3+: TODO
- Degree (Comp/Super): TODO
- Coordination + shared adj: TODO
- Distributed adjectives in lists: TODO
- Adjs on coordinated subject: TODO
- Adjs + quant/neg/question: TODO

**Cel**: minimum 80–100 dobrych, różnorodnych przypadków z tej macierzy w ciągu najbliższych iteracji.

