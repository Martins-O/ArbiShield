#!/bin/bash
# Cross-platform Coverage Report Generator using grcov
# Works on Linux, macOS, and Windows (Git Bash)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
COVERAGE_DIR="target/coverage"
OUTPUT_DIR="coverage"
MIN_COVERAGE=95.0

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  grcov Coverage Report Generator${NC}"
echo -e "${BLUE}  (Cross-platform alternative)${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Step 1: Check for grcov
echo -e "${BLUE}[1/7]${NC} Checking for grcov..."
if ! command -v grcov &> /dev/null; then
    echo -e "${YELLOW}grcov not found. Installing...${NC}"
    cargo install grcov
    echo -e "${GREEN}✓ grcov installed${NC}"
else
    echo -e "${GREEN}✓ grcov found${NC}"
fi
echo ""

# Step 2: Check for llvm-tools
echo -e "${BLUE}[2/7]${NC} Checking for llvm-tools-preview..."
if ! rustup component list | grep -q "llvm-tools-preview (installed)"; then
    echo -e "${YELLOW}llvm-tools-preview not found. Installing...${NC}"
    rustup component add llvm-tools-preview
    echo -e "${GREEN}✓ llvm-tools-preview installed${NC}"
else
    echo -e "${GREEN}✓ llvm-tools-preview found${NC}"
fi
echo ""

# Step 3: Clean previous coverage data
echo -e "${BLUE}[3/7]${NC} Cleaning previous coverage data..."
rm -rf "$COVERAGE_DIR"
rm -rf "$OUTPUT_DIR"
mkdir -p "$COVERAGE_DIR"
mkdir -p "$OUTPUT_DIR"
echo -e "${GREEN}✓ Coverage directories prepared${NC}"
echo ""

# Step 4: Set environment variables
echo -e "${BLUE}[4/7]${NC} Setting up environment..."
export RUSTFLAGS="-C instrument-coverage"
export LLVM_PROFILE_FILE="$COVERAGE_DIR/arbishield-%p-%m.profraw"
echo -e "  RUSTFLAGS=$RUSTFLAGS"
echo -e "  LLVM_PROFILE_FILE=$LLVM_PROFILE_FILE"
echo -e "${GREEN}✓ Environment configured${NC}"
echo ""

# Step 5: Build and run tests
echo -e "${BLUE}[5/7]${NC} Building and running tests with coverage..."
echo -e "${YELLOW}This may take a few minutes...${NC}"
echo ""

cargo build
cargo test

echo ""
echo -e "${GREEN}✓ Tests completed${NC}"
echo ""

# Step 6: Generate coverage reports
echo -e "${BLUE}[6/7]${NC} Generating coverage reports..."

# Find binary path
BINARY_PATH=$(cargo metadata --format-version 1 | jq -r '.target_directory')/debug

# Generate HTML report
grcov "$COVERAGE_DIR" \
    --binary-path "$BINARY_PATH" \
    --source-dir . \
    --output-type html \
    --branch \
    --ignore-not-existing \
    --excl-line '#\[derive\(|//|/\*' \
    --ignore 'tests/*' \
    --ignore 'benches/*' \
    --ignore 'fuzz/*' \
    --ignore 'examples/*' \
    --output-path "$OUTPUT_DIR"

# Generate Cobertura XML for CI
grcov "$COVERAGE_DIR" \
    --binary-path "$BINARY_PATH" \
    --source-dir . \
    --output-type cobertura \
    --branch \
    --ignore-not-existing \
    --excl-line '#\[derive\(|//|/\*' \
    --ignore 'tests/*' \
    --ignore 'benches/*' \
    --ignore 'fuzz/*' \
    --ignore 'examples/*' \
    --output-path "$OUTPUT_DIR/cobertura.xml"

# Generate JSON for programmatic access
grcov "$COVERAGE_DIR" \
    --binary-path "$BINARY_PATH" \
    --source-dir . \
    --output-type coveralls \
    --branch \
    --ignore-not-existing \
    --excl-line '#\[derive\(|//|/\*' \
    --ignore 'tests/*' \
    --ignore 'benches/*' \
    --ignore 'fuzz/*' \
    --ignore 'examples/*' \
    --output-path "$OUTPUT_DIR/coveralls.json"

echo -e "${GREEN}✓ Reports generated${NC}"
echo ""

# Step 7: Extract coverage metrics
echo -e "${BLUE}[7/7]${NC} Extracting coverage metrics..."

if [ -f "$OUTPUT_DIR/cobertura.xml" ]; then
    # Extract line coverage from XML
    LINE_COVERAGE=$(grep -oP 'line-rate="\K[^"]+' "$OUTPUT_DIR/cobertura.xml" | head -1)
    LINE_COVERAGE_PCT=$(echo "$LINE_COVERAGE * 100" | bc)
    LINE_COVERAGE_PCT_INT=${LINE_COVERAGE_PCT%.*}

    # Extract branch coverage
    BRANCH_COVERAGE=$(grep -oP 'branch-rate="\K[^"]+' "$OUTPUT_DIR/cobertura.xml" | head -1)
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

# Final summary
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  Coverage Generation Complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo -e "Reports generated in: ${BLUE}${OUTPUT_DIR}/${NC}"
echo -e "  - HTML:    ${OUTPUT_DIR}/index.html"
echo -e "  - XML:     ${OUTPUT_DIR}/cobertura.xml"
echo -e "  - JSON:    ${OUTPUT_DIR}/coveralls.json"
echo ""
echo -e "To view the HTML report:"
case "$OSTYPE" in
    darwin*)  echo -e "  ${YELLOW}open ${OUTPUT_DIR}/index.html${NC}" ;;
    linux*)   echo -e "  ${YELLOW}xdg-open ${OUTPUT_DIR}/index.html${NC}" ;;
    msys*)    echo -e "  ${YELLOW}start ${OUTPUT_DIR}/index.html${NC}" ;;
    *)        echo -e "  ${YELLOW}Open ${OUTPUT_DIR}/index.html in your browser${NC}" ;;
esac
echo ""
echo -e "To upload to Codecov:"
echo -e "  ${YELLOW}bash <(curl -s https://codecov.io/bash) -f ${OUTPUT_DIR}/cobertura.xml${NC}"
echo ""

# Clean up environment variables
unset RUSTFLAGS
unset LLVM_PROFILE_FILE
