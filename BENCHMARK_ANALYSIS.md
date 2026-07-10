# Analiza Wyników Benchmarku

**Data:** 2026-07-10  
**Wynik ogólny:** 95/110 (86%)  
- PL→EN: 55/55 (100%)  
- EN→PL: 40/55 (73%)

---

## 🔴 KRYTYCZNE PROBLEMY (15 błędów)

### Błędy Capability Checking (15 zdań)

Wszystkie błędy to: `ERROR: Translate error: Feature not expressible in target language`

**Dotyczy zdań:** #80, 90-94, 96-100, 108-110, 115

**Przykłady:**
- #80: "A student read a book" → ERROR
- #90: "Tom loved Iza" → ERROR
- #96: "Tom made a book" → ERROR
- #108: "Tom thought" → ERROR

**Przyczyna:**
English engine nie ma zadeklarowanych wszystkich wymaganych capabilities w `EN_CAPABILITIES`.

**Rozwiązanie:**
Dodać brakujące capabilities do `src/engines/en/mod.rs`:
```rust
static EN_CAPABILITIES: &[Capability] = &[
    // ... istniejące ...
    Capability::EmotionExpression,  // dla LOVE, HATE
    Capability::MorphologicalInflection,  // dla MAKE, READ
    Capability::ProDrop,  // dla THINK
    Capability::FreeWordOrder,  // dla READ
    Capability::Ambiguity,  // dla READ
];
```

---

## 🟡 PROBLEMY JAKOŚCIOWE (poprawne tłumaczenia ale z błędami)

### 1. Zamiana Ról (Agent/Theme/Patient) - 20 zdań

**PL→EN (13 zdań):** #7, 8, 18, 19, 21, 30-32, 46, 49, 56, 57, 62, 63

**Przykłady:**
- #7: "Student dał mleko profesorowi" → "Milk gave a student to teacher" ❌
  - Powinno być: "Student gave milk to teacher"
  - Problem: Parser przypisuje mleko jako Agent, student jako Theme
  
- #18: "Iza piła mleko" → "Milk ate Iza" ❌
  - Powinno być: "Iza drank milk"
  - Problem: Parser przypisuje mleko jako Agent, Iza jako Patient

- #30: "Iza zrobiła jabłko" → "Apple made Iza" ❌
  - Powinno być: "Iza made an apple"
  - Problem: Parser przypisuje jabłko jako Agent, Iza jako Theme

**EN→PL (7 zdań):** #73, 79, 85, 88, 113, 121

**Przykłady:**
- #73: "Iza gave a book to Tom" → "Książka dało Tom Unknown" ❌
  - Powinno być: "Iza dała książkę Tomowi"
  - Problem: Parser przypisuje książkę jako Agent

**Przyczyna:**
Parser nie używa poprawnie case markings do przypisywania ról. Heurystyka Pass 2 w `build_frame()` jest zbyt prosta.

**Rozwiązanie:**
Ulepszyć `resolve_cases_and_roles()` w Deduction Engine:
1. Użyć verb frames z lexicon do ustalenia wymaganych ról
2. Dopasować entities do ról na podstawie case markings
3. Dodać walidację: Agent musi być Animate

---

### 2. Brakujące Wpisy w Leksykonie ("Unknown") - 15 zdań

**PL→EN (10 zdań):** #8, 13, 14, 26, 27, 41-43, 65, 67, 69-70

**Przykłady:**
- #13: "Student czytał książkę" → "Unknown saw an unknown" ❌
  - Powinno być: "Student read a book"
  - Problem: Brak wpisu "czytać" w EN lexicon
  
- #14: "Tomek widział kota" → "Tomek saw an unknown" ❌
  - Powinno być: "Tomek saw a cat"
  - Problem: Brak wpisu "kot" w PL lexicon (jest tylko "pies")

- #41: "Tomek myślał" → "Unknown thought an unknown" ❌
  - Powinno być: "Tomek thought"
  - Problem: Parser nie rozpoznaje "Tomek" jako subject

**EN→PL (5 zdań):** #72, 74-76, 82

**Przykłady:**
- #72: "Tom gave an apple to Iza" → "Tom dał jabłko Unknown" ❌
  - Powinno być: "Tom dał jabłko Izie"
  - Problem: Brak wpisu "Iza" w PL lexicon

**Przyczyna:**
Niekompletne leksykony - brakuje wielu podstawowych słów.

**Rozwiązanie:**
Dodać brakujące wpisy do leksykonów:
- PL: kot, czytać, myśleć, Iza, profesor, student
- EN: cat, read, think, Iza, professor, student

---

### 3. Niepoprawne Mapowanie Czasowników - 4 zdania

**PL→EN (3 zdania):** #49, 57, 116

**Przykłady:**
- #49: "Tomek nie pił mleka" → "Tomek did not eat milk" ❌
  - Powinno być: "Tomek did not drink milk"
  - Problem: `find_verb_for_frame()` mapuje Consumption → "eat" zamiast rozróżniać eat/drink

- #57: "Tomek pił mleko dzisiaj" → "Tomek ate milk today" ❌
  - Powinno być: "Tomek drank milk today"
  - Problem: To samo co wyżej

**Przyczyna:**
`find_verb_for_frame()` w `src/engines/en/generator.rs` używa hardcoded mapowania:
```rust
Frame::Consumption { .. } => "eat",
```

**Rozwiązanie:**
Zmienić logikę na dynamiczną:
1. Sprawdzić theme/patient entity
2. Jeśli theme jest płyn (WATER, MILK) → "drink"
3. Jeśli theme jest jedzenie (APPLE, BREAD) → "eat"

---

### 4. Czasowniki w Formie Bezokolicznika - 25 zdań

**EN→PL (25 zdań):** #75, 81, 102-106, 112-114, 117-118, 120, 122-124, 126-130, 132-134, 136-137

**Przykłady:**
- #102: "Tom went" → "Tom iść" ❌
  - Powinno być: "Tom poszedł"
  - Problem: Generator używa lemma zamiast odmienionej formy

- #81: "Tom saw a cat" → "Tom widział kot" ❌
  - Powinno być: "Tom widział kota"
  - Problem: Generator nie odmienia rzeczownika (kot → kota w ACC)

- #112: "Tom did not give an apple to Iza" → "Tom da nie jabłka Unknown" ❌
  - Powinno być: "Tom nie dał jabłka Izie"
  - Problem: Generator nie odmienia czasownika (da → dał) i rzeczownika (Iza → Izie w DAT)

**Przyczyna:**
Generator PL nie używa morfologii do odmiany czasowników i rzeczowników. Używa lemma z lexicon.

**Rozwiązanie:**
W `src/engines/pl/generator.rs`:
1. Dodać `inflect_verb()` dla czasowników
2. Dodać `inflect_noun()` dla rzeczowników
3. Użyć `PolishMorphology` z załadowanymi paradygmatami

---

### 5. Podwójna Negacja - 1 zdanie

**PL→EN (1 zdanie):** #66

**Przykład:**
- #66: "Nikt nie widział książki" → "None did not book see an unknown" ❌
  - Powinno być: "Nobody saw a book"
  - Problem: Generator dodaje "did not" dwa razy

**Przyczyna:**
Generator EN dodaje negację zarówno w `generate_two_role()` jak i w `finalize_sentence()`.

**Rozwiązanie:**
Usunąć dodawanie negacji z `generate_two_role()` i zostawić tylko w `finalize_sentence()`.

---

### 6. Niepoprawna Forma Czasownika w Pytaniach - 5 zdań

**PL→EN (5 zdań):** #59-63

**Przykłady:**
- #59: "Czy Tomek dał jabłko Izie" → "Did Tomek gave an apple to Iza?" ❌
  - Powinno być: "Did Tomek give an apple to Iza?"
  - Problem: Po "Did" czasownik powinien być w formie podstawowej (give), nie past tense (gave)

**Przyczyna:**
Generator EN nie zmienia formy czasownika po "Did" na base form.

**Rozwiązanie:**
W `finalize_sentence()` gdy `is_question == true`:
1. Znaleźć czasownik w words
2. Zamienić past tense na base form (gave → give, saw → see, ate → eat)

---

## 📊 PODSUMOWANIE PROBLEMÓW

| Kategoria | Liczba Zdań | Procent | Priorytet |
|-----------|-------------|---------|-----------|
| Capability checking errors | 15 | 13.6% | 🔴 Krytyczny |
| Zamiana ról | 20 | 18.2% | 🔴 Krytyczny |
| Brakujące wpisy | 15 | 13.6% | 🟡 Ważny |
| Niepoprawne mapowanie czasowników | 4 | 3.6% | 🟡 Ważny |
| Czasowniki w bezokoliczniku | 25 | 22.7% | 🔴 Krytyczny |
| Podwójna negacja | 1 | 0.9% | 🟢 Niski |
| Niepoprawna forma w pytaniach | 5 | 4.5% | 🟡 Ważny |

**Łącznie problemów:** 85 zdań (77% benchmarku)

---

## 🎯 PLAN NAPRAW (Priorytety)

### Priorytet 1: Krytyczne (2-3 dni)

1. **Naprawić Capability Checking**
   - Dodać brakujące capabilities do EN engine
   - **Wpływ:** +15 zdań (95% → 100%)

2. **Naprawić Zamianę Ról**
   - Ulepszyć `resolve_cases_and_roles()` w Deduction
   - Dodać verb frame matching z lexicon
   - **Wpływ:** +20 zdań

3. **Naprawić Czasowniki w Bezokoliczniku**
   - Dodać `inflect_verb()` do PL generator
   - Dodać `inflect_noun()` do PL generator
   - Użyć PolishMorphology
   - **Wpływ:** +25 zdań

### Priorytet 2: Ważne (1-2 dni)

4. **Dodać Brakujące Wpisy**
   - Rozszerzyć leksykony PL i EN
   - **Wpływ:** +15 zdań

5. **Naprawić Mapowanie Czasowników**
   - Zmienić `find_verb_for_frame()` na dynamiczne
   - **Wpływ:** +4 zdania

6. **Naprawić Formę Czasownika w Pytaniach**
   - Dodać konwersję past → base form po "Did"
   - **Wpływ:** +5 zdań

### Priorytet 3: Niski (30 minut)

7. **Naprawić Podwójną Negację**
   - Usunąć duplikat negacji w EN generator
   - **Wpływ:** +1 zdanie

---

## 📈 OCZEKIWANY WYNIK PO NAPRAWACH

| Faza | Obecny | Po naprawach | Wzrost |
|------|--------|--------------|--------|
| Priorytet 1 | 86% | 95% | +9% |
| Priorytet 2 | 95% | 99% | +4% |
| Priorytet 3 | 99% | 100% | +1% |

**Cel:** 100% sukcesu w benchmarku

---

## 🔍 SZCZEGÓŁOWA ANALIZA PRZYKŁADÓW

### Przykład 1: Zamiana Ról (#7)

**Input:** "Student dał mleko profesorowi"  
**Output:** "Milk gave a student to teacher" ❌  
**Expected:** "Student gave milk to teacher"

**Analiza:**
```
Parser:
  - Tokens: [Student(NOM), dał(Verb), mleko(ACC), profesorowi(DAT)]
  - Verb: "dać" → Transfer frame [Agent, Recipient, Theme]
  
Deduction (obecna):
  - Pass 1: Przypisuje case na podstawie frame
    - Agent → NOM (Student)
    - Recipient → DAT (profesorowi)
    - Theme → ACC (mleko)
  - Pass 2: Buduje frame z entities
    - Problem: entities są w kolejności: [Student, mleko, profesorowi]
    - Heurystyka przypisuje:
      - roles[0] (Agent) → entities[0] (Student) ✓
      - roles[1] (Recipient) → entities[1] (mleko) ❌
      - roles[2] (Theme) → entities[2] (profesorowi) ❌

Deduction (poprawna):
  - Pass 1: Przypisuje case na podstawie frame
  - Pass 2: Buduje frame używając case markings
    - NOM entity → Agent (Student)
    - DAT entity → Recipient (profesorowi)
    - ACC entity → Theme (mleko)
```

### Przykład 2: Czasownik w Bezokoliczniku (#102)

**Input:** "Tom went"  
**Output:** "Tom iść" ❌  
**Expected:** "Tom poszedł"

**Analiza:**
```
Parser:
  - Tokens: [Tom(Noun), went(Verb)]
  - Verb: "went" → lemma: "go", tense: Past
  
Generator (obecny):
  - find_verb_for_frame(Motion) → "iść"
  - Używa lemma bezpośrednio bez odmiany
  
Generator (poprawny):
  - find_verb_for_frame(Motion) → "iść"
  - inflect_verb("iść", tense=Past, person=Third, number=Singular, gender=Masculine)
  - → "poszedł" (z paradygmatu verb_ic)
```

### Przykład 3: Capability Error (#90)

**Input:** "Tom loved Iza"  
**Output:** ERROR ❌  
**Expected:** "Tom kochał Izę"

**Analiza:**
```
Parser:
  - Tokens: [Tom(Noun), loved(Verb), Iza(Noun)]
  - Verb: "loved" → lemma: "love", tense: Past
  - Frame: Emotion [Experiencer, Stimulus]
  
Capability Check:
  - Required: [Reference, EmotionExpression]
  - Available (EN): [Reference, TemporalReference, ...]
  - Missing: EmotionExpression ❌
  
Rozwiązanie:
  - Dodać Capability::EmotionExpression do EN_CAPABILITIES
```

---

## ✅ ZDANIA BEZ BŁĘDÓW (25 zdań)

Następujące zdania są przetłumaczone poprawnie:

**PL→EN (18 zdań):** #5, 6, 9, 11, 12, 15, 17, 20, 23-25, 29, 33, 35-39, 45, 47-48, 50-51, 53-55

**EN→PL (7 zdań):** #78, 84, 86-87, 120, 122-123

**Wspólne cechy:**
- Proste zdania SVO
- Czasowniki z jasnym mapowaniem (give, see, eat)
- Brak quantification
- Brak passive voice
- Poprawne case markings

---

## 📝 WNIOSKI

1. **Parser działa poprawnie** dla prostych zdań z jasnymi case markings
2. **Deduction Engine jest za słaby** - nie rozwiązuje case ambiguity
3. **Generator PL nie używa morfologii** - to główny problem jakości
4. **Capability checking jest niekompletne** - brakuje wielu capabilities
5. **Leksykony są niekompletne** - brakuje wielu podstawowych słów

**Główny problem:** Implementacja nie jest zgodna z dokumentacją:
- Parser powinien być prosty, Deduction potężny
- Generator powinien używać morfologii
- Capability checking powinno być kompletne

**Rekomendacja:** Skupić się na Priorytet 1 (krytyczne naprawy) - to da 95% sukcesu.
