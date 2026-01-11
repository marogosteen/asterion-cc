#!/bin/bash
set -euo pipefail

echo "=== asterion initialization ==="

# Check Rust toolchain
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo not found. Please install Rust toolchain."
    exit 1
fi

echo "Rust toolchain: $(rustc --version)"

# Cargo deny check (security and license)
echo "Running cargo deny check..."
if command -v cargo-deny &> /dev/null || cargo deny --version &> /dev/null; then
    if ! cargo deny check; then
        echo "Cargo deny check: FAILED"
        exit 1
    fi
    echo "Cargo deny check: PASSED"
else
    echo "Warning: cargo-deny not installed. Run 'cargo install cargo-deny' to enable security checks."
fi

# Format check
echo "Checking code format..."
if ! cargo fmt --check; then
    echo "Format check: FAILED - Run 'cargo fmt' to fix"
    exit 1
fi
echo "Format check: PASSED"

# Clippy check (strict mode)
echo "Running clippy (strict mode)..."
if ! cargo clippy -- -D warnings; then
    echo "Clippy check: FAILED"
    exit 1
fi
echo "Clippy check: PASSED"

# Run tests (serial to avoid test interference due to env::set_current_dir)
echo "Running tests..."
if ! cargo test -- --test-threads=1; then
    echo "Tests: FAILED"
    exit 1
fi
echo "Tests: PASSED"

# Coverage check (if cargo-llvm-cov is installed)
COVERAGE_THRESHOLD=50
echo "Checking code coverage..."
if command -v cargo-llvm-cov &> /dev/null || cargo llvm-cov --version &> /dev/null 2>&1; then
    # Generate coverage report and extract percentage
    COVERAGE_OUTPUT=$(cargo llvm-cov --summary-only 2>&1)

    if [ $? -eq 0 ]; then
        # Extract line coverage percentage (format: "TOTAL ... XX.XX%")
        COVERAGE_PCT=$(echo "$COVERAGE_OUTPUT" | grep -E "^TOTAL" | awk '{print $NF}' | tr -d '%')

        if [ -n "$COVERAGE_PCT" ]; then
            echo "Line coverage: ${COVERAGE_PCT}%"

            # Check against threshold (integer comparison)
            COVERAGE_INT=${COVERAGE_PCT%.*}
            if [ "$COVERAGE_INT" -lt "$COVERAGE_THRESHOLD" ]; then
                echo "Coverage check: FAILED - Coverage ${COVERAGE_PCT}% is below threshold ${COVERAGE_THRESHOLD}%"
                exit 1
            fi
            echo "Coverage check: PASSED (threshold: ${COVERAGE_THRESHOLD}%)"
        else
            echo "Warning: Could not parse coverage percentage"
        fi
    else
        echo "Warning: Coverage report generation failed"
    fi
else
    echo "Warning: cargo-llvm-cov not installed. Run 'cargo install cargo-llvm-cov' to enable coverage checks."
fi

# Build project
echo "Building project..."
if [ -f "Cargo.toml" ]; then
    cargo build --release
else
    echo "Warning: Cargo.toml not found. Skipping build."
fi

# CLI verification
echo "Verifying CLI..."
if [ -f "target/release/asterion" ]; then
    # Basic verification: check --help and --version work
    echo "Running CLI verification..."

    if ./target/release/asterion --help > /dev/null 2>&1; then
        echo "CLI --help: PASSED"
    else
        echo "CLI --help: FAILED"
        exit 1
    fi

    if ./target/release/asterion --version > /dev/null 2>&1; then
        echo "CLI --version: PASSED"
    else
        echo "CLI --version: FAILED"
        exit 1
    fi
else
    echo "Warning: Binary not found. Skipping CLI verification."
fi

echo "=== Initialization complete ==="
