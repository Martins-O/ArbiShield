#!/bin/bash
# Coverage Report Generator for ArbiShield
# Generates comprehensive code coverage reports and badges

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
COVERAGE_DIR="coverage"
BADGE_DIR="docs/badges"
MIN_COVERAGE=95.0

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  ArbiShield Coverage Report Generator${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Check if running on Linux (cargo-tarpaulin requirement)
if [[ "$OSTYPE" != "linux-gnu"* ]]; then
    echo -e "${YELLOW}Warning: cargo-tarpaulin only works on Linux${NC}"
    echo -e "${YELLOW}Consider using grcov for cross-platform coverage${NC}"
    echo ""
    echo "To use grcov instead:"
    echo "  1. Install: cargo install grcov"
    echo "  2. Install llvm-tools: rustup component add llvm-tools-preview"
    echo "  3. Run with LLVM profiling enabled"
    exit 1
fi

# Step 1: Check/Install cargo-tarpaulin
echo -e "${BLUE}[1/6]${NC} Checking for cargo-tarpaulin..."
if ! command -v cargo-tarpaulin &> /dev/null; then
    echo -e "${YELLOW}cargo-tarpaulin not found. Installing...${NC}"
    cargo install cargo-tarpaulin
    echo -e "${GREEN}✓ cargo-tarpaulin installed${NC}"
else
    echo -e "${GREEN}✓ cargo-tarpaulin found${NC}"
fi
echo ""

# Step 2: Clean previous coverage data
echo -e "${BLUE}[2/6]${NC} Cleaning previous coverage data..."
rm -rf "$COVERAGE_DIR"
mkdir -p "$COVERAGE_DIR"
mkdir -p "$BADGE_DIR"
echo -e "${GREEN}✓ Coverage directories prepared${NC}"
echo ""

# Step 3: Generate coverage report
echo -e "${BLUE}[3/6]${NC} Generating coverage report..."
echo -e "${YELLOW}This may take a few minutes...${NC}"
echo ""

cargo tarpaulin \
    --verbose \
    --all-features \
    --workspace \
    --timeout 300 \
    --out Xml \
    --out Html \
    --out Json \
    --output-dir "$COVERAGE_DIR" \
    --exclude-files 'tests/*' 'benches/*' 'fuzz/*' 'examples/*' \
    --engine llvm \
    --color always

echo ""
echo -e "${GREEN}✓ Coverage report generated${NC}"
echo ""

# Step 4: Extract coverage percentage
echo -e "${BLUE}[4/6]${NC} Extracting coverage metrics..."

if [ -f "$COVERAGE_DIR/cobertura.xml" ]; then
    # Extract line coverage from XML
    LINE_COVERAGE=$(grep -oP 'line-rate="\K[^"]+' "$COVERAGE_DIR/cobertura.xml" | head -1)
    LINE_COVERAGE_PCT=$(echo "$LINE_COVERAGE * 100" | bc)
    LINE_COVERAGE_PCT_INT=${LINE_COVERAGE_PCT%.*}

    # Extract branch coverage
    BRANCH_COVERAGE=$(grep -oP 'branch-rate="\K[^"]+' "$COVERAGE_DIR/cobertura.xml" | head -1)
    BRANCH_COVERAGE_PCT=$(echo "$BRANCH_COVERAGE * 100" | bc)
    BRANCH_COVERAGE_PCT_INT=${BRANCH_COVERAGE_PCT%.*}

    echo -e "  Line Coverage:   ${GREEN}${LINE_COVERAGE_PCT_INT}%${NC}"
    echo -e "  Branch Coverage: ${GREEN}${BRANCH_COVERAGE_PCT_INT}%${NC}"
    echo ""

    # Check if meets minimum threshold
    if (( $(echo "$LINE_COVERAGE_PCT < $MIN_COVERAGE" | bc -l) )); then
        echo -e "${RED}✗ Coverage ${LINE_COVERAGE_PCT_INT}% is below ${MIN_COVERAGE}% threshold${NC}"
        exit 1
    else
        echo -e "${GREEN}✓ Coverage ${LINE_COVERAGE_PCT_INT}% meets ${MIN_COVERAGE}% threshold${NC}"
    fi
else
    echo -e "${RED}✗ Coverage report not found${NC}"
    exit 1
fi
echo ""

# Step 5: Generate coverage badges
echo -e "${BLUE}[5/6]${NC} Generating coverage badges..."

# Function to get badge color based on coverage
get_badge_color() {
    local coverage=$1
    if (( $(echo "$coverage >= 95" | bc -l) )); then
        echo "brightgreen"
    elif (( $(echo "$coverage >= 90" | bc -l) )); then
        echo "green"
    elif (( $(echo "$coverage >= 80" | bc -l) )); then
        echo "yellowgreen"
    elif (( $(echo "$coverage >= 70" | bc -l) )); then
        echo "yellow"
    elif (( $(echo "$coverage >= 60" | bc -l) )); then
        echo "orange"
    else
        echo "red"
    fi
}

LINE_COLOR=$(get_badge_color "$LINE_COVERAGE_PCT_INT")
BRANCH_COLOR=$(get_badge_color "$BRANCH_COVERAGE_PCT_INT")

# Generate badge URLs and download
echo "  Generating line coverage badge..."
LINE_BADGE_URL="https://img.shields.io/badge/coverage-${LINE_COVERAGE_PCT_INT}%25-${LINE_COLOR}?style=flat-square"
curl -s "$LINE_BADGE_URL" > "$BADGE_DIR/coverage.svg"

echo "  Generating branch coverage badge..."
BRANCH_BADGE_URL="https://img.shields.io/badge/branch--coverage-${BRANCH_COVERAGE_PCT_INT}%25-${BRANCH_COLOR}?style=flat-square"
curl -s "$BRANCH_BADGE_URL" > "$BADGE_DIR/branch-coverage.svg"

# Generate test status badge (always passing if we got here)
TEST_BADGE_URL="https://img.shields.io/badge/tests-passing-brightgreen?style=flat-square"
curl -s "$TEST_BADGE_URL" > "$BADGE_DIR/tests.svg"

echo -e "${GREEN}✓ Badges generated in ${BADGE_DIR}${NC}"
echo ""

# Step 6: Generate summary report
echo -e "${BLUE}[6/6]${NC} Generating coverage summary..."

SUMMARY_FILE="$COVERAGE_DIR/SUMMARY.md"
cat > "$SUMMARY_FILE" << EOF
# Coverage Summary

**Generated:** $(date +"%Y-%m-%d %H:%M:%S")
**Tool:** cargo-tarpaulin
**Branch:** $(git branch --show-current)
**Commit:** $(git rev-parse --short HEAD)

---

## Overall Coverage

| Metric | Coverage | Status |
|--------|----------|--------|
| **Line Coverage** | ${LINE_COVERAGE_PCT_INT}% | $([ $LINE_COVERAGE_PCT_INT -ge 95 ] && echo "✅ PASS" || echo "❌ FAIL") |
| **Branch Coverage** | ${BRANCH_COVERAGE_PCT_INT}% | $([ $BRANCH_COVERAGE_PCT_INT -ge 90 ] && echo "✅ PASS" || echo "❌ FAIL") |
| **Threshold** | ${MIN_COVERAGE}% | $([ $LINE_COVERAGE_PCT_INT -ge 95 ] && echo "✅ MET" || echo "❌ NOT MET") |

---

## Badges

### Line Coverage
![Coverage](../docs/badges/coverage.svg)

\`\`\`markdown
![Coverage](docs/badges/coverage.svg)
\`\`\`

### Branch Coverage
![Branch Coverage](../docs/badges/branch-coverage.svg)

\`\`\`markdown
![Branch Coverage](docs/badges/branch-coverage.svg)
\`\`\`

### Tests
![Tests](../docs/badges/tests.svg)

\`\`\`markdown
![Tests](docs/badges/tests.svg)
\`\`\`

---

## Files

- **HTML Report:** [coverage/index.html](index.html)
- **XML Report:** [coverage/cobertura.xml](cobertura.xml)
- **JSON Report:** [coverage/tarpaulin-report.json](tarpaulin-report.json)

---

## Quick View

Open the HTML report for detailed coverage:

\`\`\`bash
# macOS
open coverage/index.html

# Linux
xdg-open coverage/index.html

# Windows (Git Bash)
start coverage/index.html
\`\`\`

---

**Generated by:** scripts/coverage.sh
EOF

echo -e "${GREEN}✓ Summary written to ${SUMMARY_FILE}${NC}"
echo ""

# Final summary
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  Coverage Generation Complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo -e "Reports generated in: ${BLUE}${COVERAGE_DIR}/${NC}"
echo -e "  - HTML:    ${COVERAGE_DIR}/index.html"
echo -e "  - XML:     ${COVERAGE_DIR}/cobertura.xml"
echo -e "  - JSON:    ${COVERAGE_DIR}/tarpaulin-report.json"
echo -e "  - Summary: ${COVERAGE_DIR}/SUMMARY.md"
echo ""
echo -e "Badges generated in: ${BLUE}${BADGE_DIR}/${NC}"
echo -e "  - coverage.svg"
echo -e "  - branch-coverage.svg"
echo -e "  - tests.svg"
echo ""
echo -e "To view the HTML report:"
echo -e "  ${YELLOW}xdg-open ${COVERAGE_DIR}/index.html${NC}"
echo ""
echo -e "To upload to Codecov:"
echo -e "  ${YELLOW}bash <(curl -s https://codecov.io/bash)${NC}"
echo ""
