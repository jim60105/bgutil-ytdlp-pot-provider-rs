#!/bin/bash
# scripts/check_coverage.sh
#
# Copyright (C) 2025 陳鈞
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program.  If not, see <https://www.gnu.org/licenses/>.
#
# Test coverage threshold checking script
# Use cargo llvm-cov to generate coverage report and verify minimum requirements

set -euo pipefail # Strict mode: exit immediately on error

# Default configuration
DEFAULT_THRESHOLD=75.0
COVERAGE_THRESHOLD=${COVERAGE_THRESHOLD:-$DEFAULT_THRESHOLD}
DEFAULT_NEXTEST_PROFILE="default"
NEXTEST_PROFILE=${NEXTEST_PROFILE:-$DEFAULT_NEXTEST_PROFILE}

# Full tests mode
FULL_TESTS=false

# Output LCOV file
LCOV_OUTPUT_PATH=""

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Usage information
usage() {
    echo "Usage: $0 [options]"
    echo "Options:"
    echo "  -t, --threshold PERCENT   Set coverage threshold (default: ${DEFAULT_THRESHOLD}%)"
    echo "  -p, --profile PROFILE     Set nextest profile (default: ${DEFAULT_NEXTEST_PROFILE})"
    echo "  --full                   Run full tests including slow tests (sets profile to full and enables slow-tests feature)"
    echo "  -T, --table              Show coverage table for all files"
    echo "  -f, --file FILENAME      Show coverage for specific file (supports partial matching)"
    echo "  -v, --verbose            Show verbose output"
    echo "  --lcov PATH              Also generate LCOV output file at specified path"
    echo "  -h, --help               Show this help"
    echo ""
    echo "Environment variables:"
    echo "  COVERAGE_THRESHOLD       Coverage threshold (default: ${DEFAULT_THRESHOLD}%)"
    echo "  NEXTEST_PROFILE          Nextest profile (default: ${DEFAULT_NEXTEST_PROFILE})"
    echo ""
    echo "Available nextest profiles: default, ci, quick, full"
    echo ""
    echo "Examples:"
    echo "  $0                       Check coverage with default threshold and profile (excludes slow tests)"
    echo "  $0 -t 80                 Set threshold to 80%"
    echo "  $0 -p ci                 Use CI profile"
    echo "  $0 --full                Run full tests including slow tests (~90 vs ~143 seconds)"
    echo "  $0 --table               Show coverage table for all files"
    echo "  $0 -f manager.rs         Show coverage for files matching 'manager.rs'"
    echo "  $0 --lcov lcov.info      Generate both JSON and LCOV coverage reports"
    echo "  COVERAGE_THRESHOLD=70 $0  Set threshold to 70% via environment variable"
    echo "  NEXTEST_PROFILE=ci $0     Set profile to ci via environment variable"
}

# Check dependencies
check_dependencies() {
    local missing_deps=()

    if ! command -v cargo &>/dev/null; then
        missing_deps+=("cargo")
    fi

    if ! command -v jq &>/dev/null; then
        missing_deps+=("jq")
    fi

    if ! command -v bc &>/dev/null; then
        missing_deps+=("bc")
    fi

    if ! cargo llvm-cov --version &>/dev/null; then
        missing_deps+=("cargo-llvm-cov")
    fi

    if ! cargo nextest --version &>/dev/null; then
        missing_deps+=("cargo-nextest")
    fi

    if [ ${#missing_deps[@]} -ne 0 ]; then
        echo -e "${RED}❌ Missing required dependencies:${NC}" >&2
        for dep in "${missing_deps[@]}"; do
            echo -e "   - ${dep}" >&2
        done
        echo -e "\n${YELLOW}Installation commands:${NC}" >&2
        echo -e "   cargo install cargo-llvm-cov" >&2
        echo -e "   cargo install cargo-nextest --locked" >&2
        echo -e "   # Ubuntu/Debian: sudo apt install jq bc" >&2
        echo -e "   # macOS: brew install jq bc" >&2
        exit 1
    fi
}

# Format percentage to 2 decimal places
format_percentage() {
    local field="$1"
    echo "(($field * 100 | round) / 100)"
}

# Generate jq filter for percentage formatting
get_percentage_filter() {
    echo 'def format_pct(x): ((x * 100 | round) / 100);'
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
        -t | --threshold)
            COVERAGE_THRESHOLD="$2"
            shift 2
            ;;
        -p | --profile)
            if [[ -z "$2" ]]; then
                echo -e "${RED}Error: --profile requires a value${NC}" >&2
                usage >&2
                exit 1
            fi
            # Validate profile value
            case "$2" in
                default|ci|quick|full)
                    NEXTEST_PROFILE="$2"
                    ;;
                *)
                    echo -e "${RED}Error: Invalid profile '$2'. Available profiles: default, ci, quick, full${NC}" >&2
                    exit 1
                    ;;
            esac
            shift 2
            ;;
        --full)
            FULL_TESTS=true
            NEXTEST_PROFILE="full"
            shift
            ;;
        -T | --table)
            SHOW_TABLE=true
            shift
            ;;
        -f | --file)
            SEARCH_FILE="$2"
            shift 2
            ;;
        -v | --verbose)
            VERBOSE=true
            shift
            ;;
        --lcov)
            LCOV_OUTPUT_PATH="$2"
            shift 2
            ;;
        -h | --help)
            usage
            exit 0
            ;;
        *)
            echo -e "${RED}Error: Unknown option $1${NC}" >&2
            usage >&2
            exit 1
            ;;
        esac
    done
}

# Display coverage table for all files
show_coverage_table() {
    local coverage_json="$1"

    echo -e "${BLUE}📊 File Coverage Report${NC}"
    echo ""

    # Table header
    printf "%-60s %8s %8s %8s %8s\n" "File" "Lines" "Funcs" "Regions" "Instants"
    printf "%-60s %8s %8s %8s %8s\n" "$(printf '%*s' 60 '' | tr ' ' '-')" "--------" "--------" "--------" "--------"

    # Extract and display file coverage data
    echo "$coverage_json" | jq -r "$(get_percentage_filter)"'
        .data[0].files[] |
        [
            .filename,
            (format_pct(.summary.lines.percent) | tostring + "%"),
            (format_pct(.summary.functions.percent) | tostring + "%"),
            (format_pct(.summary.regions.percent) | tostring + "%"),
            (format_pct(.summary.instantiations.percent) | tostring + "%")
        ] | @tsv
    ' | while IFS=$'\t' read -r filename lines_pct funcs_pct regions_pct instants_pct; do
        # Truncate long filenames for better display
        local display_name
        if [[ ${#filename} -gt 57 ]]; then
            display_name="...${filename: -54}"
        else
            display_name="$filename"
        fi

        # Color coding based on line coverage
        local line_coverage
        line_coverage=$(echo "$lines_pct" | sed 's/%//')
        local color=""
        if (( $(echo "$line_coverage >= 80" | bc -l) )); then
            color="$GREEN"
        elif (( $(echo "$line_coverage >= 60" | bc -l) )); then
            color="$YELLOW"
        else
            color="$RED"
        fi

        printf "%-60s ${color}%8s${NC} %8s %8s %8s\n" \
            "$display_name" "$lines_pct" "$funcs_pct" "$regions_pct" "$instants_pct"
    done

    echo ""
    echo -e "${BLUE}Legend:${NC} ${GREEN}>=80%${NC} ${YELLOW}60-79%${NC} ${RED}<60%${NC} (based on line coverage)"
    
    # Display overall coverage summary
    show_overall_coverage "$coverage_json"
}

# Display overall coverage summary and check threshold
show_overall_coverage() {
    local coverage_json="$1"
    local show_header="${2:-true}"
    
    if [[ "$show_header" == "true" ]]; then
        echo ""
        echo -e "${BLUE}📈 Overall Coverage Summary${NC}"
        echo ""
    fi
    
    # Parse overall coverage data
    local current_coverage
    if ! current_coverage=$(echo "$coverage_json" | jq -r "$(get_percentage_filter)"'format_pct(.data[0].totals.lines.percent)' 2>/dev/null); then
        echo -e "${RED}❌ Unable to parse overall coverage data${NC}" >&2
        return 1
    fi

    # Validate data validity
    if [[ "$current_coverage" == "null" ]] || [[ -z "$current_coverage" ]]; then
        echo -e "${RED}❌ Unable to get valid overall coverage data${NC}" >&2
        return 1
    fi

    # Display overall results
    echo -e "Current coverage: ${BLUE}${current_coverage}%${NC}"
    echo -e "Required threshold: ${BLUE}${COVERAGE_THRESHOLD}%${NC}"

    # Show detailed information (if verbose mode is enabled or in table mode)
    if [[ "${VERBOSE:-false}" == "true" ]] || [[ "${SHOW_TABLE:-false}" == "true" ]]; then
        echo -e "\n${YELLOW}Detailed coverage information:${NC}"
        echo "$coverage_json" | jq -r "$(get_percentage_filter)"'
            .data[0].totals |
            "  Function coverage: " + (format_pct(.functions.percent) | tostring) + "% (\(.functions.covered)/\(.functions.count))",
            "  Line coverage:     " + (format_pct(.lines.percent) | tostring) + "% (\(.lines.covered)/\(.lines.count))",
            "  Region coverage:   " + (format_pct(.regions.percent) | tostring) + "% (\(.regions.covered)/\(.regions.count))"
        '
    fi

    # Compare coverage with threshold
    if (($(echo "${current_coverage} >= ${COVERAGE_THRESHOLD}" | bc -l))); then
        echo -e "\n${GREEN}✅ Coverage meets requirements${NC}"
        return 0
    else
        local deficit
        deficit=$(echo "${COVERAGE_THRESHOLD} - ${current_coverage}" | bc -l)
        echo -e "\n${RED}❌ Coverage below threshold (deficit: ${deficit}%)${NC}"
        return 1
    fi
}

# Search and display coverage for specific file
search_file_coverage() {
    local coverage_json="$1"
    local search_pattern="$2"

    echo -e "${BLUE}🔍 Searching for files matching '${search_pattern}'...${NC}"
    echo ""

    # Search for matching files (case-insensitive)
    local matching_files
    matching_files=$(echo "$coverage_json" | jq -r --arg pattern "$search_pattern" "$(get_percentage_filter)"'
        .data[0].files[] |
        select(.filename | ascii_downcase | contains($pattern | ascii_downcase)) |
        [
            .filename,
            format_pct(.summary.lines.percent),
            .summary.lines.covered,
            .summary.lines.count,
            format_pct(.summary.functions.percent),
            .summary.functions.covered,
            .summary.functions.count,
            format_pct(.summary.regions.percent),
            .summary.regions.covered,
            .summary.regions.count,
            format_pct(.summary.instantiations.percent),
            .summary.instantiations.covered,
            .summary.instantiations.count
        ] | @tsv
    ')

    if [[ -z "$matching_files" ]]; then
        echo -e "${RED}❌ No files found matching '${search_pattern}'${NC}"
        return 1
    fi
    
    echo "$matching_files" | while IFS=$'\t' read -r filename lines_pct lines_covered lines_total \
        funcs_pct funcs_covered funcs_total regions_pct regions_covered regions_total \
        instants_pct instants_covered instants_total; do

        echo -e "${GREEN}📄 File: ${NC}${filename}"
        echo -e "   ${BLUE}Lines:${NC}         ${lines_pct}% (${lines_covered}/${lines_total})"
        echo -e "   ${BLUE}Functions:${NC}     ${funcs_pct}% (${funcs_covered}/${funcs_total})"
        echo -e "   ${BLUE}Regions:${NC}       ${regions_pct}% (${regions_covered}/${regions_total})"
        echo -e "   ${BLUE}Instantiations:${NC} ${instants_pct}% (${instants_covered}/${instants_total})"
        echo ""
    done
}

# Main coverage check function
check_coverage() {
    echo -e "${BLUE}🔍 Checking test coverage...${NC}"
    echo -e "${BLUE}🔧 Using nextest profile: ${NEXTEST_PROFILE}${NC}"
    if [[ "${FULL_TESTS}" == "true" ]]; then
        echo -e "${BLUE}⚡ Full tests mode: Including slow tests (~143s vs ~90s)${NC}"
    else
        echo -e "${BLUE}🚀 Fast tests mode: Excluding slow tests (~90s vs ~143s)${NC}"
        echo -e "${BLUE}   Use --full to include slow tests${NC}"
    fi

    # Generate coverage report
    local coverage_json
    local coverage_cmd_output
    local coverage_features=""
    if [[ "${FULL_TESTS}" == "true" ]]; then
        coverage_features="--features slow-tests"
    fi
    
    echo -e "${BLUE}📄 Running tests to generate coverage data...${NC}"
    
    # First, run tests once to generate coverage data
    if ! cargo llvm-cov nextest --profile "${NEXTEST_PROFILE}" --workspace ${coverage_features} --no-report >/dev/null 2>&1; then
        echo -e "${RED}❌ Unable to run tests for coverage${NC}" >&2
        exit 1
    fi
    
    echo -e "${GREEN}✅ Tests completed successfully${NC}"
    
    # Generate LCOV output if requested, using report to avoid re-running tests
    if [[ -n "${LCOV_OUTPUT_PATH}" ]]; then
        echo -e "${BLUE}📄 Generating LCOV output at: ${LCOV_OUTPUT_PATH}${NC}"
        if ! cargo llvm-cov report --lcov --output-path "${LCOV_OUTPUT_PATH}" >/dev/null 2>&1; then
            echo -e "${RED}❌ Unable to generate LCOV coverage report${NC}" >&2
            exit 1
        fi
        echo -e "${GREEN}✅ LCOV coverage report generated successfully${NC}"
    fi
    
    # Generate JSON output for analysis, using report to avoid re-running tests
    if ! coverage_cmd_output=$(cargo llvm-cov report --json --summary-only 2>&1); then
        echo -e "${RED}❌ Unable to extract JSON coverage data${NC}" >&2
        echo -e "${YELLOW}Error message: ${coverage_cmd_output}${NC}" >&2
        exit 1
    fi
    
    echo -e "${GREEN}✅ JSON coverage data extracted successfully${NC}"

    # Extract JSON part (filter out test output)
    coverage_json=$(echo "$coverage_cmd_output" | grep -E '^\{.*\}$' | tail -1)

    if [[ -z "$coverage_json" ]]; then
        echo -e "${RED}❌ Unable to extract JSON data from output${NC}" >&2
        echo -e "${YELLOW}Raw output:${NC}" >&2
        echo "$coverage_cmd_output" >&2
        exit 1
    fi

    # Handle table display option
    if [[ "${SHOW_TABLE:-false}" == "true" ]]; then
        show_coverage_table "$coverage_json"
        return $?
    fi

    # Handle file search option
    if [[ -n "${SEARCH_FILE:-}" ]]; then
        search_file_coverage "$coverage_json" "$SEARCH_FILE"
        return 0
    fi

    # Default mode - show overall coverage
    show_overall_coverage "$coverage_json" "false"
}

# Main program
main() {
    parse_args "$@"
    check_dependencies
    check_coverage
}

# Execute main program
main "$@"
