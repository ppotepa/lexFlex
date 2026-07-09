# Performance Guide — Optimization and Benchmarks

This document provides guidance on performance optimization, benchmarking, and expected performance characteristics of lexFlex v0.1.

## Performance Targets

### v0.1 Performance Goals

| Metric | Target | Notes |
|--------|--------|-------|
| Translation latency | < 100ms | Per simple sentence |
| Memory usage | < 50MB | Full system loaded |
| Throughput | > 10 sentences/sec | Sustained |
| Startup time | < 500ms | Load all components |
| Accuracy | > 80% | On test suite |

### Performance Characteristics

**Complexity Analysis:**

| Component | Time Complexity | Space Complexity | Notes |
|-----------|----------------|------------------|-------|
| Tokenization | O(n) | O(n) | n = input length |
| Morphology | O(n × p) | O(p) | p = number of paradigms |
| Parsing | O(n²) | O(n²) | Dependency parsing |
| Deduction | O(n²) | O(n) | n = number of NPs |
| Generation | O(n) | O(n) | n = number of roles |

**Overall:** O(n²) for simple sentences, where n = number of words.

## Optimization Strategies

### 1. Morphology Optimization

**Problem:** Morphology lookup is O(n × p) where p = number of paradigms.

**Solution:** Use indexed lookup with caching.

```rust
pub struct OptimizedMorphology {
    // Index by lemma for fast lookup
    lemma_index: HashMap<String, Vec<MorphParadigm>>,
    
    // Cache recent lookups
    cache: LruCache<(String, FeatureBundle), String>,
}

impl OptimizedMorphology {
    pub fn inflect(
        &mut self,
        lemma: &str,
        features: &FeatureBundle,
    ) -> Result<String, MorphError> {
        // Check cache first
        let cache_key = (lemma.to_string(), features.clone());
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(cached.clone());
        }
        
        // Look up paradigms
        let paradigms = self.lemma_index.get(lemma)
            .ok_or(MorphError::UnknownLemma(lemma.to_string()))?;
        
        // Apply first matching paradigm
        for paradigm in paradigms {
            if let Ok(result) = paradigm.apply(features) {
                // Cache result
                self.cache.put(cache_key, result.clone());
                return Ok(result);
            }
        }
        
        Err(MorphError::NoMatchingParadigm)
    }
}
```

**Expected improvement:** 10-20x speedup for repeated forms.

### 2. Lexicon Optimization

**Problem:** Lexicon lookup requires scanning all entries.

**Solution:** Use indexed lookup with concept ID.

```rust
pub struct OptimizedLexicon {
    // Index by concept ID for fast lookup
    concept_index: HashMap<ConceptId, Vec<LexEntry>>,
    
    // Index by lemma for reverse lookup
    lemma_index: HashMap<String, LexEntry>,
    
    // Index by language
    language_index: HashMap<LanguageId, HashMap<String, LexEntry>>,
}

impl OptimizedLexicon {
    pub fn lookup_by_concept(
        &self,
        concept: &ConceptId,
        language: LanguageId,
    ) -> Option<&LexEntry> {
        self.concept_index.get(concept)?
            .iter()
            .find(|entry| entry.language == language)
    }
    
    pub fn lookup_by_lemma(
        &self,
        lemma: &str,
        language: LanguageId,
    ) -> Option<&LexEntry> {
        self.language_index.get(&language)?
            .get(lemma)
    }
}
```

**Expected improvement:** O(1) lookup instead of O(n).

### 3. Deduction Optimization

**Problem:** Deduction is O(n²) due to role assignment.

**Solution:** Use early termination and priority queues.

```rust
pub fn optimize_deduction(
    sentence: &mut Sentence,
    context: &DeductionContext,
) -> Result<(), DeductionError> {
    
    // Step 1: Quick verb lookup
    let verb = find_main_verb(sentence)?;
    let frame_template = context.lexicon.lookup_verb(&verb.lemma)?;
    
    // Step 2: Collect NPs with explicit cases first
    let mut explicit_nps: Vec<_> = sentence.tokens.iter()
        .filter(|t| t.pos == PartOfSpeech::Noun && t.features.case.is_some())
        .collect();
    
    // Step 3: Assign explicit cases immediately (O(n))
    let mut assignments = Vec::new();
    for np in &explicit_nps {
        if let Some(role) = find_role_for_case(np.features.case.unwrap(), &frame_template) {
            assignments.push((role, np));
        }
    }
    
    // Step 4: Only process ambiguous NPs (usually 0-2)
    let ambiguous_nps: Vec<_> = sentence.tokens.iter()
        .filter(|t| t.pos == PartOfSpeech::Noun && t.features.case.is_none())
        .collect();
    
    // Step 5: Resolve ambiguities (O(k²) where k = number of ambiguous NPs)
    if ambiguous_nps.len() <= 2 {
        // Use simple algorithm for small k
        resolve_simple(&mut assignments, &ambiguous_nps, &frame_template)?;
    } else {
        // Use constraint solver for large k
        solve_constraints(&mut assignments, &ambiguous_nps, &frame_template)?;
    }
    
    Ok(())
}
```

**Expected improvement:** 5-10x speedup for typical sentences (k ≤ 2).

### 4. Caching Strategy

**Problem:** Repeated processing of same structures.

**Solution:** Multi-level caching.

```rust
pub struct TranslationCache {
    // Level 1: Full sentence cache
    sentence_cache: LruCache<String, String>,
    
    // Level 2: InterlinguaNode cache
    interlingua_cache: LruCache<String, InterlinguaNode>,
    
    // Level 3: Morphology cache (in OptimizedMorphology)
    // Already implemented
}

impl TranslationCache {
    pub fn translate_cached(
        &mut self,
        input: &str,
        from: LanguageId,
        to: LanguageId,
        translator: &UniversalTranslator,
    ) -> Result<String, TranslateError> {
        
        // Check sentence cache
        let cache_key = format!("{}:{}:{}", from, to, input);
        if let Some(cached) = self.sentence_cache.get(&cache_key) {
            return Ok(cached.clone());
        }
        
        // Check interlingua cache
        let il_key = format!("{}:{}", from, input);
        let il = if let Some(cached_il) = self.interlingua_cache.get(&il_key) {
            cached_il.clone()
        } else {
            let new_il = translator.parse(input, from)?;
            self.interlingua_cache.put(il_key, new_il.clone());
            new_il
        };
        
        // Generate
        let output = translator.generate(&il, to)?;
        
        // Cache result
        self.sentence_cache.put(cache_key, output.clone());
        
        Ok(output)
    }
}
```

**Expected improvement:** 2-5x speedup for repeated sentences.

### 5. Memory Optimization

**Problem:** Full system uses ~50MB.

**Solution:** Lazy loading and memory-mapped files.

```rust
pub struct LazyLexicon {
    // Memory-mapped file
    mmap: Mmap,
    
    // Index for fast lookup (loaded on demand)
    index: Option<HashMap<String, usize>>,
}

impl LazyLexicon {
    pub fn new(path: &Path) -> Result<Self, io::Error> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        
        Ok(Self {
            mmap,
            index: None,
        })
    }
    
    pub fn lookup(&mut self, lemma: &str) -> Option<&LexEntry> {
        // Build index on first access
        if self.index.is_none() {
            self.build_index();
        }
        
        // Use index for fast lookup
        let offset = self.index.as_ref()?.get(lemma)?;
        
        // Deserialize entry from mmap
        let entry: LexEntry = bincode::deserialize(&self.mmap[*offset..])?;
        Some(Box::leak(Box::new(entry)))
    }
}
```

**Expected improvement:** 50-70% memory reduction.

## Benchmarking

### Benchmark Suite

```rust
#[cfg(test)]
mod benchmarks {
    use super::*;
    use criterion::{black_box, criterion_group, criterion_main, Criterion};
    
    fn bench_tokenization(c: &mut Criterion) {
        let parser = PolishParser::new();
        let input = "Tomek dał jabłko Izie wczoraj";
        
        c.bench_function("tokenize_simple", |b| {
            b.iter(|| parser.tokenize(black_box(input)))
        });
    }
    
    fn bench_morphology(c: &mut Criterion) {
        let morphology = OptimizedMorphology::load("data/morphology/pl/").unwrap();
        let lemma = "jabłko";
        let features = FeatureBundle {
            case: Some(Case::Accusative),
            number: Some(Number::Singular),
            ..Default::default()
        };
        
        c.bench_function("inflect_noun", |b| {
            b.iter(|| morphology.inflect(black_box(lemma), black_box(&features)))
        });
    }
    
    fn bench_parsing(c: &mut Criterion) {
        let parser = PolishParser::new();
        let input = "Tomek dał jabłko Izie";
        
        c.bench_function("parse_simple", |b| {
            b.iter(|| parser.parse(black_box(input)))
        });
    }
    
    fn bench_deduction(c: &mut Criterion) {
        let parser = PolishParser::new();
        let context = DeductionContext::new();
        let utterance = parser.parse("Tomek dał jabłko Izie").unwrap();
        
        c.bench_function("deduce_simple", |b| {
            b.iter(|| deduction::deduce(black_box(utterance.clone()), &context))
        });
    }
    
    fn bench_generation(c: &mut Criterion) {
        let generator = PolishGenerator::new();
        let il = create_test_interlingua();
        
        c.bench_function("generate_simple", |b| {
            b.iter(|| generator.generate(black_box(&il)))
        });
    }
    
    fn bench_translation(c: &mut Criterion) {
        let translator = UniversalTranslator::new();
        let input = "Tomek dał jabłko Izie";
        
        c.bench_function("translate_pl_to_en", |b| {
            b.iter(|| translator.translate(black_box(input), LanguageId::PL, LanguageId::EN))
        });
    }
    
    criterion_group!(
        benches,
        bench_tokenization,
        bench_morphology,
        bench_parsing,
        bench_deduction,
        bench_generation,
        bench_translation,
    );
    criterion_main!(benches);
}
```

### Expected Benchmark Results

| Benchmark | Target | Actual (estimated) |
|-----------|--------|-------------------|
| Tokenization | < 1ms | ~0.5ms |
| Morphology inflection | < 1ms | ~0.8ms |
| Parsing (simple) | < 10ms | ~8ms |
| Deduction | < 20ms | ~15ms |
| Generation | < 10ms | ~7ms |
| Full translation | < 100ms | ~85ms |

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench translation

# Generate HTML report
cargo bench -- --save-baseline performance

# Compare with baseline
cargo bench -- --baseline performance
```

## Profiling

### CPU Profiling

```bash
# Profile with perf (Linux)
perf record -g cargo run --release -- translate "Tomek dał jabłko Izie" --from pl --to en
perf report

# Profile with Instruments (macOS)
instruments -t "Time Profiler" cargo run --release -- translate "Tomek dał jabłko Izie" --from pl --to en

# Profile with Visual Studio (Windows)
# Use built-in profiler
```

### Memory Profiling

```bash
# Profile with Valgrind (Linux)
valgrind --tool=massif cargo run --release -- translate "Tomek dał jabłko Izie" --from pl --to en
ms_print massif.out.*

# Profile with heaptrack
heaptrack cargo run --release -- translate "Tomek dał jabłko Izie" --from pl --to en
heaptrack_gui heaptrack.*
```

### Common Bottlenecks

**1. Morphology lookup**
- **Symptom:** High CPU usage in `inflect_*` functions
- **Cause:** Scanning all paradigms for each word
- **Solution:** Use indexed lookup with caching (see Optimization #1)

**2. Deduction**
- **Symptom:** O(n²) scaling with sentence length
- **Cause:** Trying all role assignments
- **Solution:** Early termination for explicit cases (see Optimization #3)

**3. Lexicon lookup**
- **Symptom:** Slow concept-to-word mapping
- **Cause:** Linear scan of lexicon
- **Solution:** Use indexed lookup (see Optimization #2)

**4. Memory allocation**
- **Symptom:** High memory usage, frequent GC
- **Cause:** Creating many temporary objects
- **Solution:** Use object pools and arenas

## Performance Testing

### Load Testing

```rust
#[test]
fn test_sustained_throughput() {
    let translator = UniversalTranslator::new();
    let sentences = load_test_sentences(1000);
    
    let start = Instant::now();
    let mut count = 0;
    
    for sentence in &sentences {
        let result = translator.translate(sentence, LanguageId::PL, LanguageId::EN);
        assert!(result.is_ok());
        count += 1;
    }
    
    let elapsed = start.elapsed();
    let throughput = count as f64 / elapsed.as_secs_f64();
    
    println!("Throughput: {:.2} sentences/sec", throughput);
    assert!(throughput > 10.0, "Throughput too low: {:.2}", throughput);
}
```

### Stress Testing

```rust
#[test]
fn test_long_sentences() {
    let translator = UniversalTranslator::new();
    
    // Test with 50-word sentence
    let long_sentence = generate_long_sentence(50);
    let result = translator.translate(&long_sentence, LanguageId::PL, LanguageId::EN);
    
    assert!(result.is_ok());
}

#[test]
fn test_many_nps() {
    let translator = UniversalTranslator::new();
    
    // Test with 10 noun phrases
    let sentence = "Tomek dał jabłko Izie, książkę Markowi, kwiaty Annie, ...";
    let result = translator.translate(sentence, LanguageId::PL, LanguageId::EN);
    
    assert!(result.is_ok());
}
```

## Monitoring in Production

### Metrics to Track

```rust
pub struct PerformanceMetrics {
    // Latency metrics
    pub translation_latency_ms: Histogram,
    pub parsing_latency_ms: Histogram,
    pub deduction_latency_ms: Histogram,
    pub generation_latency_ms: Histogram,
    
    // Throughput metrics
    pub sentences_per_second: Gauge,
    
    // Error metrics
    pub error_rate: Gauge,
    pub error_types: Counter,
    
    // Resource metrics
    pub memory_usage_mb: Gauge,
    pub cpu_usage_percent: Gauge,
    
    // Cache metrics
    pub cache_hit_rate: Gauge,
    pub cache_size: Gauge,
}
```

### Logging Performance Data

```rust
fn log_performance(metrics: &PerformanceMetrics) {
    log::info!(
        "Performance: latency={:.2}ms, throughput={:.2}s/s, memory={}MB, cache_hit={:.2}%",
        metrics.translation_latency_ms.mean(),
        metrics.sentences_per_second.value(),
        metrics.memory_usage_mb.value(),
        metrics.cache_hit_rate.value() * 100.0,
    );
}
```

## Performance Checklist

Before release, verify:

- [ ] All benchmarks pass targets
- [ ] No memory leaks (run for 1 hour)
- [ ] Throughput > 10 sentences/sec sustained
- [ ] Memory usage < 50MB
- [ ] Startup time < 500ms
- [ ] No CPU hotspots in profiler
- [ ] Cache hit rate > 50%
- [ ] Error rate < 1%

## References

- [IMPLEMENTATION_GUIDE.md](./IMPLEMENTATION_GUIDE.md) - Implementation guidance
- [KNOWN_LIMITATIONS.md](./KNOWN_LIMITATIONS.md) - Known performance limitations
- [IMPLEMENTATION_ROADMAP.md](./IMPLEMENTATION_ROADMAP.md) - Performance milestones
- [TEST_STRATEGY.md](./TEST_STRATEGY.md) - Performance testing
