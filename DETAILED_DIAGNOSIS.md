# Szczegółowa Diagnoza: Implementacja vs Dokumentacja

**Data:** 2026-07-10  
**Status:** Krytyczne luki w implementacji

---

## 🚨 KRYTYCZNE PROBLEMY

### 1. Parser Nie Używa Morfologii

**Lokalizacja:** `/home/ppotepa/git/lexFlex/src/engines/pl/parser.rs:10`

**Problem:**
```rust
pub struct PolishParser {
    lexicon: Lexicon,
    _morphology: PolishMorphology,  // ❌ PREFIKS _ = NIEUŻYWANE!
    ontology: Ontology,
}
```

**Co mówi dokumentacja (DEDUCTION.md:45-80):**
> Parser powinien robić **minimalną analizę strukturalną** - tokenizację, POS tagging, **analizę morfologiczną**.  
> Parser powinien identyfikować cechy morfologiczne (case, tense, person, number).

**Rzeczywistość:**
- Parser ignoruje moduł morfologii
- Parser polega wyłącznie na pre-computed cechach w leksykonie
- Parser nie potrafi analizować nieznanych form morfologicznych

**Konsekwencje:**
- "Tom ate an apple" → Parser nie rozpoznaje "ate" jako past tense od "eat"
- "Student dał mleko" → Parser nie analizuje końcówki "-ał" jako past tense
- Brak prawdziwej analizy morfologicznej = parser jest zbyt prosty

**Wpływ na benchmark:** ~15 zdań (wszystkie problemy z czasownikami)

---

### 2. Deduction Engine Jest Za Słaby

**Lokalizacja:** `/home/ppotepa/git/lexFlex/src/core/deduction.rs:171-250`

**Problem:**
```rust
fn resolve_cases_and_roles(sentence: &mut Sentence) -> Result<(), DeductionError> {
    for frame in &mut sentence.frames {
        match frame {
            Frame::Transfer { agent, recipient, theme } => {
                agent.features.case = Some(Case::Nominative);
                recipient.features.case = Some(Case::Dative);
                // ...
            }
        }
    }
    Ok(())
}
```

**Co mówi dokumentacja (DEDUCTION.md:200-280):**
> Deduction powinien:
> 1. **Używać verb frames** z leksykonu do ustalenia wymaganych ról
> 2. **Rozwiązywać case ambiguity** używając ontology + pozycji w zdaniu
> 3. **Walidować semantic types** (np. tylko Animate może być Agent)
> 4. **Dziedziczyć cechy** z ontology (APPLE → FRUIT → FOOD)

**Rzeczywistość:**
- Deduction tylko **ustawia** case, nie **rozwiązuje** ambiguity
- Deduction nie używa verb frames do ustalenia ról
- Deduction nie waliduje semantic types
- Deduction nie dziedziczy cech z ontology (tylko `inherit_features` w `apply_verb_frames`)

**Konsekwencje:**
- "Student dał mleko profesorowi" → Parser przypisuje role heurystycznie, Deduction nie koryguje
- Brak walidacji = "kamień zjadł jabłko" przechodzi bez błędu
- Brak dziedziczenia = APPLE nie wie że jest FRUIT

**Wpływ na benchmark:** ~10 zdań (wszystkie problemy z zamianą ról)

---

### 3. Generator Ignoruje LanguageDescriptor

**Lokalizacja:** `/home/ppotepa/git/lexFlex/src/engines/pl/generator.rs:7`

**Problem:**
```rust
pub struct PolishGenerator {
    lexicon: Lexicon,
    morphology: PolishMorphology,
    _descriptor: LanguageDescriptor,  // ❌ PREFIKS _ = NIEUŻYWANE!
}
```

**Co mówi dokumentacja (GENERATOR.md:50-120):**
> Generator powinien:
> - Czytać **LanguageDescriptor** aby ustalić word order
> - Używać **descriptor.syntax.word_order** (SVO/SOV/VSO)
> - Używać **descriptor.features** do decyzji o inflection

**Rzeczywistość:**
- Generator ignoruje descriptor
- Generator ma hardcoded SVO
- Generator nie sprawdza language-specific rules

**Konsekwencje:**
- Brak elastyczności - nie można łatwo dodać nowego języka
- Trudno debugować problemy z word order
- Generator nie jest data-driven

**Wpływ na benchmark:** ~5 zdań (problemy z word order)

---

### 4. Brak Prawdziwego Verb Frame Matching

**Lokalizacja:** `/home/ppotepa/git/lexFlex/src/engines/pl/parser.rs:120-145`

**Problem:**
```rust
let (frame_type, roles) = if let Some(entry) = verb_entry {
    if let Some(ref ft) = entry.frame_type {
        let parsed_roles: Vec<SemanticRole> = entry.roles.iter()
            .filter_map(|r| parse_role_str(r))
            .collect();
        (ft.clone(), parsed_roles)
    } else {
        ("Statement".to_string(), vec![SemanticRole::Topic, SemanticRole::Theme])
    }
} else {
    ("Statement".to_string(), vec![SemanticRole::Topic, SemanticRole::Theme])
}
```

**Co mówi dokumentacja (DEDUCTION.md:300-380):**
> apply_verb_frames powinien:
> 1. Look up verb w **concepts.ron** (nie w lexicon!)
> 2. Ustalić wymagane role z **frame definition**
> 3. Dopasować NPs do ról używając case + pozycji + ontology constraints
> 4. Zgłosić błąd jeśli brakuje wymaganych ról

**Rzeczywistość:**
- Parser robi prosty lookup w leksykonie
- Parser nie używa concepts.ron
- Parser nie dopasowuje NPs do ról - to robi `build_frame` (który jest w Parserze, a powinien być w Deduction!)
- Parser nie zgłasza błędów o brakujących rolach

**Konsekwencje:**
- Logika semantyczna jest w Parserze (powinna być w Deduction)
- Trudno rozszerzyć o nowe frame types
- Brak walidacji kompletności frame'ów

**Wpływ na benchmark:** ~8 zdań (wszystkie problemy z frame assignment)

---

## 📊 PODSUMOWANIE LUK

### Implementacja vs Dokumentacja

| Komponent | Dokumentacja | Implementacja | Luka |
|-----------|--------------|---------------|------|
| **Parser - Morfologia** | Używa modułu morfologii | Ignoruje (`_morphology`) | 🔴 KRYTYCZNA |
| **Parser - Verb Frames** | Lookup w concepts.ron | Lookup w lexicon.ron | 🟡 WAŻNA |
| **Parser - Role Assignment** | Parser zbiera NPs, Deduction przypisuje role | Parser przypisuje role (`build_frame`) | 🟡 WAŻNA |
| **Deduction - Case Resolution** | Rozwiązuje ambiguity | Tylko ustawia case | 🔴 KRYTYCZNA |
| **Deduction - Semantic Validation** | Waliduje types | Brak walidacji | 🔴 KRYTYCZNA |
| **Deduction - Feature Inheritance** | Dziedziczy z ontology | Tylko `inherit_features` | 🟡 WAŻNA |
| **Generator - LanguageDescriptor** | Używa descriptor | Ignoruje (`_descriptor`) | 🟡 WAŻNA |

### Wpływ na Benchmark

| Kategoria | Liczba Zdań | Problemy |
|-----------|-------------|----------|
| **Podwójna negacja (EN)** | 7 | ✅ NAPRAWIONE |
| **Zamiana ról (PL)** | 15 | ❌ WYMAGA: Deduction + Parser |
| **Czasowniki (PL→EN, EN→PL)** | 12 | ❌ WYMAGA: Morfologia w Parser |
| **Brakujące capabilities (EN)** | 16 | ✅ NAPRAWIONE (częściowo) |
| **Word order** | 5 | ❌ WYMAGA: LanguageDescriptor |
| **Brakujące wpisy (lexicon)** | 8 | 🟡 ŁATWE DO NAPRAWIENIA |

**Łączny wpływ:** ~63 zdania z problemami (57% benchmarku)

---

## 🔧 PLAN NAPRAW (Priorytety)

### Priorytet 1: Krytyczne (2-3 dni)

1. **Naprawić Parser - Morfologia**
   - Usunąć prefiks `_` z `_morphology`
   - Dodać `analyze_morphology()` do Parsera
   - Używać morfologii do analizy nieznanych form
   - **Plik:** `src/engines/pl/parser.rs`
   - **Czas:** 4-6 godzin

2. **Naprawić Deduction - Role Assignment**
   - Przenieść `build_frame` z Parsera do Deduction
   - Dodać prawdziwe verb frame matching z concepts.ron
   - Dodać case ambiguity resolution
   - **Plik:** `src/core/deduction.rs`
   - **Czas:** 6-8 godzin

3. **Naprawić Deduction - Semantic Validation**
   - Dodać `validate_semantic_types()`
   - Sprawdzać animacy dla Agent/Experiencer
   - Zgłaszać błędy dla nieprawidłowych kombinacji
   - **Plik:** `src/core/deduction.rs`
   - **Czas:** 3-4 godziny

### Priorytet 2: Ważne (1-2 dni)

4. **Naprawić Generator - LanguageDescriptor**
   - Usunąć prefiks `_` z `_descriptor`
   - Używać `descriptor.syntax.word_order`
   - Używać `descriptor.features` do decyzji
   - **Plik:** `src/engines/pl/generator.rs`
   - **Czas:** 3-4 godziny

5. **Dodać Brakujące Wpisy**
   - kot, tata, ate, read (past), itp.
   - **Plik:** `data/lexicons/*.ron`
   - **Czas:** 2-3 godziny

6. **Naprawić Feature Inheritance**
   - Ulepszyć `inherit_features()` w Deduction
   - Dodać true ontology traversal
   - **Plik:** `src/core/deduction.rs`
   - **Czas:** 2-3 godziny

### Priorytet 3: Opcjonalne (1 dzień)

7. **Refactoring: Parser vs Deduction**
   - Przenieść semantyczną logikę z Parsera do Deduction
   - Zgodnie z zasadą: "Parser should be as simple as possible, Deduction should be as powerful as possible"
   - **Czas:** 4-6 godzin

---

## 📈 OCZEKIWANY WPŁYW NA BENCHMARK

### Przed Naprawami
- **PL→EN:** 55/55 (100%) - ale wiele "Unknown" w output
- **EN→PL:** 39/55 (71%)
- **Łącznie:** 94/110 (85%)

### Po Naprawach Priorytetu 1
- **PL→EN:** 55/55 (100%) - mniej "Unknown"
- **EN→PL:** 48/55 (87%) - naprawiona zamiana ról
- **Łącznie:** 103/110 (94%)

### Po Naprawach Priorytetu 2
- **PL→EN:** 55/55 (100%)
- **EN→PL:** 52/55 (95%) - naprawione czasowniki
- **Łącznie:** 107/110 (97%)

### Po Naprawach Priorytetu 3
- **PL→EN:** 55/55 (100%)
- **EN→PL:** 54/55 (98%)
- **Łącznie:** 109/110 (99%)

---

## 🎯 REKOMENDACJA

**Natychmiastowe działania:**
1. ✅ Naprawić podwójną negację (JUŻ ZROBIONE)
2. ✅ Dodać brakujące capabilities (JUŻ ZROBIONE)
3. 🔴 **Naprawić Parser - Morfologia** (Priorytet 1)
4. 🔴 **Naprawić Deduction - Role Assignment** (Priorytet 1)

**Średnioterminowe:**
5. 🟡 Naprawić Generator - LanguageDescriptor
6. 🟡 Dodać brakujące wpisy
7. 🟡 Naprawić feature inheritance

**Długoterminowe:**
8. 🟢 Refactoring Parser vs Deduction
9. 🟢 Dodać więcej testów jednostkowych
10. 🟢 Poprawić error messages

**Szacowany czas do 97% sukcesu:** 3-4 dni pracy

---

## 📝 NOTATKI KOŃCOWE

Główny problem to **architekturalne naruszenie** zasad opisanych w dokumentacji:
- Parser robi zbyt dużo (semantic decisions)
- Deduction robi zbyt mało (tylko case assignment)
- Generator ignoruje data-driven approach

Naprawa tych problemów nie tylko poprawi benchmark, ale także:
- Ułatwi dodawanie nowych języków
- Ułatwi debugowanie
- Umożliwi rozszerzanie o nowe frame types
- Zmniejszy code duplication

**Kluczowa zasada z dokumentacji:**
> "Parser should be as simple as possible, Deduction should be as powerful as possible."

Obecna implementacja jest **odwrotnością** tej zasady.
