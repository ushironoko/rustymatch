const crypto = require('crypto');

/**
 * Generate test file paths for benchmarking
 */
function generateTestPaths(count) {
    const paths = [];
    const extensions = ['js', 'ts', 'rs', 'txt', 'json', 'md', 'css', 'html'];
    const dirs = ['src', 'lib', 'test', 'docs', 'examples', 'assets', 'components'];
    const files = ['main', 'index', 'utils', 'helper', 'config', 'types', 'data'];
    
    for (let i = 0; i < count; i++) {
        const depth = Math.floor(Math.random() * 5) + 1;
        const pathParts = [];
        
        // Add random directories
        for (let j = 0; j < depth - 1; j++) {
            pathParts.push(dirs[Math.floor(Math.random() * dirs.length)]);
        }
        
        // Add filename
        const file = files[Math.floor(Math.random() * files.length)];
        const ext = extensions[Math.floor(Math.random() * extensions.length)];
        pathParts.push(`${file}.${ext}`);
        
        paths.push(pathParts.join('/'));
    }
    
    return paths;
}

/**
 * Generate specific test cases for different pattern types
 */
function generateSpecificTestCases() {
    return [
        // Basic patterns
        ['*.js', [
            'main.js',
            'index.js', 
            'src/main.js',
            'test.ts',
            'file.txt'
        ]],
        
        // Globstar patterns  
        ['**/*.js', [
            'main.js',
            'src/main.js',
            'src/lib/utils.js',
            'deep/nested/path/file.js',
            'main.ts'
        ]],
        
        // Complex globstar
        ['**/test/**/*.js', [
            'test/main.js',
            'src/test/unit/helper.js',
            'lib/test/integration/api.js',
            'src/main.js',
            'test.js'
        ]],
        
        // Character classes
        ['test[0-9].js', [
            'test1.js',
            'test5.js', 
            'test9.js',
            'testa.js',
            'test10.js'
        ]],
        
        // Range patterns
        ['[a-z]*.txt', [
            'readme.txt',
            'file.txt',
            'data.txt', 
            'README.txt',
            '123.txt'
        ]]
    ];
}

/**
 * Generate complex patterns for stress testing
 */
function generateComplexPatterns() {
    return [
        ['**/node_modules/**/*.js', [
            'node_modules/package/index.js',
            'src/node_modules/lib/util.js', 
            'deep/node_modules/test/spec.js',
            'node_modules/package/lib/deep/file.js'
        ]],
        
        ['src/**/test/**/*.{spec,test}.{js,ts}', [
            'src/components/test/button.spec.js',
            'src/lib/test/utils.test.ts',
            'src/deep/nested/test/integration.spec.js',
            'src/main.js'
        ]],
        
        ['**/{test,spec,__tests__}/**/*.{js,ts,jsx,tsx}', [
            'test/unit/helper.js',
            'spec/integration/api.ts',
            '__tests__/components/button.jsx', 
            'src/test/utils.tsx',
            'main.js'
        ]]
    ];
}

module.exports = {
    generateTestPaths,
    generateSpecificTestCases,
    generateComplexPatterns
};