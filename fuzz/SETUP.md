# Fuzzing Setup Guide

This guide will help you set up and run fuzzing tests for ArbiShield contracts.

## Prerequisites

### 1. Install Rust Nightly

cargo-fuzz requires Rust nightly toolchain:

```bash
# Install nightly
rustup install nightly

# Set nightly as default (optional)
rustup default nightly

# Or use nightly just for this project
rustup override set nightly
```

### 2. Install cargo-fuzz

```bash
cargo install cargo-fuzz
```

### 3. Verify Installation

```bash
cargo fuzz --version
```

Expected output: `cargo-fuzz 0.x.x`

## Quick Start

### Run a Single Fuzzer

```bash
# Navigate to project root
cd /path/to/arbishield

# Run for 60 seconds
cargo fuzz run alert_registry_priority -- -max_total_time=60

# Run for specific number of iterations
cargo fuzz run arithmetic_fuzzer -- -runs=100000

# Run indefinitely (Ctrl+C to stop)
cargo fuzz run rbac_fuzzer
```

### Run All Fuzzers

```bash
cd fuzz
./run_all_fuzzers.sh
```

Configure run time:

```bash
# Run each fuzzer for 5 minutes
FUZZ_TIME=300 ./run_all_fuzzers.sh

# Run each fuzzer for 1 million iterations
FUZZ_RUNS=1000000 ./run_all_fuzzers.sh
```

## Common Issues

### Issue: "cargo-fuzz requires a nightly compiler"

**Solution**: Switch to nightly Rust

```bash
rustup default nightly
# Or for this project only
rustup override set nightly
```

### Issue: "failed to resolve: use of undeclared crate or module"

**Solution**: Ensure you're in the project root directory

```bash
cd /path/to/arbishield
cargo fuzz list  # Should show all fuzzers
```

### Issue: Fuzzer builds fail

**Solution**: Clean and rebuild

```bash
cargo fuzz clean
cargo fuzz build alert_registry_priority
```

### Issue: "Error: no fuzz targets found"

**Solution**: Check that `fuzz/Cargo.toml` and `fuzz/fuzz_targets/` exist

```bash
ls fuzz/fuzz_targets/
# Should list all .rs files
```

## Understanding Fuzzer Output

### Normal Operation

```
INFO: Running with entropic power schedule (0xFF, 100).
INFO: Seed: 3840238402
INFO: Loaded 1 modules   (1234 inline 8-bit counters)
#1000   pulse  cov: 42 ft: 58 corp: 8/24b lim: 4096 exec/s: 500
#10000  REDUCE cov: 45 ft: 67 corp: 12/34b lim: 4096 exec/s: 5000
```

**Key Metrics**:
- `cov`: Code coverage (number of edges hit)
- `ft`: Features (unique code paths discovered)
- `corp`: Corpus size (number of interesting inputs saved)
- `exec/s`: Executions per second (higher is better)

### Crash Found

```
==12345==ERROR: libFuzzer: deadly signal
    #0 0x55b123456789 in arbishield::alert_registry::compute_priority
SUMMARY: libFuzzer: deadly signal
artifact_prefix='./'; Test unit written to ./crash-abc123
```

**Action**: Investigate the crash

```bash
# Reproduce the crash
cargo fuzz run alert_registry_priority fuzz/artifacts/alert_registry_priority/crash-abc123

# Debug with lldb
rust-lldb -- fuzz/target/*/release/alert_registry_priority fuzz/artifacts/alert_registry_priority/crash-abc123
```

## Advanced Usage

### Parallel Fuzzing

Run multiple workers in parallel:

```bash
cargo fuzz run arithmetic_fuzzer -- -workers=4 -jobs=4
```

### Minimize Crash Input

Find the smallest input that reproduces a crash:

```bash
cargo fuzz cmin alert_registry_priority fuzz/artifacts/alert_registry_priority/crash-*
```

### Coverage-Guided Corpus Minimization

Reduce corpus to minimal set:

```bash
cargo fuzz cmin alert_registry_priority
```

### Coverage Report

Generate coverage report:

```bash
# Run with coverage
cargo fuzz coverage alert_registry_priority

# View coverage
cargo cov -- show fuzz/target/*/release/alert_registry_priority \
    --format=html \
    --instr-profile=fuzz/coverage/alert_registry_priority/coverage.profdata \
    > coverage.html

# Open in browser
firefox coverage.html
```

### Custom Dictionary

Create a dictionary file to guide fuzzing:

```bash
# Create fuzz/dict.txt
echo 'kw_admin="\x01"' > fuzz/dict.txt
echo 'kw_monitor="\x02"' >> fuzz/dict.txt

# Run with dictionary
cargo fuzz run rbac_fuzzer -- -dict=fuzz/dict.txt
```

### Continuous Fuzzing

For long-running fuzzing campaigns:

```bash
# Run for 24 hours
cargo fuzz run alert_registry_priority -- -max_total_time=86400

# Or indefinitely with checkpoints
while true; do
    cargo fuzz run alert_registry_priority -- -max_total_time=3600
    sleep 5
done
```

## Integration with CI/CD

See `.github/workflows/fuzz.yml` for GitHub Actions integration.

### GitLab CI

```yaml
fuzz:
  image: rust:nightly
  script:
    - cargo install cargo-fuzz
    - cd fuzz && ./run_all_fuzzers.sh
  artifacts:
    when: on_failure
    paths:
      - fuzz/artifacts/
```

### Jenkins

```groovy
pipeline {
    agent any
    stages {
        stage('Fuzz') {
            steps {
                sh 'rustup default nightly'
                sh 'cargo install cargo-fuzz'
                sh 'cd fuzz && FUZZ_TIME=300 ./run_all_fuzzers.sh'
            }
        }
    }
    post {
        failure {
            archiveArtifacts artifacts: 'fuzz/artifacts/**/*'
        }
    }
}
```

## Benchmarking Fuzzers

Compare fuzzer performance:

```bash
for fuzzer in alert_registry_priority circuit_breaker_state arithmetic_fuzzer; do
    echo "=== $fuzzer ==="
    cargo fuzz run $fuzzer -- -max_total_time=60 -print_final_stats=1 2>&1 | grep "exec/s"
done
```

## Best Practices

1. **Start small**: Run for 60 seconds first to verify setup
2. **Scale up**: Gradually increase to hours/days for thorough testing
3. **Parallel execution**: Use `-jobs` and `-workers` for faster coverage
4. **Save corpus**: Commit interesting inputs for regression testing
5. **Regular runs**: Fuzz daily or on every PR
6. **Monitor crashes**: Every crash is a potential bug/vulnerability
7. **Minimize crashes**: Use `cmin` to get the smallest reproducer
8. **Coverage tracking**: Generate coverage reports to identify untested code

## Troubleshooting

### Fuzzer is slow (< 1000 exec/s)

- Use release build (default for cargo-fuzz)
- Reduce instrumentation: `RUSTFLAGS="-C opt-level=3"`
- Simplify invariant checks in fuzz target

### Fuzzer plateaus (no new coverage)

- Let it run longer (coverage often plateaus then jumps)
- Try different seed: `-- -seed=12345`
- Use structured fuzzing for complex inputs

### Out of memory

- Reduce corpus size: `cargo fuzz cmin <target>`
- Limit max input size: `-- -max_len=1024`
- Add memory limit: `-- -rss_limit_mb=2048`

## Further Reading

- [cargo-fuzz book](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [libFuzzer docs](https://llvm.org/docs/LibFuzzer.html)
- [Fuzzing Rust](https://github.com/rust-fuzz/trophy-case)
- [Structure-Aware Fuzzing](https://github.com/rust-fuzz/arbitrary)
