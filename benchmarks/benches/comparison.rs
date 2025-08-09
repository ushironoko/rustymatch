use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput};
use rand::prelude::*;
use satch::is_match;

/// Generate edge case patterns and paths for testing
fn generate_edge_cases() -> Vec<(String, Vec<String>)> {
    vec![
        // Very long paths
        ("**/*.js".to_string(), vec![
            "a/very/deeply/nested/directory/structure/with/many/many/levels/that/goes/on/and/on/and/continues/for/a/very/long/time/until/finally/reaching/the/file.js".to_string(),
            format!("{}/file.js", "deeply/nested/".repeat(50)),
        ]),
        
        // Patterns with many alternatives
        ("**/*.{js,ts,jsx,tsx,vue,svelte,astro,md,mdx,json,yaml,yml,toml,xml,html,htm,css,scss,sass,less,styl}".to_string(), vec![
            "src/component.jsx".to_string(),
            "docs/readme.md".to_string(),
            "config/settings.toml".to_string(),
            "styles/main.scss".to_string(),
        ]),
        
        // Complex character classes
        ("[a-zA-Z0-9._-]*[!@#$%^&*()+={}\\[\\]:;\"'<>,.?/~`|\\\\]*".to_string(), vec![
            "normal_file.txt".to_string(),
            "file-with-special!@#.txt".to_string(),
            "MixedCase123.txt".to_string(),
        ]),
        
        // Multiple consecutive globstars
        ("**/**/test/**/**/*.spec.js".to_string(), vec![
            "src/components/test/button.spec.js".to_string(),
            "lib/utils/test/helper.spec.js".to_string(),
            "deep/nested/test/integration/api.spec.js".to_string(),
        ]),
        
        // Empty and single character cases
        ("*".to_string(), vec![
            "".to_string(),
            "a".to_string(),
            "ab".to_string(),
        ]),
        
        // Unicode and special characters
        ("**/*日本語*.txt".to_string(), vec![
            "docs/日本語ファイル.txt".to_string(),
            "src/日本語test.txt".to_string(),
            "english.txt".to_string(),
        ]),
        
        // Pathological cases that might cause performance issues
        ("a*a*a*a*a*a*a*a*a*a*".to_string(), vec![
            "aaaaaaaaaa".to_string(),
            "abacadaeafagahaiajakal".to_string(),
            "bbbbbbbbb".to_string(),
        ]),
    ]
}

/// Generate realistic project structures for testing
fn generate_realistic_project_paths(count: usize) -> Vec<String> {
    let mut rng = thread_rng();
    let mut paths = Vec::new();
    
    // Common project structure patterns
    let frameworks = ["react", "vue", "angular", "svelte"];
    let dirs = ["src", "lib", "components", "pages", "utils", "hooks", "stores", "types"];
    let test_dirs = ["__tests__", "test", "spec", "e2e"];
    let file_types = [
        ("ts", 0.3), ("js", 0.25), ("tsx", 0.15), ("jsx", 0.1),
        ("json", 0.05), ("md", 0.03), ("css", 0.05), ("scss", 0.02),
        ("test.ts", 0.03), ("spec.js", 0.02)
    ];
    
    for _ in 0..count {
        let framework = frameworks[rng.gen_range(0..frameworks.len())];
        let depth = rng.gen_range(1..8);
        let mut path_parts = vec![framework];
        
        // Build directory structure
        for _ in 0..depth {
            if rng.gen_bool(0.2) {
                // Add test directory
                path_parts.push(test_dirs[rng.gen_range(0..test_dirs.len())]);
            } else {
                // Add regular directory
                path_parts.push(dirs[rng.gen_range(0..dirs.len())]);
            }
        }
        
        // Add filename
        let mut cumulative = 0.0;
        let rand_val: f64 = rng.gen();
        
        let mut selected_ext = file_types[0].0;
        for (ext, prob) in &file_types {
            cumulative += prob;
            if rand_val <= cumulative {
                selected_ext = ext;
                break;
            }
        }
        
        let filename = format!("component.{}", selected_ext);
        path_parts.push(&filename);
        
        paths.push(path_parts.join("/"));
    }
    
    paths
}

/// Benchmark edge cases
fn bench_edge_cases(c: &mut Criterion) {
    let edge_cases = generate_edge_cases();
    
    let mut group = c.benchmark_group("edge_cases");
    group.sample_size(50); // Fewer samples for potentially slow edge cases
    
    for (pattern, test_paths) in edge_cases {
        group.bench_with_input(
            BenchmarkId::new("satch_edge", &pattern),
            &(pattern.clone(), test_paths.clone()),
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

/// Benchmark with statistical rigor
fn bench_statistical_analysis(c: &mut Criterion) {
    let patterns = ["*.js", "**/*.js", "**/test/**/*.js", "[a-z]*.txt"];
    let path_counts = [100, 500, 1000];
    
    let mut group = c.benchmark_group("statistical_analysis");
    group.sample_size(200); // More samples for better statistics
    group.measurement_time(std::time::Duration::from_secs(10));
    
    for &pattern in &patterns {
        for &count in &path_counts {
            let paths = generate_realistic_project_paths(count);
            
            group.throughput(Throughput::Elements(count as u64));
            group.bench_with_input(
                BenchmarkId::new(format!("satch_rigorous_{}", count), pattern),
                &(pattern, &paths),
                |b, (pattern, paths)| {
                    b.iter_batched(
                        || (*paths).clone(),
                        |paths| {
                            let mut matches = 0u32;
                            for path in paths {
                                if is_match(black_box(&path), black_box(pattern)) {
                                    matches += 1;
                                }
                            }
                            black_box(matches)
                        },
                        BatchSize::SmallInput,
                    );
                },
            );
        }
    }
    
    group.finish();
}

/// Benchmark pattern compilation separately (simulation)
fn bench_pattern_compilation(c: &mut Criterion) {
    let complex_patterns = [
        "*.js",
        "**/*.js",
        "**/test/**/*.js",
        "**/*.{js,ts,jsx,tsx}",
        "**/node_modules/**/*.js",
        "src/**/test/**/*.{spec,test}.{js,ts}",
        "**/{test,spec,__tests__}/**/*.{js,ts,jsx,tsx}",
    ];
    
    let mut group = c.benchmark_group("pattern_compilation");
    
    // Note: Satch doesn't have explicit compilation phase like picomatch,
    // but we can measure the first-time pattern parsing overhead
    for pattern in &complex_patterns {
        group.bench_function(
            BenchmarkId::new("satch_first_parse", pattern),
            |b| {
                b.iter(|| {
                    // Simulate first-time parsing by testing against a simple path
                    black_box(is_match(black_box("test.js"), black_box(pattern)))
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark memory allocation patterns
fn bench_memory_patterns(c: &mut Criterion) {
    let long_paths: Vec<String> = (0..1000).map(|i| {
        format!("very/long/path/with/many/segments/{}/level{}/sublevel/file{}.js", 
                "segment".repeat(10), i, i)
    }).collect();
    
    let mut group = c.benchmark_group("memory_patterns");
    
    let patterns = ["**/*.js", "**/test/**/*.js", "**/*level*/**/*.js"];
    
    for pattern in &patterns {
        group.bench_with_input(
            BenchmarkId::new("satch_memory_intensive", pattern),
            &(pattern, &long_paths),
            |b, (pattern, paths)| {
                b.iter(|| {
                    let mut total_matches = 0u32;
                    for path in paths.iter() {
                        if is_match(black_box(path), black_box(pattern)) {
                            total_matches += 1;
                        }
                    }
                    black_box(total_matches)
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark with different input characteristics
fn bench_input_characteristics(c: &mut Criterion) {
    let mut group = c.benchmark_group("input_characteristics");
    
    // Short paths vs long paths
    let short_paths: Vec<String> = (0..1000).map(|i| format!("f{}.js", i)).collect();
    let long_paths: Vec<String> = (0..1000).map(|i| {
        format!("{}/file{}.js", "very/deeply/nested/directory/structure".repeat(5), i)
    }).collect();
    
    let pattern = "**/*.js";
    
    group.bench_with_input(
        BenchmarkId::new("satch_short_paths", "1000_paths"),
        &(pattern, &short_paths),
        |b, (pattern, paths)| {
            b.iter(|| {
                for path in paths.iter() {
                    black_box(is_match(black_box(path), black_box(pattern)));
                }
            });
        },
    );
    
    group.bench_with_input(
        BenchmarkId::new("satch_long_paths", "1000_paths"),
        &(pattern, &long_paths),
        |b, (pattern, paths)| {
            b.iter(|| {
                for path in paths.iter() {
                    black_box(is_match(black_box(path), black_box(pattern)));
                }
            });
        },
    );
    
    // High match rate vs low match rate
    let high_match_paths: Vec<String> = (0..1000).map(|i| format!("src/file{}.js", i)).collect();
    let low_match_paths: Vec<String> = (0..1000).map(|i| format!("src/file{}.txt", i)).collect();
    
    let js_pattern = "**/*.js";
    
    group.bench_with_input(
        BenchmarkId::new("satch_high_match_rate", "90%_matches"),
        &(js_pattern, &high_match_paths),
        |b, (pattern, paths)| {
            b.iter(|| {
                for path in paths.iter() {
                    black_box(is_match(black_box(path), black_box(pattern)));
                }
            });
        },
    );
    
    group.bench_with_input(
        BenchmarkId::new("satch_low_match_rate", "0%_matches"),
        &(js_pattern, &low_match_paths),
        |b, (pattern, paths)| {
            b.iter(|| {
                for path in paths.iter() {
                    black_box(is_match(black_box(path), black_box(pattern)));
                }
            });
        },
    );
    
    group.finish();
}

criterion_group!(
    improved_benches,
    bench_edge_cases,
    bench_statistical_analysis,
    bench_pattern_compilation,
    bench_memory_patterns,
    bench_input_characteristics
);
criterion_main!(improved_benches);