# Diagnoza Problemów z Benchmarku

## Wyniki Benchmarku (2026-07-10)
- **PL→EN:** 55/55 (100%) - wszystkie zdania przetłumaczone
- **EN→PL:** 39/55 (71%) - 16 błędów
- **Łącznie:** 94/110 (85%)

## Zidentyfikowane Problemy

### 1. 🔴 Podwójna Negacja w EN Generatorze
**Objaw:** "Tomek nie dał jabłka Izie" → "Tomek did not did not give an apple to Iza"

**Przyczyna:** Negacja dodawana dwukrotnie:
- Raz w `generate_transfer` / `generate_two_role` (linie 268-271, 333-336)
- Drugi raz w `finalize_sentence` (linie 173-182)

**Rozwiązanie:** Usunąć dodawanie negacji z frame generation methods, zostawić tylko w `finalize_sentence`.

**Pliki do naprawienia:**
- `/home/ppotepa/git/lexFlex/src/engines/en/generator.rs`

### 2. 🔴 Zamiana Ról w PL Parser
**Objaw:** "Student dał mleko profesorowi" → "Milk gave a student to teacher"

**Przyczyna:** Heurystyka Pass 2 w `build_frame` przypisuje role zbyt prosto:
```rust
// Prefer Theme slot first (post-verbal object position)
if let Some(idx) = roles.iter().position(|r| *r == SemanticRole::Theme) {
    if assigned[idx].is_none() {
        assigned[idx] = Some(entity.clone());
        continue;
    }
}
```

Problem: "student" (NOM, brak specjalnej końcówki) trafia do unassigned, a heurystyka przypisuje go do Theme zamiast Agent.

**Rozwiązanie:** Poprawić heurystykę Pass 2:
1. Sprawdzić pozycję w zdaniu (entity przed czasownikiem = Agent)
2. Sprawdzić animacy (animate entities częściej są agentami)
3. Sprawdzić case (jeśli entity ma NOM i slot Agent jest pusty, przypisz do Agent)

**Pliki do naprawienia:**
- `/home/ppotepa/git/lexFlex/src/engines/pl/parser.rs`

### 3. 🟡 Brakujące Wpisy w Leksykonie
**Objaw:** "Unknown" w outputach, np.:
- "Tomek widział kota" → "Tomek saw an unknown"
- "Tomek kochał tatę" → "Tomek loved an unknown"

**Przyczyna:** Brakuje wpisów w leksykonie:
- "kot" (PL) - brak wpisu
- "tata" (PL) - brak wpisu (jest tylko "father")
- "read" (EN past tense) - brak formy "read" (past)

**Rozwiązanie:** Dodać brakujące wpisy do leksykonów.

**Pliki do naprawienia:**
- `/home/ppotepa/git/lexFlex/data/lexicons/pl/lexicon.ron`
- `/home/ppotepa/git/lexFlex/data/lexicons/en/lexicon.ron`

### 4. 🟡 Brakujące Capabilities w EN Engine
**Objaw:** Błędy "Feature not expressible in target language" dla EN→PL:
- "Tom loved Iza" - EN nie może wyrazić Emotion
- "Tom made a book" - EN nie może wyrazić Creation
- "Tom thought" - EN nie może wyrazić Cognition

**Przyczyna:** EN engine nie ma zadeklarowanych capabilities dla tych frame types.

**Rozwiązanie:** Dodać brakujące capabilities do EN engine.

**Pliki do naprawienia:**
- `/home/ppotepa/git/lexFlex/src/engines/en/mod.rs`

### 5. 🟡 Czasowniki Nie Odmieniają Się Poprawnie
**Objaw:** 
- "Tom ate an apple" → "Tom jeść jabłko" (powinno być "Tom jadł jabłko")
- "Iza drank milk" → "Mleko jeść Unknown" (powinno być "Iza piła mleko")

**Przyczyna:** 
1. EN parser nie rozpoznaje "ate" jako past tense od "eat"
2. PL generator używa lemma zamiast odmienionej formy
3. Zamiana ról (patrz problem 2)

**Rozwiązanie:**
1. Dodać "ate" do EN leksykonu z tense=Past
2. Poprawić PL generator, żeby używał morphology do odmiany czasowników
3. Naprawić zamianę ról

**Pliki do naprawienia:**
- `/home/ppotepa/git/lexFlex/data/lexicons/en/lexicon.ron`
- `/home/ppotepa/git/lexFlex/src/engines/pl/generator.rs`

## Plan Napraw (Priorytety)

### Krytyczne (muszą być naprawione):
1. ✅ Podwójna negacja w EN generatorze
2. ✅ Zamiana ról w PL parser
3. ✅ Brakujące capabilities w EN engine

### Ważne (powinny być naprawione):
4. ✅ Brakujące wpisy w leksykonie (kot, tata, ate, itp.)
5. ✅ Czasowniki nie odmieniają się poprawnie

### Opcjonalne (mogą być naprawione później):
6. Lepsza heurystyka dla frame assignment
7. Lepsze error messages
8. Więcej testów

## Szacowany Czas Napraw
- Problem 1 (negacja): 10 minut
- Problem 2 (role): 30-45 minut
- Problem 3 (leksykon): 20 minut
- Problem 4 (capabilities): 5 minut
- Problem 5 (czasowniki): 15 minut

**Łącznie:** ~1.5-2 godziny

## Oczekiwany Wynik po Naprawach
- PL→EN: 55/55 (100%) → 55/55 (100%)
- EN→PL: 39/55 (71%) → 50-53/55 (91-96%)
- **Łącznie:** 94/110 (85%) → 105-108/110 (95-98%)
