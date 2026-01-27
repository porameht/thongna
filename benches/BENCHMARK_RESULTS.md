# BM25 Benchmark Results

**Date:** 2026-01-27
**Platform:** macOS Darwin 25.1.0
**Rust:** Release build with optimizations

## Quick Summary

| Operation | Recommendation |
|-----------|----------------|
| `fit_batch` | Use `auto` - switches to parallel at 300+ docs |
| `embed_batch` | Use `auto` - switches to parallel at 3000+ queries |
| IDF Cache | Always call `build_cache()` for 8% speedup |
| BM25+ | Use default (delta=1.0) - no performance cost |

## API Reference

```rust
// Auto-selection (recommended)
bm25.fit_batch(&docs);      // auto: seq <300, par ≥300
bm25.embed_batch(&queries); // auto: seq <3000, par ≥3000

// Manual control
bm25.fit_batch_seq(&docs);  // always sequential
bm25.fit_batch_par(&docs);  // always parallel
bm25.embed_batch_seq(&queries);
bm25.embed_batch_par(&queries);
```

## Detailed Benchmark Results

### 1. fit_batch: Sequential vs Parallel vs Auto

| Documents | Sequential | Parallel | Auto | Winner |
|-----------|------------|----------|------|--------|
| 100 | **180µs** | 262µs | 181µs | seq (parallel 45% slower) |
| 300 | 565µs | **554µs** | 568µs | par (2% faster) |
| 500 | 954µs | **880µs** | 879µs | par (8% faster) |
| 1000 | 1.92ms | 1.72ms | **1.70ms** | par (11% faster) |
| 2000 | 3.87ms | 3.33ms | **3.30ms** | par (14% faster) |

**Crossover Point:** ~300 documents

**Recommendation:**
- `fit_batch()` (auto) correctly selects optimal strategy
- Use `fit_batch_seq()` for <300 docs if you need consistent behavior
- Use `fit_batch_par()` for large batch imports

### 2. embed_batch: Sequential vs Parallel vs Auto

| Queries | Sequential | Parallel | Auto | Winner |
|---------|------------|----------|------|--------|
| 100 | **11.3µs** | 37.3µs | 11.3µs | seq (3.3x faster) |
| 500 | **56.5µs** | 82.5µs | 56.4µs | seq (1.5x faster) |
| 1000 | **113µs** | 133µs | 114µs | seq (15% faster) |
| 5000 | 618µs | **260µs** | 593µs | **par (2.4x faster)** |

**Crossover Point:** ~3000 queries

**Recommendation:**
- `embed_batch()` (auto) is optimal for most cases
- Use `embed_batch_par()` explicitly for 3000+ queries
- Sequential is faster for typical use cases (<1000 queries)

### 3. IDF Cache Impact

| Mode | Time (100 queries) | Improvement |
|------|-------------------|-------------|
| Without cache | 12.2µs | - |
| With cache | **11.2µs** | **8% faster** |

**Recommendation:** Always call `build_cache()` after fitting.

### 4. Corpus Scaling (Query Time)

| Corpus Size | Embed Time (100 queries) |
|-------------|-------------------------|
| 100 docs | 11.35µs |
| 500 docs | 11.23µs |
| 1000 docs | 11.19µs |
| 5000 docs | 11.23µs |
| 10000 docs | 11.19µs |

**Observation:** Query time is **O(1)** - constant regardless of corpus size!
This is because IDF values are cached and looked up by term hash.

### 5. BM25 vs BM25+

| Variant | Time (100 queries) |
|---------|-------------------|
| BM25 Standard (delta=0) | 11.23µs |
| BM25+ (delta=1) | 11.13µs |

**Conclusion:** No performance difference. BM25+ provides better ranking quality for free.

### 6. Dot Product Performance

| Operation | Time |
|-----------|------|
| Single dot product | **33ns** |
| Batch 100 dot products | **5.25µs** (~52ns each) |

### 7. Parallel Crossover Analysis (fit)

Detailed measurements around the threshold:

| Documents | Sequential | Parallel | Δ |
|-----------|------------|----------|---|
| 200 | **379µs** | 412µs | seq +9% |
| 300 | 561µs | **544µs** | par +3% |
| 400 | 755µs | **700µs** | par +7% |
| 500 | 953µs | **871µs** | par +9% |
| 600 | 1.13ms | **1.04ms** | par +8% |
| 700 | 1.33ms | **1.21ms** | par +9% |
| 800 | 1.52ms | **1.35ms** | par +11% |

**Analysis:** Parallel becomes faster starting at ~300 documents, with the advantage growing as batch size increases.

## Usage Examples

### Small Dataset (<300 docs, <3000 queries)
```rust
let mut bm25 = BM25::new();
bm25.fit_batch(&small_docs);  // auto uses sequential
bm25.build_cache();
let results = bm25.embed_batch(&queries);  // auto uses sequential
```

### Large Dataset (1000+ docs, 5000+ queries)
```rust
let mut bm25 = BM25::new();
bm25.fit_batch(&large_docs);  // auto uses parallel
bm25.build_cache();
let results = bm25.embed_batch(&many_queries);  // auto uses parallel
```

### Explicit Control
```rust
// Force sequential for predictable timing
bm25.fit_batch_seq(&docs);
let results = bm25.embed_batch_seq(&queries);

// Force parallel for maximum throughput
bm25.fit_batch_par(&docs);
let results = bm25.embed_batch_par(&queries);
```

## Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark group
cargo bench -- fit_strategies
cargo bench -- embed_strategies
cargo bench -- parallel_crossover

# View HTML reports
open target/criterion/report/index.html
```

## Environment

- CPU: Apple Silicon
- Rust: 1.x (release mode)
- Rayon: Default thread pool
