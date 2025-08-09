const Benchmark = require('benchmark');
const micromatch = require('micromatch');
const picomatch = require('picomatch');
const { generateTestPaths, generateSpecificTestCases, generateComplexPatterns } = require('./testdata');

// JIT warmup control
function warmupPhase(fn, iterations = 1000) {
    console.log(`🔥 JIT warmup phase (${iterations} iterations)...`);
    const start = process.hrtime.bigint();
    
    for (let i = 0; i < iterations; i++) {
        fn();
    }
    
    const end = process.hrtime.bigint();
    const warmupTime = Number(end - start) / 1000000; // Convert to milliseconds
    console.log(`   Warmup completed in ${warmupTime.toFixed(2)}ms`);
    
    return warmupTime;
}

// GC control wrapper
function runWithGCControl(testFn, label = 'test') {
    // Force GC before test
    if (global.gc) {
        global.gc();
        console.log(`   Pre-${label} GC completed`);
    }
    
    const memBefore = process.memoryUsage();
    const result = testFn();
    const memAfter = process.memoryUsage();
    
    // Force GC after test
    if (global.gc) {
        global.gc();
        console.log(`   Post-${label} GC completed`);
    }
    
    const memFinal = process.memoryUsage();
    
    return {
        result,
        memory: {
            before: memBefore,
            after: memAfter,
            final: memFinal,
            netIncrease: memFinal.heapUsed - memBefore.heapUsed
        }
    };
}

// Pattern compilation timing
function measurePatternCompilation(patterns) {
    const results = {
        micromatch: {},
        picomatch: {}
    };
    
    console.log('📊 Measuring pattern compilation times...');
    
    for (const pattern of patterns) {
        // Micromatch compilation (immediate compilation)
        const micromatchStart = process.hrtime.bigint();
        for (let i = 0; i < 100; i++) {
            // Micromatch compiles on each call, so we measure actual usage
            micromatch.makeRe(pattern);
        }
        const micromatchEnd = process.hrtime.bigint();
        
        // Picomatch compilation
        const picomatchStart = process.hrtime.bigint();
        for (let i = 0; i < 100; i++) {
            picomatch(pattern);
        }
        const picomatchEnd = process.hrtime.bigint();
        
        results.micromatch[pattern] = Number(micromatchEnd - micromatchStart) / 1000000 / 100;
        results.picomatch[pattern] = Number(picomatchEnd - picomatchStart) / 1000000 / 100;
        
        console.log(`   ${pattern}:`);
        console.log(`     Micromatch: ${results.micromatch[pattern].toFixed(3)}ms`);
        console.log(`     Picomatch:  ${results.picomatch[pattern].toFixed(3)}ms`);
    }
    
    return results;
}

// Enhanced memory measurement
function measureMemoryUsage(testFn, iterations = 100) {
    const measurements = [];
    
    console.log('💾 Enhanced memory measurement...');
    
    // Initial GC
    if (global.gc) global.gc();
    const baseline = process.memoryUsage();
    
    for (let i = 0; i < iterations; i++) {
        const beforeMem = process.memoryUsage();
        testFn();
        const afterMem = process.memoryUsage();
        
        measurements.push({
            iteration: i,
            heapUsedDelta: afterMem.heapUsed - beforeMem.heapUsed,
            heapTotalDelta: afterMem.heapTotal - beforeMem.heapTotal,
            rss: afterMem.rss
        });
        
        // Periodic GC to measure actual retention
        if (i % 10 === 0 && global.gc) {
            global.gc();
        }
    }
    
    // Final GC and measurement
    if (global.gc) global.gc();
    const final = process.memoryUsage();
    
    return {
        baseline,
        final,
        netHeapIncrease: final.heapUsed - baseline.heapUsed,
        measurements,
        avgHeapDeltaPerOp: measurements.reduce((sum, m) => sum + m.heapUsedDelta, 0) / measurements.length,
        maxHeapDelta: Math.max(...measurements.map(m => m.heapUsedDelta)),
        minHeapDelta: Math.min(...measurements.map(m => m.heapUsedDelta))
    };
}

// Improved benchmark suite with JIT/GC control
class ImprovedBenchmarkSuite {
    constructor() {
        this.results = {
            compilation: {},
            performance: {},
            memory: {},
            warmup: {}
        };
    }
    
    async runCompilationBenchmarks() {
        console.log('\n🔧 === Pattern Compilation Analysis ===');
        
        const testPatterns = [
            '*.js',
            '**/*.js', 
            '**/test/**/*.js',
            'test[0-9].js',
            '[a-z]*.txt',
            '**/node_modules/**/*.js',
            'src/**/test/**/*.{spec,test}.{js,ts}'
        ];
        
        this.results.compilation = measurePatternCompilation(testPatterns);
    }
    
    async runWarmupAnalysis() {
        console.log('\n🔥 === JIT Warmup Analysis ===');
        
        const testCases = generateSpecificTestCases();
        const sampleCase = testCases[0];
        const [pattern, paths] = sampleCase;
        
        // Micromatch warmup
        console.log('Micromatch warmup analysis:');
        this.results.warmup.micromatch = warmupPhase(() => {
            paths.forEach(path => micromatch.isMatch(path, pattern));
        }, 2000);
        
        // Picomatch warmup  
        console.log('Picomatch warmup analysis:');
        const matcher = picomatch(pattern);
        this.results.warmup.picomatch = warmupPhase(() => {
            paths.forEach(path => matcher(path));
        }, 2000);
    }
    
    async runControlledPerformanceBenchmarks() {
        console.log('\n⚡ === Controlled Performance Benchmarks ===');
        
        const testCases = generateSpecificTestCases();
        
        for (const [pattern, paths] of testCases) {
            console.log(`\nBenchmarking pattern: ${pattern}`);
            
            await this.runSinglePatternBenchmark(pattern, paths);
        }
    }
    
    async runSinglePatternBenchmark(pattern, paths) {
        return new Promise((resolve) => {
            const suite = new Benchmark.Suite(`Controlled: ${pattern}`, {
                onStart: () => {
                    console.log(`   Starting controlled benchmark for ${pattern}`);
                },
                
                onCycle: (event) => {
                    console.log(`   ${String(event.target)}`);
                },
                
                onComplete: function() {
                    const fastest = this.filter('fastest')[0];
                    const slowest = this.filter('slowest')[0];
                    
                    console.log(`   Fastest: ${fastest.name}`);
                    console.log(`   Speed difference: ${(fastest.hz / slowest.hz).toFixed(2)}x`);
                    
                    resolve({
                        fastest: {
                            name: fastest.name,
                            hz: fastest.hz,
                            stats: fastest.stats
                        },
                        slowest: {
                            name: slowest.name,
                            hz: slowest.hz,
                            stats: slowest.stats
                        }
                    });
                }
            });
            
            // Pre-compile patterns
            const picomatchMatcher = picomatch(pattern);
            
            // Warmup phase
            console.log('   Running warmup...');
            for (let i = 0; i < 500; i++) {
                paths.forEach(path => {
                    micromatch.isMatch(path, pattern);
                    picomatchMatcher(path);
                });
            }
            
            // Force GC before benchmarks
            if (global.gc) global.gc();
            
            suite
                .add(`micromatch: ${pattern}`, {
                    fn: function() {
                        paths.forEach(path => {
                            micromatch.isMatch(path, pattern);
                        });
                    },
                    setup: function() {
                        // Ensure no dead code elimination
                        if (typeof window !== 'undefined') window.benchmarkResult = null;
                    }
                })
                .add(`picomatch: ${pattern}`, {
                    fn: function() {
                        paths.forEach(path => {
                            picomatchMatcher(path);
                        });
                    },
                    setup: function() {
                        if (typeof window !== 'undefined') window.benchmarkResult = null;
                    }
                })
                .run({ async: true });
        });
    }
    
    async runMemoryAnalysis() {
        console.log('\n💾 === Enhanced Memory Analysis ===');
        
        const testPaths = generateTestPaths(5000);
        const pattern = '**/*.js';
        
        // Micromatch memory analysis
        console.log('Micromatch memory analysis:');
        this.results.memory.micromatch = measureMemoryUsage(() => {
            testPaths.forEach(path => {
                micromatch.isMatch(path, pattern);
            });
        });
        
        // Picomatch memory analysis
        console.log('Picomatch memory analysis:');
        const matcher = picomatch(pattern);
        this.results.memory.picomatch = measureMemoryUsage(() => {
            testPaths.forEach(path => {
                matcher(path);
            });
        });
        
        // Memory summary
        console.log('\nMemory Analysis Summary:');
        console.log(`Micromatch net heap increase: ${(this.results.memory.micromatch.netHeapIncrease / 1024 / 1024).toFixed(2)} MB`);
        console.log(`Picomatch net heap increase: ${(this.results.memory.picomatch.netHeapIncrease / 1024 / 1024).toFixed(2)} MB`);
        console.log(`Micromatch avg heap delta per op: ${this.results.memory.micromatch.avgHeapDeltaPerOp.toFixed(2)} bytes`);
        console.log(`Picomatch avg heap delta per op: ${this.results.memory.picomatch.avgHeapDeltaPerOp.toFixed(2)} bytes`);
    }
    
    saveResults() {
        const fs = require('fs');
        const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
        const filename = `../results/improved-benchmark-${timestamp}.json`;
        
        const report = {
            timestamp: new Date().toISOString(),
            nodeVersion: process.version,
            platform: process.platform,
            arch: process.arch,
            memoryAtStart: process.memoryUsage(),
            results: this.results
        };
        
        fs.writeFileSync(filename, JSON.stringify(report, null, 2));
        console.log(`\n📁 Improved benchmark results saved to ${filename}`);
        
        return filename;
    }
}

// Main execution
async function main() {
    console.log('🏁 Starting Improved JavaScript Benchmark Suite...');
    console.log(`Node.js: ${process.version}`);
    console.log(`Platform: ${process.platform} ${process.arch}`);
    console.log(`GC Available: ${global.gc ? 'Yes' : 'No (run with --expose-gc for better results)'}`);
    
    const suite = new ImprovedBenchmarkSuite();
    
    try {
        await suite.runCompilationBenchmarks();
        await suite.runWarmupAnalysis();
        await suite.runControlledPerformanceBenchmarks();
        await suite.runMemoryAnalysis();
        
        const savedFile = suite.saveResults();
        
        console.log('\n✅ Improved benchmark suite completed!');
        console.log('🔬 Key improvements:');
        console.log('  - JIT warmup control');
        console.log('  - GC timing control');
        console.log('  - Pattern compilation separation');
        console.log('  - Enhanced memory analysis');
        
    } catch (error) {
        console.error('❌ Improved benchmark failed:', error);
        process.exit(1);
    }
}

if (require.main === module) {
    main();
}