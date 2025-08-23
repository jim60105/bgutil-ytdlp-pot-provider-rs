#!/bin/bash
# scripts/quality_check.sh
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
# Quality Assurance Check Script
# This script performs comprehensive code quality checks including compilation,
# formatting, linting, documentation, and testing

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

# Initialize verbose mode
VERBOSE=false

# Initialize nextest profile
NEXTEST_PROFILE="default"

# Initialize full tests mode
FULL_TESTS=false

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Usage information
usage() {
    echo "Usage: $0 [options]"
    echo "Options:"
    echo "  -v, --verbose            Show verbose output"
    echo "  -p, --profile PROFILE    Set nextest profile (default: default)"
    echo "  --full                   Run full tests including slow tests (sets profile to full and enables slow-tests feature)"
    echo "  -h, --help               Show this help"
    echo ""
    echo "This script performs comprehensive quality assurance checks including:"
    echo "  - Code compilation and formatting checks"
    echo "  - Clippy linting and code quality analysis"
    echo "  - Documentation generation and coverage"
    echo "  - Documentation examples testing"
    echo "  - Unit and integration tests"
    echo ""
    echo "Available nextest profiles: default, ci, quick, full"
    echo ""
    echo "Examples:"
    echo "  $0                       Run all quality checks with standard output (excludes slow tests)"
    echo "  $0 -v                    Run all quality checks with verbose output (excludes slow tests)"
    echo "  $0 -p ci                 Run with CI profile"
    echo "  $0 --full                Run full tests including slow tests (~90 vs ~143 seconds)"
    echo "  $0 -v --full             Run full tests with verbose output"
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
        -v | --verbose)
            VERBOSE=true
            shift
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

# Function to print colored output
print_status() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

# Function to check command success
check_result() {
    local exit_code=$1
    local test_name=$2
    
    if [ "${exit_code}" -eq 0 ]; then
        print_status "$GREEN" "✅ $test_name: Passed"
        return 0
    else
        print_status "$RED" "❌ $test_name: Failed"
        return 1
    fi
}

# Initialize counters
total_checks=0
passed_checks=0
failed_checks=0

run_check() {
    local check_name=$1
    local command=$2
    
    total_checks=$((total_checks + 1))
    if [[ "${VERBOSE}" == "true" ]]; then
        print_status "$BLUE" "\n🔍 Running check: $check_name"
    fi
    
    if eval "$command"; then
        check_result 0 "$check_name"
        passed_checks=$((passed_checks + 1))
        return 0
    else
        check_result $? "$check_name"
        failed_checks=$((failed_checks + 1))
        return 1
    fi
}

# Function to run tests with conditional output (hide output unless failure or verbose)
run_test_with_conditional_output() {
    local test_name=$1
    local command=$2
    
    total_checks=$((total_checks + 1))
    
    if [[ "${VERBOSE}" == "true" ]]; then
        # Verbose mode: show all output
        print_status "$BLUE" "\n🔍 Running check: $test_name"
        if eval "$command"; then
            check_result 0 "$test_name"
            passed_checks=$((passed_checks + 1))
            return 0
        else
            check_result $? "$test_name"
            failed_checks=$((failed_checks + 1))
            return 1
        fi
    else
        # Non-verbose mode: capture output and only show on failure
        local temp_output
        temp_output=$(mktemp)
        
        if eval "$command" > "$temp_output" 2>&1; then
            # Test passed - only show success message
            check_result 0 "$test_name"
            passed_checks=$((passed_checks + 1))
            rm -f "$temp_output"
            return 0
        else
            # Test failed - show failure message and output
            check_result $? "$test_name"
            echo ""
            echo "=== Test Output ==="
            cat "$temp_output"
            echo "==================="
            failed_checks=$((failed_checks + 1))
            rm -f "$temp_output"
            return 1
        fi
    fi
}

# Main function
main() {
    parse_args "$@"
    
    # Show startup message
    echo "🔍 Quality Assurance Check Starting..."
    echo "========================================"
    echo "🔧 Using nextest profile: ${NEXTEST_PROFILE}"
    if [[ "${FULL_TESTS}" == "true" ]]; then
        echo "⚡ Full tests mode: Including slow tests (~143s vs ~90s)"
    else
        echo "🚀 Fast tests mode: Excluding slow tests (~90s vs ~143s)"
        echo "   Use --full to include slow tests"
    fi
    echo "========================================"
    
    # 1. Code compilation check
    local cargo_features=""
    if [[ "${FULL_TESTS}" == "true" ]]; then
        cargo_features="--features slow-tests"
    fi
    
    if [[ "${VERBOSE}" == "true" ]]; then
        run_check "Code Compilation Check" "cargo check --all-features ${cargo_features}"
    else
        run_check "Code Compilation Check" "cargo check --all-features ${cargo_features} --quiet"
    fi

    # 2. Code formatting check
    run_check "Code Formatting Check" "cargo fmt -- --check"

    # 3. Clippy linting check
    if [[ "${VERBOSE}" == "true" ]]; then
        run_check "Clippy Code Quality Check" "cargo clippy --all-features ${cargo_features} -- -D warnings"
    else
        run_check "Clippy Code Quality Check" "cargo clippy --all-features ${cargo_features} --quiet -- -D warnings"
    fi

    # 4. Documentation generation check
    if [[ "${VERBOSE}" == "true" ]]; then
        print_status "$BLUE" "\n🔍 Running check: Documentation Generation Check"
    fi
    total_checks=$((total_checks + 1))

    # Create temporary file for documentation output
    local doc_output
    doc_output=$(mktemp)

    if [[ "${VERBOSE}" == "true" ]]; then
        cargo doc --all-features ${cargo_features} --no-deps --document-private-items 2>&1 | tee "$doc_output"
    else
        cargo doc --all-features ${cargo_features} --no-deps --document-private-items > "$doc_output" 2>&1
    fi

    # Check for critical errors (excluding known lint warnings)
    if grep -E "(error)" "$doc_output" | grep -v "warning\[E0602\]: unknown lint"; then
        print_status "$RED" "❌ Documentation Generation Check: Critical errors found"
        failed_checks=$((failed_checks + 1))
    else
        # Count warnings (excluding known lint warnings)
        warning_lines=$(grep -E "(warning)" "$doc_output" | grep -v "warning\[E0602\]: unknown lint" || true)
        if [ -n "$warning_lines" ]; then
            warning_count=$(echo "$warning_lines" | wc -l)
        else
            warning_count=0
        fi
        if [ "$warning_count" -gt 0 ]; then
            print_status "$YELLOW" "⚠️  Documentation Generation Check: Passed (with $warning_count warnings)"
        else
            print_status "$GREEN" "✅ Documentation Generation Check: Passed"
        fi
        passed_checks=$((passed_checks + 1))
    fi

    # Clean up documentation output file
    rm -f "$doc_output"

    # 5. Documentation examples test
    run_test_with_conditional_output "Documentation Examples Test" "cargo test --doc --all-features ${cargo_features}"

    # 6. Documentation coverage check  
    if [[ "${VERBOSE}" == "true" ]]; then
        print_status "$BLUE" "\n🔍 Running check: Documentation Coverage Check"
    fi
    total_checks=$((total_checks + 1))

    # Check for missing documentation (allow warnings, don't fail build)
    if [[ "${VERBOSE}" == "true" ]]; then
        missing_docs_output=$(cargo clippy --all-features ${cargo_features} -- -W missing_docs 2>&1 | grep -v "warning\[E0602\]" | grep "missing documentation" || true)
    else
        missing_docs_output=$(cargo clippy --all-features ${cargo_features} --quiet -- -W missing_docs 2>&1 | grep -v "warning\[E0602\]" | grep "missing documentation" || true)
    fi

    if [ -n "$missing_docs_output" ]; then
        if [ -n "$missing_docs_output" ]; then
            missing_count=$(echo "$missing_docs_output" | wc -l)
        else
            missing_count=0
        fi
        print_status "$YELLOW" "⚠️  Documentation Coverage Check: Found $missing_count items missing documentation"
        
        # Only show details in verbose mode
        if [[ "${VERBOSE}" == "true" ]]; then
            # Only show first 5 items to avoid overwhelming output
            echo "$missing_docs_output" | head -5
            if [ "$missing_count" -gt 5 ]; then
                echo "... (showing first 5 of $missing_count items)"
            fi
            print_status "$BLUE" "ℹ️  These are improvement suggestions and won't affect build success"
        fi
    else
        print_status "$GREEN" "✅ Documentation Coverage Check: All public APIs have documentation"
    fi
    passed_checks=$((passed_checks + 1))

    # 7. Unit tests
    local nextest_features=""
    if [[ "${FULL_TESTS}" == "true" ]]; then
        nextest_features="--features slow-tests"
    fi
    run_test_with_conditional_output "Unit Tests" "cargo nextest run --profile ${NEXTEST_PROFILE} ${nextest_features} -E 'kind(lib)' --ignore-default-filter"

    # 8. Integration tests  
    run_test_with_conditional_output "Integration Tests" "cargo nextest run --profile ${NEXTEST_PROFILE} ${nextest_features} --ignore-default-filter"

    # Cleanup
    # (Temporary files are cleaned up in their respective sections)

    # Summary
    echo ""
    echo "========================================"
    print_status "$BLUE" "📊 Quality Assurance Check Summary"
    echo "========================================"
    print_status "$GREEN" "✅ Passed checks: $passed_checks"
    print_status "$RED" "❌ Failed checks: $failed_checks"  
    print_status "$BLUE" "📋 Total checks: $total_checks"

    if [ $failed_checks -eq 0 ]; then
        print_status "$GREEN" "\n🎉 All quality assurance checks passed!"
        exit 0
    else
        print_status "$RED" "\n⚠️  Some checks failed, please review the error messages above"
        exit 1
    fi
}

# Execute main program
main "$@"
