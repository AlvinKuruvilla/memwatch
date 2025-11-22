#!/usr/bin/env node
/**
 * Test Runner Script for Build Pipeline Example
 *
 * Runs tests with configurable memory allocation to simulate realistic test workloads.
 * This demonstrates how Node.js processes appear in memwatch's process tree.
 *
 * Usage:
 *   node run_tests.js --size small --binary target/debug/build_example
 */

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

// Problem size configurations
const SIZES = {
    small: {
        testDataSize: 10_000,        // 10K items
        iterations: 100,
        description: 'Quick test (~50 MB memory)'
    },
    medium: {
        testDataSize: 100_000,       // 100K items
        iterations: 500,
        description: 'Moderate test (~200 MB memory)'
    },
    large: {
        testDataSize: 500_000,       // 500K items
        iterations: 1000,
        description: 'Stress test (~500 MB memory)'
    }
};

// Parse command line arguments
function parseArgs() {
    const args = process.argv.slice(2);
    const options = {
        size: 'small',
        binary: null
    };

    for (let i = 0; i < args.length; i++) {
        if (args[i] === '--size' && i + 1 < args.length) {
            options.size = args[i + 1];
            i++;
        } else if (args[i] === '--binary' && i + 1 < args.length) {
            options.binary = args[i + 1];
            i++;
        } else if (args[i] === '--help') {
            console.log('Usage: node run_tests.js --size <small|medium|large> --binary <path>');
            process.exit(0);
        }
    }

    if (!SIZES[options.size]) {
        console.error(`Error: Invalid size '${options.size}'. Must be: small, medium, or large`);
        process.exit(1);
    }

    if (!options.binary) {
        console.error('Error: --binary argument is required');
        process.exit(1);
    }

    return options;
}

// Generate test data (allocates memory)
function generateTestData(size) {
    console.log(`  Generating ${size.toLocaleString()} test items...`);

    const data = [];
    for (let i = 0; i < size; i++) {
        data.push({
            id: i,
            value: Math.floor(Math.random() * 1000000),
            timestamp: Date.now(),
            metadata: {
                processed: false,
                checksum: i * 997
            }
        });
    }

    return data;
}

// Process test data (computation + memory)
function processTestData(data, iterations) {
    console.log(`  Processing data (${iterations} iterations)...`);

    let checksum = 0;

    for (let iter = 0; iter < iterations; iter++) {
        // Simulate data processing
        for (let i = 0; i < data.length; i++) {
            data[i].metadata.processed = true;
            checksum += data[i].value;
        }

        // Simulate intermediate allocations
        if (iter % 100 === 0 && iter > 0) {
            const tempBuffer = new Array(1000).fill(0);
            checksum += tempBuffer.length;
        }
    }

    console.log(`  Checksum: ${checksum}`);
    return checksum > 0;
}

// Run binary validation test
function testBinary(binaryPath) {
    console.log(`  Testing binary: ${path.basename(binaryPath)}`);

    if (!fs.existsSync(binaryPath)) {
        console.error(`  Error: Binary not found: ${binaryPath}`);
        return false;
    }

    try {
        const output = execSync(binaryPath, { encoding: 'utf-8', timeout: 30000 });
        console.log(`  Binary executed successfully`);

        // Check for expected output
        if (output.includes('All modules processed successfully!')) {
            console.log(`  ✓ Binary output validated`);
            return true;
        } else {
            console.log(`  ⚠ Binary output unexpected`);
            return false;
        }
    } catch (error) {
        console.error(`  Error running binary: ${error.message}`);
        return false;
    }
}

// Memory-intensive string operations test
function testStringOperations(size) {
    console.log(`  Testing string operations...`);

    const strings = [];
    for (let i = 0; i < size / 10; i++) {
        strings.push('test-string-' + i.toString().padStart(10, '0') + '-data');
    }

    // String concatenation (allocates memory)
    let concatenated = '';
    for (let i = 0; i < Math.min(1000, strings.length); i++) {
        concatenated += strings[i] + '\n';
    }

    console.log(`  ✓ String operations complete (${concatenated.length} chars)`);
    return concatenated.length > 0;
}

// Array manipulation test
function testArrayOperations(size) {
    console.log(`  Testing array operations...`);

    // Create large array
    const arr = new Array(size).fill(0).map((_, i) => i);

    // Map operation (allocates new array)
    const mapped = arr.map(x => x * 2);

    // Filter operation (allocates new array)
    const filtered = mapped.filter(x => x % 4 === 0);

    // Reduce operation
    const sum = filtered.reduce((acc, x) => acc + x, 0);

    console.log(`  ✓ Array operations complete (sum: ${sum})`);
    return sum > 0;
}

// Main test suite
function runTests(options) {
    const config = SIZES[options.size];

    console.log('='.repeat(50));
    console.log('Test Runner for Build Pipeline Example');
    console.log('='.repeat(50));
    console.log(`Size: ${options.size}`);
    console.log(`Description: ${config.description}`);
    console.log(`Test data size: ${config.testDataSize.toLocaleString()}`);
    console.log(`Iterations: ${config.iterations.toLocaleString()}`);
    console.log();

    let testsPassed = 0;
    let testsFailed = 0;

    // Test 1: Binary validation
    console.log('Test 1: Binary Validation');
    if (testBinary(options.binary)) {
        testsPassed++;
    } else {
        testsFailed++;
    }
    console.log();

    // Test 2: Data processing
    console.log('Test 2: Data Processing');
    const testData = generateTestData(config.testDataSize);
    if (processTestData(testData, config.iterations)) {
        console.log('  ✓ Data processing test passed');
        testsPassed++;
    } else {
        console.log('  ✗ Data processing test failed');
        testsFailed++;
    }
    console.log();

    // Test 3: String operations
    console.log('Test 3: String Operations');
    if (testStringOperations(config.testDataSize)) {
        console.log('  ✓ String operations test passed');
        testsPassed++;
    } else {
        console.log('  ✗ String operations test failed');
        testsFailed++;
    }
    console.log();

    // Test 4: Array operations
    console.log('Test 4: Array Operations');
    if (testArrayOperations(config.testDataSize)) {
        console.log('  ✓ Array operations test passed');
        testsPassed++;
    } else {
        console.log('  ✗ Array operations test failed');
        testsFailed++;
    }
    console.log();

    // Summary
    console.log('='.repeat(50));
    console.log('Test Results');
    console.log('='.repeat(50));
    console.log(`Passed: ${testsPassed}`);
    console.log(`Failed: ${testsFailed}`);
    console.log();

    if (testsFailed === 0) {
        console.log('✓ All tests passed!');
        return 0;
    } else {
        console.log('✗ Some tests failed');
        return 1;
    }
}

// Entry point
function main() {
    try {
        const options = parseArgs();
        const exitCode = runTests(options);
        process.exit(exitCode);
    } catch (error) {
        console.error(`\nError: ${error.message}`);
        process.exit(1);
    }
}

main();
