use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use rand::prelude::*;
use satch::is_match;

/// Generate test file paths for benchmarking
fn generate_test_paths(count: usize) -> Vec<String> {
    let mut rng = thread_rng();
    let mut paths = Vec::with_capacity(count);
    
    let extensions = ["js", "ts", "rs", "txt", "json", "md", "css", "html"];
    let dirs = ["src", "lib", "test", "docs", "examples", "assets", "components"];
    let files = ["main", "index", "utils", "helper", "config", "types", "data"];
    
    for _ in 0..count {
        let depth = rng.gen_range(1..=5);
        let mut path_parts = Vec::new();
        
        // Add random directories
        for _ in 0..depth - 1 {
            path_parts.push(dirs[rng.gen_range(0..dirs.len())].to_string());
        }
        
        // Add filename
        let file = files[rng.gen_range(0..files.len())];
        let ext = extensions[rng.gen_range(0..extensions.len())];
        path_parts.push(format!("{}.{}", file, ext));
        
        paths.push(path_parts.join("/"));
    }
    
    paths
}

/// Generate specific test cases for different pattern types
fn generate_specific_test_cases() -> Vec<(String, Vec<String>)> {
    vec![
        // Basic patterns
        ("*.js".to_string(), vec![
            "main.js".to_string(),
            "index.js".to_string(),
            "src/main.js".to_string(),
            "test.ts".to_string(),
            "file.txt".to_string(),
        ]),
        
        // Globstar patterns
        ("**/*.js".to_string(), vec![
            "main.js".to_string(),
            "src/main.js".to_string(),
            "src/lib/utils.js".to_string(),
            "deep/nested/path/file.js".to_string(),
            "main.ts".to_string(),
        ]),
        
        // Complex globstar
        ("**/test/**/*.js".to_string(), vec![
            "test/main.js".to_string(),
            "src/test/unit/helper.js".to_string(),
            "lib/test/integration/api.js".to_string(),
            "src/main.js".to_string(),
            "test.js".to_string(),
        ]),
        
        // Character classes
        ("test[0-9].js".to_string(), vec![
            "test1.js".to_string(),
            "test5.js".to_string(),
            "test9.js".to_string(),
            "testa.js".to_string(),
            "test10.js".to_string(),
        ]),
        
        // Range patterns
        ("[a-z]*.txt".to_string(), vec![
            "readme.txt".to_string(),
            "file.txt".to_string(),
            "data.txt".to_string(),
            "README.txt".to_string(),
            "123.txt".to_string(),
        ]),
        
        // Complex nested patterns
        ("src/**/*.{js,ts,rs}".to_string(), vec![
            "src/main.js".to_string(),
            "src/types.ts".to_string(),
            "src/lib/utils.rs".to_string(),
            "src/deep/nested/file.js".to_string(),
            "src/main.txt".to_string(),
        ]),
    ]
}

fn bench_basic_patterns(c: &mut Criterion) {
    let test_cases = generate_specific_test_cases();
    
    let mut group = c.benchmark_group("basic_patterns");
    
    for (pattern, paths) in test_cases {
        group.bench_with_input(
            BenchmarkId::new("satch", &pattern),
            &(pattern.clone(), paths.clone()),
            |b, (pattern, paths)| {
                b.iter(|| {
                    for path in paths {
                        black_box(is_match(black_box(path), black_box(pattern)));
                    }
                });
            },
        );
    }
    
    group.finish();
}

fn bench_large_dataset(c: &mut Criterion) {
    let sizes = [100, 1000, 5000, 10000];
    let patterns = ["*.js", "**/*.js", "**/test/**/*.js", "[a-z]*.txt"];
    
    let mut group = c.benchmark_group("large_dataset");
    
    for &size in &sizes {
        let paths = generate_test_paths(size);
        
        for &pattern in &patterns {
            group.bench_with_input(
                BenchmarkId::new(format!("satch_{}paths", size), pattern),
                &(pattern, &paths),
                |b, (pattern, paths)| {
                    b.iter_batched(
                        || paths.clone(),
                        |paths| {
                            for path in paths {
                                black_box(is_match(black_box(&path), black_box(pattern)));
                            }
                        },
                        BatchSize::SmallInput,
                    );
                },
            );
        }
    }
    
    group.finish();
}

fn bench_complex_patterns(c: &mut Criterion) {
    let complex_patterns = vec![
        ("**/node_modules/**/*.js", vec![
            "node_modules/package/index.js",
            "src/node_modules/lib/util.js",
            "deep/node_modules/test/spec.js",
            "node_modules/package/lib/deep/file.js",
        ]),
        ("src/**/test/**/*.{spec,test}.{js,ts}", vec![
            "src/components/test/button.spec.js",
            "src/lib/test/utils.test.ts",
            "src/deep/nested/test/integration.spec.js",
            "src/main.js",
        ]),
        ("**/{test,spec,__tests__}/**/*.{js,ts,jsx,tsx}", vec![
            "test/unit/helper.js",
            "spec/integration/api.ts",
            "__tests__/components/button.jsx",
            "src/test/utils.tsx",
            "main.js",
        ]),
    ];
    
    let mut group = c.benchmark_group("complex_patterns");
    
    for (pattern, test_paths) in complex_patterns {
        // Expand test paths to larger set
        let mut paths = Vec::new();
        for _ in 0..1000 {
            paths.extend(test_paths.iter().map(|s| s.to_string()));
        }
        
        group.bench_with_input(
            BenchmarkId::new("satch_complex", pattern),
            &(pattern, paths),
            |b, (pattern, paths)| {
                b.iter(|| {
                    for path in paths {
                        black_box(is_match(black_box(path), black_box(pattern)));
                    }
                });
            },
        );
    }
    
    group.finish();
}

fn bench_memory_intensive(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_intensive");
    
    // Test with very long paths and complex patterns
    let long_paths: Vec<String> = (0..100).map(|i| {
        format!("very/deep/nested/directory/structure/with/many/levels/and/subdirectories/level{}/sublevel/file{}.js", i, i)
    }).collect();
    
    let complex_pattern = "**/deep/**/structure/**/many/**/level*/**/file*.js";
    
    group.bench_function("satch_long_paths", |b| {
        b.iter(|| {
            for path in &long_paths {
                black_box(is_match(black_box(path), black_box(complex_pattern)));
            }
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_basic_patterns,
    bench_large_dataset,
    bench_complex_patterns,
    bench_memory_intensive
);
criterion_main!(benches);