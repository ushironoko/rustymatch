const Benchmark = require('benchmark');
const micromatch = require('micromatch');
const picomatch = require('picomatch');
const { generateTestPaths, generateSpecificTestCases, generateComplexPatterns } = require('./testdata');

// Benchmark results storage
const results = {
    basic_patterns: {},
    large_dataset: {},
    complex_patterns: {},
    memory_usage: {}
};

/**
 * Run basic pattern benchmarks
 */
function runBasicPatternBenchmarks() {
    console.log('🚀 Running basic pattern benchmarks...');
    const testCases = generateSpecificTestCases();
    
    return new Promise((resolve) => {
        let completedTests = 0;
        const totalTests = testCases.length;
        
        testCases.forEach(([pattern, paths]) => {
            const suite = new Benchmark.Suite(`Basic Pattern: ${pattern}`);
            
            suite
                .add(`micromatch: ${pattern}`, function() {
                    paths.forEach(path => {
                        micromatch.isMatch(path, pattern);
                    });
                })
                .add(`picomatch: ${pattern}`, function() {
                    const matcher = picomatch(pattern);
                    paths.forEach(path => {
                        matcher(path);
                    });
                })
                .on('cycle', function(event) {
                    console.log(String(event.target));
                })
                .on('complete', function() {
                    const fastest = this.filter('fastest')[0];
                    const slowest = this.filter('slowest')[0];
                    
                    results.basic_patterns[pattern] = {
                        fastest: {
                            name: fastest.name,
                            hz: fastest.hz,
                            mean: fastest.stats.mean,
                            rme: fastest.stats.rme
                        },
                        slowest: {
                            name: slowest.name,
                            hz: slowest.hz,
                            mean: slowest.stats.mean,
                            rme: slowest.stats.rme
                        },
                        speedup: fastest.hz / slowest.hz
                    };
                    
                    console.log(`Fastest is ${fastest.name}\n`);
                    
                    completedTests++;
                    if (completedTests === totalTests) {
                        resolve();
                    }
                })
                .run({ async: true });
        });
    });
}

/**
 * Run large dataset benchmarks
 */
function runLargeDatasetBenchmarks() {
    console.log('📊 Running large dataset benchmarks...');
    const sizes = [100, 1000, 5000];
    const patterns = ['*.js', '**/*.js', '**/test/**/*.js'];
    
    return new Promise((resolve) => {
        let completedTests = 0;
        const totalTests = sizes.length * patterns.length;
        
        sizes.forEach(size => {
            const paths = generateTestPaths(size);
            
            patterns.forEach(pattern => {
                const suite = new Benchmark.Suite(`Large Dataset: ${size} paths, ${pattern}`);
                
                suite
                    .add(`micromatch: ${size} paths, ${pattern}`, function() {
                        paths.forEach(path => {
                            micromatch.isMatch(path, pattern);
                        });
                    })
                    .add(`picomatch: ${size} paths, ${pattern}`, function() {
                        const matcher = picomatch(pattern);
                        paths.forEach(path => {
                            matcher(path);
                        });
                    })
                    .on('cycle', function(event) {
                        console.log(String(event.target));
                    })
                    .on('complete', function() {
                        const fastest = this.filter('fastest')[0];
                        const slowest = this.filter('slowest')[0];
                        
                        const key = `${size}_paths_${pattern}`;
                        results.large_dataset[key] = {
                            fastest: {
                                name: fastest.name,
                                hz: fastest.hz,
                                mean: fastest.stats.mean,
                                rme: fastest.stats.rme
                            },
                            slowest: {
                                name: slowest.name, 
                                hz: slowest.hz,
                                mean: slowest.stats.mean,
                                rme: slowest.stats.rme
                            },
                            speedup: fastest.hz / slowest.hz
                        };
                        
                        console.log(`Fastest is ${fastest.name}\n`);
                        
                        completedTests++;
                        if (completedTests === totalTests) {
                            resolve();
                        }
                    })
                    .run({ async: true });
            });
        });
    });
}

/**
 * Run complex pattern benchmarks
 */
function runComplexPatternBenchmarks() {
    console.log('🔥 Running complex pattern benchmarks...');
    const complexPatterns = generateComplexPatterns();
    
    return new Promise((resolve) => {
        let completedTests = 0;
        const totalTests = complexPatterns.length;
        
        complexPatterns.forEach(([pattern, testPaths]) => {
            // Expand test paths to larger set for meaningful benchmarking
            const paths = [];
            for (let i = 0; i < 1000; i++) {
                paths.push(...testPaths);
            }
            
            const suite = new Benchmark.Suite(`Complex Pattern: ${pattern}`);
            
            suite
                .add(`micromatch: ${pattern}`, function() {
                    paths.forEach(path => {
                        micromatch.isMatch(path, pattern);
                    });
                })
                .add(`picomatch: ${pattern}`, function() {
                    const matcher = picomatch(pattern);
                    paths.forEach(path => {
                        matcher(path);
                    });
                })
                .on('cycle', function(event) {
                    console.log(String(event.target));
                })
                .on('complete', function() {
                    const fastest = this.filter('fastest')[0];
                    const slowest = this.filter('slowest')[0];
                    
                    results.complex_patterns[pattern] = {
                        fastest: {
                            name: fastest.name,
                            hz: fastest.hz,
                            mean: fastest.stats.mean,
                            rme: fastest.stats.rme
                        },
                        slowest: {
                            name: slowest.name,
                            hz: slowest.hz,
                            mean: slowest.stats.mean,
                            rme: slowest.stats.rme
                        },
                        speedup: fastest.hz / slowest.hz
                    };
                    
                    console.log(`Fastest is ${fastest.name}\n`);
                    
                    completedTests++;
                    if (completedTests === totalTests) {
                        resolve();
                    }
                })
                .run({ async: true });
        });
    });
}

/**
 * Measure memory usage
 */
function measureMemoryUsage() {
    console.log('💾 Measuring memory usage...');
    
    const paths = generateTestPaths(10000);
    const pattern = '**/*.js';
    
    // Measure micromatch memory usage
    const micromatchBefore = process.memoryUsage();
    for (let i = 0; i < 100; i++) {
        paths.forEach(path => {
            micromatch.isMatch(path, pattern);
        });
    }
    if (global.gc) global.gc();
    const micromatchAfter = process.memoryUsage();
    
    // Measure picomatch memory usage
    const picomatchBefore = process.memoryUsage();
    const matcher = picomatch(pattern);
    for (let i = 0; i < 100; i++) {
        paths.forEach(path => {
            matcher(path);
        });
    }
    if (global.gc) global.gc();
    const picomatchAfter = process.memoryUsage();
    
    results.memory_usage = {
        micromatch: {
            heapUsed: micromatchAfter.heapUsed - micromatchBefore.heapUsed,
            heapTotal: micromatchAfter.heapTotal - micromatchBefore.heapTotal,
            rss: micromatchAfter.rss - micromatchBefore.rss
        },
        picomatch: {
            heapUsed: picomatchAfter.heapUsed - picomatchBefore.heapUsed,
            heapTotal: picomatchAfter.heapTotal - picomatchBefore.heapTotal,
            rss: picomatchAfter.rss - picomatchBefore.rss
        }
    };
    
    console.log('Memory usage comparison:');
    console.log(`Micromatch heap used: ${(results.memory_usage.micromatch.heapUsed / 1024 / 1024).toFixed(2)} MB`);
    console.log(`Picomatch heap used: ${(results.memory_usage.picomatch.heapUsed / 1024 / 1024).toFixed(2)} MB`);
}

/**
 * Save results to file
 */
function saveResults() {
    const fs = require('fs');
    const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
    const filename = `../results/js-benchmark-results-${timestamp}.json`;
    
    fs.writeFileSync(filename, JSON.stringify(results, null, 2));
    console.log(`\n📁 Results saved to ${filename}`);
}

/**
 * Main benchmark runner
 */
async function main() {
    console.log('🏁 Starting JavaScript benchmark suite...\n');
    
    try {
        await runBasicPatternBenchmarks();
        await runLargeDatasetBenchmarks();
        await runComplexPatternBenchmarks();
        measureMemoryUsage();
        
        console.log('\n✅ All benchmarks completed!');
        saveResults();
        
    } catch (error) {
        console.error('❌ Benchmark failed:', error);
        process.exit(1);
    }
}

if (require.main === module) {
    main();
}