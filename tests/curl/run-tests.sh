#!/bin/bash

# RustAxum cURL Test Runner
# Orchestrates all cURL-based API tests with reporting and CI/CD integration

set -e

# Configuration
BASE_URL="${BASE_URL:-http://localhost:3000}"
TEST_TYPE="${TEST_TYPE:-all}"
REPORT_FORMAT="${REPORT_FORMAT:-console}"  # console, json, html, junit
OUTPUT_DIR="${OUTPUT_DIR:-./test-results}"
PARALLEL="${PARALLEL:-false}"
FAIL_FAST="${FAIL_FAST:-false}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Test results tracking
TOTAL_TEST_SUITES=0
PASSED_TEST_SUITES=0
FAILED_TEST_SUITES=0
TOTAL_EXECUTION_TIME=0

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Logging functions
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

success() {
    echo -e "${GREEN}✓${NC} $1"
}

error() {
    echo -e "${RED}✗${NC} $1"
}

warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

info() {
    echo -e "${PURPLE}ℹ${NC} $1"
}

header() {
    echo -e "\n${BOLD}${CYAN}$1${NC}"
    echo -e "${CYAN}$(printf '%.0s=' {1..60})${NC}"
}

# Check prerequisites
check_prerequisites() {
    log "Checking prerequisites..."

    # Check if server is running
    if ! curl -s --connect-timeout 5 "$BASE_URL/api/docs" >/dev/null 2>&1; then
        error "API server is not accessible at $BASE_URL"
        error "Please start the server with: cargo run --bin artisan -- serve --port 3000"
        exit 1
    fi

    success "API server is accessible at $BASE_URL"

    # Check for required tools
    local missing_tools=()

    if ! command -v curl >/dev/null 2>&1; then
        missing_tools+=("curl")
    fi

    if [[ "$REPORT_FORMAT" == "json" || "$REPORT_FORMAT" == "html" ]] && ! command -v jq >/dev/null 2>&1; then
        warning "jq not found - JSON parsing may be limited"
    fi

    if [[ ${#missing_tools[@]} -gt 0 ]]; then
        error "Missing required tools: ${missing_tools[*]}"
        exit 1
    fi

    success "All required tools are available"
}

# Run a single test suite
run_test_suite() {
    local test_name="$1"
    local test_script="$2"
    local test_args="${3:-}"

    ((TOTAL_TEST_SUITES++))

    header "Running $test_name"

    local start_time=$(date +%s)
    local suite_output="$OUTPUT_DIR/${test_name,,}-output.txt"
    local suite_result="$OUTPUT_DIR/${test_name,,}-result.json"

    # Run the test script
    local exit_code=0

    if [[ "$PARALLEL" == "true" ]]; then
        {
            BASE_URL="$BASE_URL" bash "$test_script" $test_args 2>&1
            echo $? > "$OUTPUT_DIR/${test_name,,}-exit-code.txt"
        } > "$suite_output" &

        local test_pid=$!
        echo "$test_pid" > "$OUTPUT_DIR/${test_name,,}-pid.txt"

        return 0  # Return immediately for parallel execution
    else
        if BASE_URL="$BASE_URL" bash "$test_script" $test_args > "$suite_output" 2>&1; then
            exit_code=0
        else
            exit_code=$?
        fi
    fi

    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    TOTAL_EXECUTION_TIME=$((TOTAL_EXECUTION_TIME + duration))

    # Process results
    process_test_result "$test_name" "$exit_code" "$duration" "$suite_output"
}

# Process test results
process_test_result() {
    local test_name="$1"
    local exit_code="$2"
    local duration="$3"
    local output_file="$4"

    # Parse test output for statistics
    local total_tests=0
    local passed_tests=0
    local failed_tests=0

    if [[ -f "$output_file" ]]; then
        # Extract test counts from output
        total_tests=$(grep -o "Total tests: [0-9]*" "$output_file" | grep -o "[0-9]*" | tail -1 || echo "0")
        passed_tests=$(grep -o "Passed: [0-9]*" "$output_file" | grep -o "[0-9]*" | tail -1 || echo "0")
        failed_tests=$(grep -o "Failed: [0-9]*" "$output_file" | grep -o "[0-9]*" | tail -1 || echo "0")
    fi

    # Create result JSON
    local result_json="$OUTPUT_DIR/${test_name,,}-result.json"
    cat > "$result_json" <<EOF
{
    "suite_name": "$test_name",
    "exit_code": $exit_code,
    "duration_seconds": $duration,
    "total_tests": $total_tests,
    "passed_tests": $passed_tests,
    "failed_tests": $failed_tests,
    "timestamp": "$(date -Iseconds)",
    "output_file": "$output_file"
}
EOF

    # Update counters
    if [[ $exit_code -eq 0 ]]; then
        ((PASSED_TEST_SUITES++))
        success "$test_name completed successfully ($duration seconds)"
    else
        ((FAILED_TEST_SUITES++))
        error "$test_name failed ($duration seconds)"

        if [[ "$FAIL_FAST" == "true" ]]; then
            error "Fail-fast mode enabled. Stopping tests."
            exit 1
        fi
    fi

    # Show summary for this suite
    if [[ $total_tests -gt 0 ]]; then
        info "  Tests: $passed_tests passed, $failed_tests failed out of $total_tests total"
    fi
}

# Wait for parallel tests to complete
wait_for_parallel_tests() {
    if [[ "$PARALLEL" != "true" ]]; then
        return 0
    fi

    log "Waiting for parallel test suites to complete..."

    # Wait for all test processes
    for pid_file in "$OUTPUT_DIR"/*-pid.txt; do
        if [[ -f "$pid_file" ]]; then
            local pid=$(cat "$pid_file")
            local test_name=$(basename "$pid_file" -pid.txt)

            log "Waiting for $test_name (PID: $pid)..."

            if wait "$pid" 2>/dev/null; then
                log "$test_name completed"
            else
                warning "$test_name may have failed or was already finished"
            fi

            # Get exit code and process results
            local exit_code_file="$OUTPUT_DIR/${test_name}-exit-code.txt"
            local exit_code=1
            if [[ -f "$exit_code_file" ]]; then
                exit_code=$(cat "$exit_code_file")
            fi

            local output_file="$OUTPUT_DIR/${test_name}-output.txt"
            local duration=0  # Duration calculation for parallel tests is approximate

            process_test_result "$test_name" "$exit_code" "$duration" "$output_file"

            # Cleanup
            rm -f "$pid_file" "$exit_code_file"
        fi
    done
}

# Generate reports
generate_reports() {
    local timestamp=$(date -Iseconds)

    case "$REPORT_FORMAT" in
        "json")
            generate_json_report "$timestamp"
            ;;
        "html")
            generate_html_report "$timestamp"
            ;;
        "junit")
            generate_junit_report "$timestamp"
            ;;
        "console"|*)
            generate_console_report "$timestamp"
            ;;
    esac
}

# Generate JSON report
generate_json_report() {
    local timestamp="$1"
    local json_report="$OUTPUT_DIR/test-report.json"

    log "Generating JSON report: $json_report"

    cat > "$json_report" <<EOF
{
    "timestamp": "$timestamp",
    "base_url": "$BASE_URL",
    "test_type": "$TEST_TYPE",
    "total_suites": $TOTAL_TEST_SUITES,
    "passed_suites": $PASSED_TEST_SUITES,
    "failed_suites": $FAILED_TEST_SUITES,
    "total_execution_time": $TOTAL_EXECUTION_TIME,
    "test_suites": [
EOF

    local first=true
    for result_file in "$OUTPUT_DIR"/*-result.json; do
        if [[ -f "$result_file" ]]; then
            if [[ "$first" == "true" ]]; then
                first=false
            else
                echo "," >> "$json_report"
            fi
            cat "$result_file" >> "$json_report"
        fi
    done

    cat >> "$json_report" <<EOF
    ]
}
EOF

    success "JSON report generated: $json_report"
}

# Generate HTML report
generate_html_report() {
    local timestamp="$1"
    local html_report="$OUTPUT_DIR/test-report.html"

    log "Generating HTML report: $html_report"

    cat > "$html_report" <<EOF
<!DOCTYPE html>
<html>
<head>
    <title>RustAxum API Test Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .header { background-color: #f5f5f5; padding: 15px; border-radius: 5px; }
        .success { color: #28a745; }
        .error { color: #dc3545; }
        .warning { color: #ffc107; }
        .suite { margin: 20px 0; padding: 15px; border: 1px solid #ddd; border-radius: 5px; }
        .suite.passed { border-color: #28a745; }
        .suite.failed { border-color: #dc3545; }
        table { width: 100%; border-collapse: collapse; margin: 10px 0; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        th { background-color: #f2f2f2; }
        .output { background-color: #f8f9fa; padding: 10px; border-radius: 3px; font-family: monospace; font-size: 12px; max-height: 200px; overflow-y: auto; }
    </style>
</head>
<body>
    <div class="header">
        <h1>RustAxum API Test Report</h1>
        <p><strong>Generated:</strong> $timestamp</p>
        <p><strong>Base URL:</strong> $BASE_URL</p>
        <p><strong>Test Type:</strong> $TEST_TYPE</p>
    </div>

    <h2>Summary</h2>
    <table>
        <tr><th>Metric</th><th>Value</th></tr>
        <tr><td>Total Test Suites</td><td>$TOTAL_TEST_SUITES</td></tr>
        <tr><td class="success">Passed Suites</td><td>$PASSED_TEST_SUITES</td></tr>
        <tr><td class="error">Failed Suites</td><td>$FAILED_TEST_SUITES</td></tr>
        <tr><td>Total Execution Time</td><td>${TOTAL_EXECUTION_TIME} seconds</td></tr>
    </table>

    <h2>Test Suite Details</h2>
EOF

    # Add details for each test suite
    for result_file in "$OUTPUT_DIR"/*-result.json; do
        if [[ -f "$result_file" && command -v jq >/dev/null 2>&1 ]]; then
            local suite_name=$(jq -r '.suite_name' "$result_file")
            local exit_code=$(jq -r '.exit_code' "$result_file")
            local duration=$(jq -r '.duration_seconds' "$result_file")
            local total_tests=$(jq -r '.total_tests' "$result_file")
            local passed_tests=$(jq -r '.passed_tests' "$result_file")
            local failed_tests=$(jq -r '.failed_tests' "$result_file")
            local output_file=$(jq -r '.output_file' "$result_file")

            local status_class="passed"
            local status_text="PASSED"
            if [[ $exit_code -ne 0 ]]; then
                status_class="failed"
                status_text="FAILED"
            fi

            cat >> "$html_report" <<EOF
    <div class="suite $status_class">
        <h3>$suite_name <span class="$status_class">[$status_text]</span></h3>
        <table>
            <tr><td>Duration</td><td>${duration} seconds</td></tr>
            <tr><td>Total Tests</td><td>$total_tests</td></tr>
            <tr><td>Passed</td><td>$passed_tests</td></tr>
            <tr><td>Failed</td><td>$failed_tests</td></tr>
        </table>

        <h4>Output</h4>
        <div class="output">
EOF

            if [[ -f "$output_file" ]]; then
                # Escape HTML and show last 50 lines
                tail -n 50 "$output_file" | sed 's/&/\&amp;/g; s/</\&lt;/g; s/>/\&gt;/g' >> "$html_report"
            fi

            echo "        </div>" >> "$html_report"
            echo "    </div>" >> "$html_report"
        fi
    done

    cat >> "$html_report" <<EOF
</body>
</html>
EOF

    success "HTML report generated: $html_report"
}

# Generate JUnit XML report
generate_junit_report() {
    local timestamp="$1"
    local junit_report="$OUTPUT_DIR/junit-report.xml"

    log "Generating JUnit report: $junit_report"

    cat > "$junit_report" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<testsuites tests="$TOTAL_TEST_SUITES" failures="$FAILED_TEST_SUITES" time="$TOTAL_EXECUTION_TIME" timestamp="$timestamp">
EOF

    for result_file in "$OUTPUT_DIR"/*-result.json; do
        if [[ -f "$result_file" && command -v jq >/dev/null 2>&1 ]]; then
            local suite_name=$(jq -r '.suite_name' "$result_file")
            local exit_code=$(jq -r '.exit_code' "$result_file")
            local duration=$(jq -r '.duration_seconds' "$result_file")
            local total_tests=$(jq -r '.total_tests' "$result_file")
            local failed_tests=$(jq -r '.failed_tests' "$result_file")
            local output_file=$(jq -r '.output_file' "$result_file")

            cat >> "$junit_report" <<EOF
    <testsuite name="$suite_name" tests="$total_tests" failures="$failed_tests" time="$duration">
        <testcase name="$suite_name" time="$duration">
EOF

            if [[ $exit_code -ne 0 ]]; then
                cat >> "$junit_report" <<EOF
            <failure message="Test suite failed with exit code $exit_code">
EOF
                if [[ -f "$output_file" ]]; then
                    tail -n 20 "$output_file" | sed 's/&/\&amp;/g; s/</\&lt;/g; s/>/\&gt;/g' >> "$junit_report"
                fi
                echo "            </failure>" >> "$junit_report"
            fi

            cat >> "$junit_report" <<EOF
        </testcase>
    </testsuite>
EOF
        fi
    done

    echo "</testsuites>" >> "$junit_report"

    success "JUnit report generated: $junit_report"
}

# Generate console report
generate_console_report() {
    local timestamp="$1"

    header "Test Execution Summary"

    echo -e "Timestamp: ${CYAN}$timestamp${NC}"
    echo -e "Base URL: ${CYAN}$BASE_URL${NC}"
    echo -e "Test Type: ${CYAN}$TEST_TYPE${NC}"
    echo -e "Output Directory: ${CYAN}$OUTPUT_DIR${NC}"
    echo

    echo -e "Total Test Suites: ${BOLD}$TOTAL_TEST_SUITES${NC}"
    echo -e "Passed: ${GREEN}$PASSED_TEST_SUITES${NC}"
    echo -e "Failed: ${RED}$FAILED_TEST_SUITES${NC}"
    echo -e "Total Execution Time: ${BOLD}${TOTAL_EXECUTION_TIME} seconds${NC}"

    if [[ $FAILED_TEST_SUITES -gt 0 ]]; then
        echo
        warning "Failed test suites:"
        for result_file in "$OUTPUT_DIR"/*-result.json; do
            if [[ -f "$result_file" && command -v jq >/dev/null 2>&1 ]]; then
                local suite_name=$(jq -r '.suite_name' "$result_file")
                local exit_code=$(jq -r '.exit_code' "$result_file")
                if [[ $exit_code -ne 0 ]]; then
                    echo -e "  ${RED}✗${NC} $suite_name"
                fi
            fi
        done
    fi

    echo
    info "Test output files are available in: $OUTPUT_DIR"
}

# Main test execution
main() {
    header "RustAxum cURL Test Runner"

    log "Starting test execution..."
    log "Base URL: $BASE_URL"
    log "Test Type: $TEST_TYPE"
    log "Report Format: $REPORT_FORMAT"
    log "Output Directory: $OUTPUT_DIR"
    log "Parallel Execution: $PARALLEL"

    # Check prerequisites
    check_prerequisites

    # Determine which tests to run
    local test_dir="$(dirname "${BASH_SOURCE[0]}")"

    case "$TEST_TYPE" in
        "auth")
            run_test_suite "Authentication Tests" "$test_dir/auth-tests.sh"
            ;;
        "me")
            run_test_suite "ME Endpoint Tests" "$test_dir/me-endpoint-tests.sh"
            ;;
        "comprehensive")
            run_test_suite "Comprehensive API Tests" "$test_dir/api-comprehensive-tests.sh" "--mode full"
            ;;
        "smoke")
            run_test_suite "Smoke Tests" "$test_dir/api-comprehensive-tests.sh" "--mode smoke"
            ;;
        "endpoints")
            run_test_suite "Endpoint Tests" "$test_dir/api-comprehensive-tests.sh" "--mode endpoints"
            ;;
        "all"|*)
            log "Running all test suites..."

            if [[ "$PARALLEL" == "true" ]]; then
                run_test_suite "Authentication Tests" "$test_dir/auth-tests.sh"
                run_test_suite "ME Endpoint Tests" "$test_dir/me-endpoint-tests.sh"
                run_test_suite "Comprehensive API Tests" "$test_dir/api-comprehensive-tests.sh" "--mode full"
                wait_for_parallel_tests
            else
                run_test_suite "Authentication Tests" "$test_dir/auth-tests.sh"
                run_test_suite "ME Endpoint Tests" "$test_dir/me-endpoint-tests.sh"
                run_test_suite "Comprehensive API Tests" "$test_dir/api-comprehensive-tests.sh" "--mode full"
            fi
            ;;
    esac

    # Generate reports
    generate_reports

    # Final result
    if [[ $FAILED_TEST_SUITES -eq 0 ]]; then
        header "🎉 All Tests Passed!"
        exit 0
    else
        header "❌ Some Tests Failed"
        exit 1
    fi
}

# Show help
show_help() {
    cat <<EOF
RustAxum cURL Test Runner

Usage: $0 [options]

Options:
    -h, --help              Show this help message
    -t, --type TYPE         Test type: all, auth, me, comprehensive, smoke, endpoints (default: all)
    -u, --url URL           Base URL (default: http://localhost:3000)
    -r, --report FORMAT     Report format: console, json, html, junit (default: console)
    -o, --output DIR        Output directory (default: ./test-results)
    -p, --parallel          Run test suites in parallel
    -f, --fail-fast         Stop on first test suite failure

Environment Variables:
    BASE_URL               API base URL
    TEST_TYPE              Test type
    REPORT_FORMAT          Report format
    OUTPUT_DIR             Output directory
    PARALLEL               Run in parallel (true/false)
    FAIL_FAST              Fail fast mode (true/false)

Examples:
    $0                                      # Run all tests
    $0 --type smoke                         # Run smoke tests only
    $0 --type me --report html              # Run /me tests with HTML report
    $0 --url http://staging.com --parallel  # Test staging with parallel execution
    $0 --report junit --output ./ci-results # Generate JUnit report for CI

Test Types:
    all             Run all test suites (default)
    auth            Authentication flow tests only
    me              /me endpoint tests only
    comprehensive   Full API test suite
    smoke           Quick smoke tests
    endpoints       Endpoint availability tests
EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -t|--type)
            TEST_TYPE="$2"
            shift 2
            ;;
        -u|--url)
            BASE_URL="$2"
            shift 2
            ;;
        -r|--report)
            REPORT_FORMAT="$2"
            shift 2
            ;;
        -o|--output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        -p|--parallel)
            PARALLEL="true"
            shift
            ;;
        -f|--fail-fast)
            FAIL_FAST="true"
            shift
            ;;
        *)
            echo "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Run main function
main "$@"