//! BM25 Benchmark
//!
//! Compares performance of different BM25 processing strategies:
//! - Sequential vs Parallel processing
//! - Auto-selection vs Manual selection
//! - With vs Without IDF cache
//!
//! Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use thainlp_rs::search::BM25;

/// Convert owned strings to string slices.
fn as_refs(strings: &[String]) -> Vec<&str> {
    strings.iter().map(String::as_str).collect()
}

/// Generate sample documents for benchmarking.
fn generate_docs(count: usize) -> Vec<String> {
    let base_words = [
        "the",
        "quick",
        "brown",
        "fox",
        "jumps",
        "over",
        "lazy",
        "dog",
        "hello",
        "world",
        "rust",
        "programming",
        "language",
        "fast",
        "safe",
        "memory",
        "system",
        "performance",
        "benchmark",
        "test",
        "data",
        "search",
        "index",
        "document",
        "query",
        "score",
        "ranking",
        "sparse",
        "embedding",
        "vector",
        "machine",
        "learning",
        "natural",
        "processing",
    ];

    (0..count)
        .map(|i| {
            let word_count = 10 + (i % 41);
            (0..word_count)
                .map(|j| base_words[(i + j) % base_words.len()])
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect()
}

/// Generate query strings.
fn generate_queries(count: usize) -> Vec<String> {
    let queries = [
        "quick brown fox",
        "rust programming",
        "machine learning",
        "search index",
        "natural language processing",
        "fast memory safe",
        "benchmark test",
        "sparse embedding vector",
    ];

    (0..count)
        .map(|i| queries[i % queries.len()].to_string())
        .collect()
}

/// Benchmark fit strategies (seq vs par vs auto) for given sizes.
fn bench_fit_for_sizes(
    group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>,
    sizes: &[usize],
) {
    for &size in sizes {
        let docs = generate_docs(size);
        let doc_refs = as_refs(&docs);

        group.bench_with_input(BenchmarkId::new("seq", size), &size, |b, _| {
            b.iter(|| {
                let mut bm25 = BM25::new();
                bm25.fit_batch_seq(black_box(&doc_refs));
                bm25
            })
        });

        group.bench_with_input(BenchmarkId::new("par", size), &size, |b, _| {
            b.iter(|| {
                let mut bm25 = BM25::new();
                bm25.fit_batch_par(black_box(&doc_refs));
                bm25
            })
        });

        group.bench_with_input(BenchmarkId::new("auto", size), &size, |b, _| {
            b.iter(|| {
                let mut bm25 = BM25::new();
                bm25.fit_batch(black_box(&doc_refs));
                bm25
            })
        });
    }
}

/// Benchmark: fit_batch_seq vs fit_batch_par vs fit_batch (auto)
fn bench_fit_strategies(c: &mut Criterion) {
    let mut group = c.benchmark_group("fit_strategies");
    bench_fit_for_sizes(&mut group, &[100, 300, 500, 1000, 2000]);
    group.finish();
}

/// Benchmark: embed_batch_seq vs embed_batch_par vs embed_batch (auto)
fn bench_embed_strategies(c: &mut Criterion) {
    let mut group = c.benchmark_group("embed_strategies");

    let corpus = generate_docs(1000);
    let mut bm25 = BM25::new();
    bm25.fit_batch(&as_refs(&corpus));
    bm25.build_cache();

    for &size in &[100, 500, 1000, 5000] {
        let queries = generate_queries(size);
        let query_refs = as_refs(&queries);

        group.bench_with_input(BenchmarkId::new("seq", size), &size, |b, _| {
            b.iter(|| bm25.embed_batch_seq(black_box(&query_refs)))
        });

        group.bench_with_input(BenchmarkId::new("par", size), &size, |b, _| {
            b.iter(|| bm25.embed_batch_par(black_box(&query_refs)))
        });

        group.bench_with_input(BenchmarkId::new("auto", size), &size, |b, _| {
            b.iter(|| bm25.embed_batch(black_box(&query_refs)))
        });
    }

    group.finish();
}

/// Benchmark: With vs Without IDF cache
fn bench_cache_impact(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_impact");

    let corpus = generate_docs(1000);
    let corpus_refs = as_refs(&corpus);
    let queries = generate_queries(100);
    let query_refs = as_refs(&queries);

    let mut bm25_no_cache = BM25::new();
    bm25_no_cache.fit_batch(&corpus_refs);

    let mut bm25_cached = BM25::new();
    bm25_cached.fit_batch(&corpus_refs);
    bm25_cached.build_cache();

    group.bench_function("without_cache", |b| {
        b.iter(|| bm25_no_cache.embed_batch_seq(black_box(&query_refs)))
    });

    group.bench_function("with_cache", |b| {
        b.iter(|| bm25_cached.embed_batch_seq(black_box(&query_refs)))
    });

    group.finish();
}

/// Benchmark: Corpus size scaling
fn bench_corpus_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("corpus_scaling");
    group.sample_size(50);

    let queries = generate_queries(100);
    let query_refs = as_refs(&queries);

    for &size in &[100, 500, 1000, 5000, 10000] {
        let corpus = generate_docs(size);
        let mut bm25 = BM25::new();
        bm25.fit_batch(&as_refs(&corpus));
        bm25.build_cache();

        group.bench_with_input(
            BenchmarkId::new("embed_100_queries", size),
            &size,
            |b, _| b.iter(|| bm25.embed_batch_seq(black_box(&query_refs))),
        );
    }

    group.finish();
}

/// Benchmark: BM25 vs BM25+
fn bench_bm25_variants(c: &mut Criterion) {
    let mut group = c.benchmark_group("bm25_variants");

    let corpus = generate_docs(500);
    let corpus_refs = as_refs(&corpus);
    let queries = generate_queries(100);
    let query_refs = as_refs(&queries);

    let mut bm25_std = BM25::new().delta(0.0);
    bm25_std.fit_batch(&corpus_refs);
    bm25_std.build_cache();

    let mut bm25_plus = BM25::new().delta(1.0);
    bm25_plus.fit_batch(&corpus_refs);
    bm25_plus.build_cache();

    group.bench_function("bm25_standard", |b| {
        b.iter(|| bm25_std.embed_batch_seq(black_box(&query_refs)))
    });

    group.bench_function("bm25_plus", |b| {
        b.iter(|| bm25_plus.embed_batch_seq(black_box(&query_refs)))
    });

    group.finish();
}

/// Benchmark: Dot product
fn bench_dot_product(c: &mut Criterion) {
    let mut group = c.benchmark_group("dot_product");

    let corpus = generate_docs(500);
    let mut bm25 = BM25::new();
    bm25.fit_batch(&as_refs(&corpus));
    bm25.build_cache();

    let doc_embedding = bm25.embed("quick brown fox jumps over lazy dog");
    let query_embedding = bm25.embed("quick fox");

    group.bench_function("single", |b| {
        b.iter(|| doc_embedding.dot(black_box(&query_embedding)))
    });

    let doc_embeddings: Vec<_> = corpus[..100].iter().map(|d| bm25.embed(d)).collect();

    group.bench_function("batch_100", |b| {
        b.iter(|| {
            doc_embeddings
                .iter()
                .map(|d| d.dot(black_box(&query_embedding)))
                .collect::<Vec<f32>>()
        })
    });

    group.finish();
}

/// Benchmark: Find crossover point for parallel (tests around the threshold)
fn bench_parallel_crossover(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_crossover_fit");
    group.sample_size(30);

    for &size in &[200, 300, 400, 500, 600, 700, 800] {
        let docs = generate_docs(size);
        let doc_refs = as_refs(&docs);

        group.bench_with_input(BenchmarkId::new("seq", size), &size, |b, _| {
            b.iter(|| {
                let mut bm25 = BM25::new();
                bm25.fit_batch_seq(black_box(&doc_refs));
                bm25
            })
        });

        group.bench_with_input(BenchmarkId::new("par", size), &size, |b, _| {
            b.iter(|| {
                let mut bm25 = BM25::new();
                bm25.fit_batch_par(black_box(&doc_refs));
                bm25
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_fit_strategies,
    bench_embed_strategies,
    bench_cache_impact,
    bench_corpus_scaling,
    bench_bm25_variants,
    bench_dot_product,
    bench_parallel_crossover,
);

criterion_main!(benches);
