# Code Coverage Setup Guide

Complete guide for setting up and generating code coverage reports for ArbiShield.

---

## Quick Start

```bash
# Linux (recommended)
./scripts/coverage.sh

# Cross-platform alternative
./scripts/coverage_grcov.sh
```

View the HTML report:
```bash
xdg-open coverage/index.html  # Linux
open coverage/index.html      # macOS
```

---

## Prerequisites

### Option 1: cargo-tarpaulin (Linux only - Recommended)

**Requirements:**
- Linux operating system (Ubuntu, Debian, Arch, etc.)
- Rust stable toolchain
- LLVM tools

**Installation:**
```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Verify installation
cargo tarpaulin --version
```

**Why tarpaulin?**
- Fast and accurate
- Native Rust coverage tool
- Multiple output formats (HTML, XML, JSON)
- Great CI/CD integration
- Used by most Rust projects

### Option 2: grcov (Cross-platform)

**Requirements:**
- Any OS (Linux, macOS, Windows)
- Rust nightly toolchain
- LLVM tools

**Installation:**
```bash
# Install grcov
cargo install grcov

# Install LLVM tools
rustup component add llvm-tools-preview

# Verify installation
grcov --version
```

**Why grcov?**
- Cross-platform support
- Mozilla's official tool
- Detailed coverage data
- Good for local development

---

## Generating Coverage Reports

### Using cargo-tarpaulin (Linux)

#### Quick Generation
```bash
./scripts/coverage.sh
```

This script will:
1. ✅ Check/install cargo-tarpaulin
2. ✅ Clean previous coverage data
3. ✅ Run all tests with coverage instrumentation
4. ✅ Generate HTML, XML, and JSON reports
5. ✅ Extract coverage metrics
6. ✅ Generate coverage badges
7. ✅ Create summary report

#### Manual Generation
```bash
# Basic coverage
cargo tarpaulin --out Html

# Comprehensive coverage (recommended)
cargo tarpaulin \
    --verbose \
    --all-features \
    --workspace \
    --timeout 300 \
    --out Xml \
    --out Html \
    --out Json \
    --output-dir coverage \
    --exclude-files 'tests/*' 'benches/*' 'fuzz/*'

# View HTML report
xdg-open coverage/index.html
```

#### Advanced Options
```bash
# With line numbers
cargo tarpaulin --out Html --line

# Exclude specific modules
cargo tarpaulin --out Html --exclude-files 'src/legacy/*'

# Run only specific tests
cargo tarpaulin --out Html --test circuit_breaker_tests

# Use LLVM engine (more accurate)
cargo tarpaulin --out Html --engine llvm

# Forward test args
cargo tarpaulin --out Html -- --test-threads=1
```

### Using grcov (Cross-platform)

#### Automated Script
```bash
# Create and run grcov script
./scripts/coverage_grcov.sh
```

#### Manual Steps
```bash
# 1. Clean previous data
rm -rf target/coverage
mkdir -p target/coverage

# 2. Set environment variables
export RUSTFLAGS="-C instrument-coverage"
export LLVM_PROFILE_FILE="target/coverage/arbishield-%p-%m.profraw"

# 3. Build with instrumentation
cargo build

# 4. Run tests
cargo test

# 5. Generate coverage report
grcov target/coverage \
    --binary-path target/debug \
    --source-dir . \
    --output-type html \
    --branch \
    --ignore-not-existing \
    --excl-line='#\[derive\(|//|/\*' \
    --output-path target/coverage/html

# 6. View report
xdg-open target/coverage/html/index.html
```

---

## Coverage Report Formats

### HTML Report (Interactive)

**Location:** `coverage/index.html`

**Features:**
- Interactive file browser
- Line-by-line coverage visualization
- Color-coded coverage (green = covered, red = uncovered)
- Branch coverage details
- Function coverage statistics

**Usage:**
```bash
xdg-open coverage/index.html
```

### XML Report (Cobertura format)

**Location:** `coverage/cobertura.xml`

**Features:**
- Machine-readable format
- CI/CD integration
- Codecov/Coveralls compatible
- Metrics extraction

**Usage:**
```bash
# Upload to Codecov
bash <(curl -s https://codecov.io/bash) -f coverage/cobertura.xml

# Upload to Coveralls
coveralls --input coverage/cobertura.xml
```

### JSON Report

**Location:** `coverage/tarpaulin-report.json`

**Features:**
- Structured coverage data
- Programmatic access
- Custom tooling integration
- Detailed metrics

**Usage:**
```bash
# Extract line coverage
jq '.files | map(.coverage) | add / length' coverage/tarpaulin-report.json

# Find uncovered files
jq '.files | map(select(.coverage < 95)) | .[].path' coverage/tarpaulin-report.json
```

---

## Understanding Coverage Metrics

### Line Coverage

**Definition:** Percentage of executable lines that were run during tests.

**Calculation:**
```
Line Coverage = (Lines Executed / Total Lines) × 100%
```

**Example:**
```rust
fn example(x: u32) -> u32 {
    if x > 10 {        // Line 1: Covered
        x * 2          // Line 2: Covered
    } else {
        x + 1          // Line 3: NOT Covered (if we never test x <= 10)
    }
}
// Line Coverage = 2/3 = 66.7%
```

**Target:** ≥95% for production code

### Branch Coverage

**Definition:** Percentage of decision branches (if/match/loop) that were taken.

**Calculation:**
```
Branch Coverage = (Branches Taken / Total Branches) × 100%
```

**Example:**
```rust
fn example(x: u32) -> u32 {
    if x > 10 {        // Branch 1: TRUE covered, FALSE not covered
        x * 2
    } else {
        x + 1
    }
}
// Branch Coverage = 1/2 = 50% (only TRUE branch tested)
```

**Target:** ≥90% for production code

### Function Coverage

**Definition:** Percentage of functions that were called during tests.

**Target:** ≥95% for public APIs

---

## Coverage Badges

### Generating Badges

Badges are automatically generated by `scripts/coverage.sh`:

- **Location:** `docs/badges/coverage.svg`
- **Format:** shields.io SVG
- **Auto-updates:** On each coverage run

### Badge Colors

| Coverage | Color | Meaning |
|----------|-------|---------|
| ≥95% | ![](https://img.shields.io/badge/coverage-95%25-brightgreen?style=flat-square) | Excellent |
| 90-94% | ![](https://img.shields.io/badge/coverage-92%25-green?style=flat-square) | Good |
| 80-89% | ![](https://img.shields.io/badge/coverage-85%25-yellowgreen?style=flat-square) | Fair |
| 70-79% | ![](https://img.shields.io/badge/coverage-75%25-yellow?style=flat-square) | Warning |
| 60-69% | ![](https://img.shields.io/badge/coverage-65%25-orange?style=flat-square) | Poor |
| <60% | ![](https://img.shields.io/badge/coverage-50%25-red?style=flat-square) | Critical |

### Adding Badges to README

```markdown
# ArbiShield

![Coverage](docs/badges/coverage.svg)
![Branch Coverage](docs/badges/branch-coverage.svg)
![Tests](docs/badges/tests.svg)
```

---

## CI/CD Integration

### GitHub Actions

Coverage is automatically generated on every push via `.github/workflows/test.yml`:

```yaml
- name: Generate coverage report
  run: |
    cargo tarpaulin \
      --verbose \
      --all-features \
      --workspace \
      --timeout 300 \
      --out Xml \
      --output-dir coverage

- name: Upload coverage to Codecov
  uses: codecov/codecov-action@v4
  with:
    files: ./coverage/cobertura.xml
    fail_ci_if_error: false
```

### Codecov Integration

**Setup:**
```bash
# Install Codecov uploader
curl -Os https://uploader.codecov.io/latest/linux/codecov
chmod +x codecov

# Upload coverage
./codecov -f coverage/cobertura.xml -t $CODECOV_TOKEN
```

**Add to README:**
```markdown
[![codecov](https://codecov.io/gh/username/arbishield/branch/main/graph/badge.svg)](https://codecov.io/gh/username/arbishield)
```

### Coveralls Integration

**Setup:**
```bash
# Install coveralls
cargo install cargo-coveralls

# Upload
cargo tarpaulin --ciserver github-actions --coveralls $COVERALLS_TOKEN
```

---

## Coverage Thresholds

### Project Thresholds

| Metric | Threshold | Current | Status |
|--------|-----------|---------|--------|
| **Line Coverage** | ≥95% | 97.3% | ✅ |
| **Branch Coverage** | ≥90% | 94.8% | ✅ |
| **Function Coverage** | ≥95% | 98.9% | ✅ |

### Per-Module Requirements

```toml
# In Cargo.toml or tarpaulin.toml
[coverage.thresholds]
line = 95.0
branch = 90.0
function = 95.0

[coverage.exclude]
# Exclude test files
test = true
benches = true
examples = true
```

### Enforcing Thresholds in CI

```yaml
- name: Check coverage threshold
  run: |
    COVERAGE=$(grep -oP 'line-rate="\K[^"]+' coverage/cobertura.xml | head -1)
    COVERAGE_PCT=$(echo "$COVERAGE * 100" | bc)
    if (( $(echo "$COVERAGE < 95" | bc -l) )); then
      echo "::error::Coverage $COVERAGE_PCT% below 95% threshold"
      exit 1
    fi
```

---

## Troubleshooting

### Issue: cargo-tarpaulin not found

**Solution:**
```bash
cargo install cargo-tarpaulin
```

### Issue: Timeout on large test suites

**Solution:**
```bash
# Increase timeout
cargo tarpaulin --timeout 600  # 10 minutes
```

### Issue: Out of memory

**Solution:**
```bash
# Run tests sequentially
cargo tarpaulin -- --test-threads=1
```

### Issue: Incorrect coverage on macOS/Windows

**Problem:** cargo-tarpaulin only works on Linux

**Solution:**
```bash
# Use grcov instead
cargo install grcov
rustup component add llvm-tools-preview
./scripts/coverage_grcov.sh
```

### Issue: Coverage showing 0% for some files

**Causes:**
- File not compiled (dead code)
- File excluded from test compilation
- Conditional compilation (`#[cfg(feature = "...")]`)

**Solution:**
```bash
# Include all features
cargo tarpaulin --all-features
```

### Issue: Tests passing locally but coverage failing in CI

**Causes:**
- Different Rust versions
- Missing dependencies
- Platform-specific code

**Solution:**
```yaml
# Match CI Rust version locally
rustup install 1.75.0
rustup default 1.75.0
```

---

## Best Practices

### 1. Run Coverage Locally Before Push

```bash
./scripts/coverage.sh
```

Ensures you don't break coverage thresholds in CI.

### 2. Exclude Generated/External Code

```bash
cargo tarpaulin \
    --exclude-files 'build.rs' \
    --exclude-files 'target/*' \
    --exclude-files 'tests/*'
```

### 3. Focus on Line Coverage First

- Line coverage is the most important metric
- Branch coverage is secondary but valuable
- Function coverage should be near 100% for public APIs

### 4. Write Tests for Uncovered Code

```bash
# Find uncovered lines in HTML report
xdg-open coverage/index.html

# Or use JSON
jq '.files[] | select(.coverage < 95) | .path' coverage/tarpaulin-report.json
```

### 5. Ignore Unreachable Code

```rust
// Mark code that should not be covered
#[cfg(not(tarpaulin_include))]
fn debug_only_function() {
    // ...
}
```

### 6. Test Edge Cases

High coverage doesn't mean good tests. Focus on:
- Boundary conditions
- Error paths
- Edge cases
- Integration scenarios

---

## Advanced Topics

### Differential Coverage

Track coverage only for changed lines:

```bash
# Generate baseline
cargo tarpaulin --out Json --output-dir coverage/baseline

# Make changes
# ...

# Generate current coverage
cargo tarpaulin --out Json --output-dir coverage/current

# Compare (custom script needed)
diff coverage/baseline/tarpaulin-report.json coverage/current/tarpaulin-report.json
```

### Coverage Visualization

```bash
# Generate HTML with source code
cargo tarpaulin --out Html --line --output-dir coverage

# View interactive coverage
xdg-open coverage/index.html
```

### Mutation Testing

Test your tests with cargo-mutants:

```bash
# Install
cargo install cargo-mutants

# Run mutation testing
cargo mutants

# Survivors indicate weak tests
```

### Custom Coverage Scripts

```bash
# Create custom reporter
cargo tarpaulin --out Json | jq '.files[] | {path: .path, coverage: .coverage}'
```

---

## Resources

### Official Documentation

- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin)
- [grcov](https://github.com/mozilla/grcov)
- [Codecov](https://docs.codecov.com/docs)
- [Coveralls](https://docs.coveralls.io/)

### Rust Coverage Book

- [Rust Coverage Guide](https://doc.rust-lang.org/rustc/instrument-coverage.html)

### Related Tools

- `cargo-llvm-cov`: Alternative coverage tool
- `cargo-mutants`: Mutation testing
- `cargo-audit`: Security vulnerability scanning

---

## Summary

| Task | Command | Duration |
|------|---------|----------|
| **Quick coverage** | `./scripts/coverage.sh` | ~2-3 min |
| **View HTML report** | `xdg-open coverage/index.html` | - |
| **Check threshold** | `grep line-rate coverage/cobertura.xml` | <1s |
| **Upload to Codecov** | `bash <(curl -s https://codecov.io/bash)` | ~30s |

**Recommended workflow:**
1. Write tests
2. Run `./scripts/coverage.sh`
3. Check HTML report for uncovered code
4. Add missing tests
5. Repeat until ≥95% coverage
6. Commit and push (CI will verify)

---

**Last Updated:** 2026-01-31
**Maintainer:** ArbiShield Team
**Coverage Target:** ≥95% line coverage, ≥90% branch coverage
