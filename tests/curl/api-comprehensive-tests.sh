#!/bin/bash

# RustAxum Comprehensive API Tests using cURL
# Full API testing suite covering multiple endpoints and scenarios

set -e  # Exit on any error

# Configuration
BASE_URL="${BASE_URL:-http://localhost:3000}"
TEMP_DIR="/tmp/rustaxum-comprehensive-tests"
TEST_MODE="${TEST_MODE:-full}"  # full, smoke, endpoints
PARALLEL_REQUESTS="${PARALLEL_REQUESTS:-5}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0

# Performance tracking
TOTAL_RESPONSE_TIME=0
TOTAL_SUCCESSFUL_REQUESTS=0

# Create temp directory
mkdir -p "$TEMP_DIR"

# Logging functions
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

success() {
    echo -e "${GREEN}✓${NC} $1"
    ((PASSED_TESTS++))
}

error() {
    echo -e "${RED}✗${NC} $1"
    ((FAILED_TESTS++))
}

warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

info() {
    echo -e "${PURPLE}ℹ${NC} $1"
}

skip() {
    echo -e "${CYAN}→${NC} $1"
    ((SKIPPED_TESTS++))
}

# Test helper function with performance tracking
run_test() {
    local test_name="$1"
    local expected_status="$2"
    local curl_args="${@:3}"

    ((TOTAL_TESTS++))

    if [[ "$TEST_MODE" == "smoke" && $TOTAL_TESTS -gt 10 ]]; then
        skip "Skipping $test_name (smoke test mode)"
        return 0
    fi

    log "Running test: $test_name"

    # Run curl and capture response
    local response_file="$TEMP_DIR/response_$TOTAL_TESTS.json"
    local headers_file="$TEMP_DIR/headers_$TOTAL_TESTS.txt"

    local start_time=$(date +%s%N)

    if HTTP_STATUS=$(curl -s -w "%{http_code}" -D "$headers_file" -o "$response_file" \
        --connect-timeout 10 --max-time 30 \
        $curl_args 2>/dev/null); then

        local end_time=$(date +%s%N)
        local duration=$(( (end_time - start_time) / 1000000 ))  # Convert to milliseconds

        if [[ "$HTTP_STATUS" == "$expected_status" ]]; then
            success "$test_name (Status: $HTTP_STATUS, Time: ${duration}ms)"

            # Track performance
            TOTAL_RESPONSE_TIME=$((TOTAL_RESPONSE_TIME + duration))
            ((TOTAL_SUCCESSFUL_REQUESTS++))

            return 0
        else
            error "$test_name (Expected: $expected_status, Got: $HTTP_STATUS, Time: ${duration}ms)"
            if [[ -f "$response_file" && -s "$response_file" ]]; then
                echo "Response body:"
                head -n 10 "$response_file" | cat
                echo
            fi
            return 1
        fi
    else
        error "$test_name (Connection failed)"
        return 1
    fi
}

# JSON parsing helper
get_json_value() {
    local json_file="$1"
    local key="$2"

    if command -v jq >/dev/null 2>&1; then
        jq -r ".$key // empty" "$json_file" 2>/dev/null
    else
        # Fallback without jq
        grep -o "\"$key\":\"[^\"]*\"" "$json_file" | cut -d'"' -f4 | head -1
    fi
}

# Setup test environment
setup_test_environment() {
    log "Setting up test environment..."

    # Health check
    log "Performing health check..."
    if ! run_test "Health Check" "200" "$BASE_URL/api/docs" >/dev/null 2>&1; then
        error "API health check failed. Is the server running at $BASE_URL?"
        exit 1
    fi

    # Create multiple test users for different scenarios
    local users_created=0

    for i in {1..3}; do
        local timestamp=$(date +%s)
        local test_email="comprehensive_test_${timestamp}_${i}@example.com"
        local test_name="Comprehensive Test User $i"
        local test_password="testpass123"

        local reg_payload=$(cat <<EOF
{
    "name": "$test_name",
    "email": "$test_email",
    "password": "$test_password"
}
EOF
)

        if HTTP_STATUS=$(curl -s -w "%{http_code}" \
            -o "$TEMP_DIR/setup_user_$i.json" \
            -X POST \
            -H "Content-Type: application/json" \
            -d "$reg_payload" \
            "$BASE_URL/api/auth/register" 2>/dev/null); then

            if [[ "$HTTP_STATUS" == "201" ]]; then
                local access_token=$(get_json_value "$TEMP_DIR/setup_user_$i.json" "access_token")
                local refresh_token=$(get_json_value "$TEMP_DIR/setup_user_$i.json" "refresh_token")
                local user_id=$(get_json_value "$TEMP_DIR/setup_user_$i.json" "user.id")

                if [[ -n "$access_token" && -n "$refresh_token" ]]; then
                    echo "$access_token" > "$TEMP_DIR/access_token_$i.txt"
                    echo "$refresh_token" > "$TEMP_DIR/refresh_token_$i.txt"
                    echo "$test_email" > "$TEMP_DIR/test_email_$i.txt"
                    echo "$user_id" > "$TEMP_DIR/user_id_$i.txt"
                    ((users_created++))
                fi
            fi
        fi

        sleep 0.1  # Small delay between user creations
    done

    success "Test environment setup completed ($users_created users created)"
    return 0
}

# Test authentication endpoints
test_authentication() {
    log "=== Testing Authentication Endpoints ==="

    if [[ ! -f "$TEMP_DIR/test_email_1.txt" ]]; then
        error "No test users available for authentication testing"
        return 1
    fi

    local test_email=$(cat "$TEMP_DIR/test_email_1.txt")
    local test_password="testpass123"

    # Valid login
    local login_payload=$(cat <<EOF
{
    "email": "$test_email",
    "password": "$test_password"
}
EOF
)

    run_test "POST /api/auth/login (valid credentials)" "200" \
        -X POST \
        -H "Content-Type: application/json" \
        -d "$login_payload" \
        "$BASE_URL/api/auth/login"

    # Invalid login
    local invalid_login_payload=$(cat <<EOF
{
    "email": "$test_email",
    "password": "wrongpassword"
}
EOF
)

    run_test "POST /api/auth/login (invalid credentials)" "401" \
        -X POST \
        -H "Content-Type: application/json" \
        -d "$invalid_login_payload" \
        "$BASE_URL/api/auth/login"

    # Test refresh token
    if [[ -f "$TEMP_DIR/refresh_token_1.txt" ]]; then
        local refresh_token=$(cat "$TEMP_DIR/refresh_token_1.txt")
        local refresh_payload="{\"refresh_token\": \"$refresh_token\"}"

        run_test "POST /api/auth/refresh-token" "200" \
            -X POST \
            -H "Content-Type: application/json" \
            -d "$refresh_payload" \
            "$BASE_URL/api/auth/refresh-token"
    fi

    # Test /me endpoint
    if [[ -f "$TEMP_DIR/access_token_1.txt" ]]; then
        local access_token=$(cat "$TEMP_DIR/access_token_1.txt")

        run_test "GET /api/me (authenticated)" "200" \
            -X GET \
            -H "Authorization: Bearer $access_token" \
            -H "Content-Type: application/json" \
            "$BASE_URL/api/me"
    fi

    # Test /me without authentication
    run_test "GET /api/me (unauthenticated)" "401" \
        -X GET \
        -H "Content-Type: application/json" \
        "$BASE_URL/api/me"
}

# Test protected endpoints
test_protected_endpoints() {
    log "=== Testing Protected Endpoints ==="

    if [[ ! -f "$TEMP_DIR/access_token_1.txt" ]]; then
        error "No access token available for protected endpoint testing"
        return 1
    fi

    local access_token=$(cat "$TEMP_DIR/access_token_1.txt")
    local auth_headers="-H \"Authorization: Bearer $access_token\" -H \"Content-Type: application/json\""

    # Define endpoints to test
    local endpoints=(
        "GET /api/users"
        "GET /api/countries"
        "GET /api/provinces"
        "GET /api/cities"
        "GET /api/roles"
        "GET /api/permissions"
        "GET /api/user-organizations"
        "GET /api/organization-position-levels"
        "GET /api/organization-positions"
        "GET /api/activity-logs"
    )

    for endpoint_def in "${endpoints[@]}"; do
        local method=$(echo "$endpoint_def" | cut -d' ' -f1)
        local path=$(echo "$endpoint_def" | cut -d' ' -f2)

        if [[ "$method" == "GET" ]]; then
            run_test "$endpoint_def (authenticated)" "200" \
                -X GET \
                -H "Authorization: Bearer $access_token" \
                -H "Content-Type: application/json" \
                "$BASE_URL$path"

            # Test same endpoint without authentication
            run_test "$endpoint_def (unauthenticated)" "401" \
                -X GET \
                -H "Content-Type: application/json" \
                "$BASE_URL$path"
        fi
    done
}

# Test OAuth endpoints
test_oauth_endpoints() {
    log "=== Testing OAuth Endpoints ==="

    # Test OAuth client endpoints (these might require special auth)
    run_test "GET /oauth/clients (no auth)" "401" \
        -X GET \
        -H "Content-Type: application/json" \
        "$BASE_URL/oauth/clients" || skip "OAuth clients endpoint not accessible"

    # Test device authorization endpoints
    run_test "GET /oauth/device (device verification page)" "200" \
        -X GET \
        "$BASE_URL/oauth/device" || skip "OAuth device page not accessible"

    # Test introspection endpoint
    run_test "POST /oauth/introspect (no token)" "400" \
        -X POST \
        -H "Content-Type: application/json" \
        -d '{}' \
        "$BASE_URL/oauth/introspect" || skip "OAuth introspect endpoint not accessible"
}

# Test error handling
test_error_handling() {
    log "=== Testing Error Handling ==="

    # 404 errors
    run_test "GET /api/nonexistent (404)" "404" \
        -X GET \
        -H "Content-Type: application/json" \
        "$BASE_URL/api/nonexistent"

    run_test "GET /completely/invalid/path (404)" "404" \
        -X GET \
        -H "Content-Type: application/json" \
        "$BASE_URL/completely/invalid/path"

    # Malformed JSON
    run_test "POST /api/auth/login (malformed JSON)" "400" \
        -X POST \
        -H "Content-Type: application/json" \
        -d '{"invalid": json}' \
        "$BASE_URL/api/auth/login"

    # Invalid Content-Type
    run_test "POST /api/auth/login (wrong content-type)" "400" \
        -X POST \
        -H "Content-Type: text/plain" \
        -d '{"email": "test@example.com", "password": "password"}' \
        "$BASE_URL/api/auth/login" || skip "Content-type validation may not be strict"

    # Very large request body
    local large_payload=$(printf '{"data": "%*s"}' 10000 | tr ' ' 'a')
    run_test "POST /api/auth/login (large payload)" "400" \
        -X POST \
        -H "Content-Type: application/json" \
        -d "$large_payload" \
        "$BASE_URL/api/auth/login" || skip "Large payload handling may vary"
}

# Test public endpoints
test_public_endpoints() {
    log "=== Testing Public Endpoints ==="

    # Documentation endpoints
    run_test "GET /api/docs" "200" \
        -X GET \
        "$BASE_URL/api/docs"

    run_test "GET /api/docs/openapi.json" "200" \
        -X GET \
        -H "Accept: application/json" \
        "$BASE_URL/api/docs/openapi.json"

    run_test "GET /api/docs/openapi.yaml" "200" \
        -X GET \
        -H "Accept: application/yaml" \
        "$BASE_URL/api/docs/openapi.yaml"

    # Web push VAPID key (public endpoint)
    run_test "GET /api/web-push/vapid-public-key" "200" \
        -X GET \
        -H "Content-Type: application/json" \
        "$BASE_URL/api/web-push/vapid-public-key" || skip "Web push not configured"
}

# Test HTTP methods and CORS
test_http_methods_and_cors() {
    log "=== Testing HTTP Methods and CORS ==="

    # OPTIONS requests for CORS
    run_test "OPTIONS /api/auth/login (CORS preflight)" "200" \
        -X OPTIONS \
        -H "Origin: http://localhost:3001" \
        -H "Access-Control-Request-Method: POST" \
        -H "Access-Control-Request-Headers: Content-Type" \
        "$BASE_URL/api/auth/login" || skip "CORS may not be configured"

    # HEAD requests
    run_test "HEAD /api/docs" "200" \
        -X HEAD \
        "$BASE_URL/api/docs" || skip "HEAD method may not be supported"

    # Invalid methods on specific endpoints
    run_test "DELETE /api/docs (method not allowed)" "405" \
        -X DELETE \
        "$BASE_URL/api/docs"

    run_test "PUT /api/docs (method not allowed)" "405" \
        -X PUT \
        "$BASE_URL/api/docs"
}

# Test concurrent requests
test_concurrent_requests() {
    log "=== Testing Concurrent Requests ==="

    if [[ ! -f "$TEMP_DIR/access_token_1.txt" ]]; then
        warning "No access token available for concurrent testing"
        return 0
    fi

    local access_token=$(cat "$TEMP_DIR/access_token_1.txt")

    log "Running $PARALLEL_REQUESTS concurrent /me requests..."

    # Create background processes for concurrent requests
    local pids=()
    for i in $(seq 1 $PARALLEL_REQUESTS); do
        (
            curl -s -w "%{http_code}" \
                -o "$TEMP_DIR/concurrent_$i.json" \
                -X GET \
                -H "Authorization: Bearer $access_token" \
                -H "Content-Type: application/json" \
                "$BASE_URL/api/me" > "$TEMP_DIR/concurrent_status_$i.txt" 2>/dev/null
        ) &
        pids+=($!)
    done

    # Wait for all requests to complete
    for pid in "${pids[@]}"; do
        wait $pid
    done

    # Check results
    local concurrent_success=0
    for i in $(seq 1 $PARALLEL_REQUESTS); do
        if [[ -f "$TEMP_DIR/concurrent_status_$i.txt" ]]; then
            local status=$(cat "$TEMP_DIR/concurrent_status_$i.txt")
            if [[ "$status" == "200" ]]; then
                ((concurrent_success++))
            fi
        fi
    done

    if [[ $concurrent_success -eq $PARALLEL_REQUESTS ]]; then
        success "All $PARALLEL_REQUESTS concurrent requests succeeded"
    else
        warning "Only $concurrent_success out of $PARALLEL_REQUESTS concurrent requests succeeded"
    fi
}

# Test performance and limits
test_performance() {
    log "=== Testing Performance ==="

    if [[ "$TEST_MODE" == "smoke" ]]; then
        skip "Skipping performance tests in smoke mode"
        return 0
    fi

    # Sequential request performance
    log "Testing sequential request performance (10 requests)..."

    local total_time=0
    local successful_requests=0

    for i in {1..10}; do
        local start_time=$(date +%s%N)

        if HTTP_STATUS=$(curl -s -w "%{http_code}" \
            -o "$TEMP_DIR/perf_$i.json" \
            -X GET \
            "$BASE_URL/api/docs" 2>/dev/null); then

            local end_time=$(date +%s%N)
            local duration=$(( (end_time - start_time) / 1000000 ))

            if [[ "$HTTP_STATUS" == "200" ]]; then
                total_time=$((total_time + duration))
                ((successful_requests++))
            fi
        fi
        sleep 0.1
    done

    if [[ $successful_requests -gt 0 ]]; then
        local average_time=$((total_time / successful_requests))
        info "Sequential performance: $successful_requests/10 successful, ${average_time}ms average"
    fi

    # Test with different payload sizes
    for size in 1 100 1000; do
        local payload=$(printf '{"data": "%*s"}' $size | tr ' ' 'a')
        local start_time=$(date +%s%N)

        curl -s -w "%{http_code}" \
            -o "$TEMP_DIR/size_test_$size.json" \
            -X POST \
            -H "Content-Type: application/json" \
            -d "$payload" \
            "$BASE_URL/api/auth/login" >/dev/null 2>&1

        local end_time=$(date +%s%N)
        local duration=$(( (end_time - start_time) / 1000000 ))
        info "Payload size ${size}B: ${duration}ms"
    done
}

# Main test execution
main() {
    log "🚀 Starting RustAxum Comprehensive API Tests"
    log "Target: $BASE_URL"
    log "Mode: $TEST_MODE"
    log "Temp directory: $TEMP_DIR"
    echo

    # Setup
    if ! setup_test_environment; then
        error "Failed to setup test environment"
        exit 1
    fi
    echo

    # Run test suites based on mode
    case "$TEST_MODE" in
        "smoke")
            log "Running smoke tests (basic functionality only)..."
            test_authentication
            test_public_endpoints
            ;;
        "endpoints")
            log "Running endpoint tests..."
            test_authentication
            test_protected_endpoints
            test_oauth_endpoints
            test_public_endpoints
            test_error_handling
            ;;
        "full"|*)
            log "Running full comprehensive tests..."
            test_authentication
            echo
            test_protected_endpoints
            echo
            test_oauth_endpoints
            echo
            test_public_endpoints
            echo
            test_error_handling
            echo
            test_http_methods_and_cors
            echo
            test_concurrent_requests
            echo
            test_performance
            ;;
    esac

    echo

    # Calculate performance statistics
    local avg_response_time=0
    if [[ $TOTAL_SUCCESSFUL_REQUESTS -gt 0 ]]; then
        avg_response_time=$((TOTAL_RESPONSE_TIME / TOTAL_SUCCESSFUL_REQUESTS))
    fi

    # Summary
    log "=== Comprehensive Test Summary ==="
    echo -e "Test Mode: ${CYAN}$TEST_MODE${NC}"
    echo -e "Total tests: ${BLUE}$TOTAL_TESTS${NC}"
    echo -e "Passed: ${GREEN}$PASSED_TESTS${NC}"
    echo -e "Failed: ${RED}$FAILED_TESTS${NC}"
    echo -e "Skipped: ${CYAN}$SKIPPED_TESTS${NC}"
    echo -e "Average response time: ${PURPLE}${avg_response_time}ms${NC}"

    # Performance assessment
    if [[ $avg_response_time -lt 100 ]]; then
        echo -e "Performance: ${GREEN}Excellent${NC} (< 100ms)"
    elif [[ $avg_response_time -lt 300 ]]; then
        echo -e "Performance: ${GREEN}Good${NC} (< 300ms)"
    elif [[ $avg_response_time -lt 500 ]]; then
        echo -e "Performance: ${YELLOW}Acceptable${NC} (< 500ms)"
    else
        echo -e "Performance: ${RED}Needs attention${NC} (> 500ms)"
    fi

    if [[ $FAILED_TESTS -eq 0 ]]; then
        echo -e "\n${GREEN}🎉 All comprehensive API tests passed!${NC}"
        exit 0
    else
        echo -e "\n${RED}❌ Some comprehensive API tests failed.${NC}"
        exit 1
    fi
}

# Cleanup function
cleanup() {
    log "Cleaning up temporary files..."
    rm -rf "$TEMP_DIR"
}

# Set up cleanup trap
trap cleanup EXIT

# Help function
show_help() {
    echo "RustAxum Comprehensive API Tests"
    echo
    echo "Usage: $0 [options]"
    echo
    echo "Options:"
    echo "  -h, --help              Show this help message"
    echo "  -m, --mode MODE         Test mode: full, smoke, endpoints (default: full)"
    echo "  -u, --url URL           Base URL (default: http://localhost:3000)"
    echo "  -p, --parallel NUM      Number of parallel requests (default: 5)"
    echo
    echo "Environment Variables:"
    echo "  BASE_URL               API base URL"
    echo "  TEST_MODE              Test mode"
    echo "  PARALLEL_REQUESTS      Number of parallel requests"
    echo
    echo "Examples:"
    echo "  $0                                    # Run full tests"
    echo "  $0 --mode smoke                       # Run smoke tests only"
    echo "  $0 --url http://localhost:8080        # Test different URL"
    echo "  BASE_URL=http://staging.com $0        # Use environment variable"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -m|--mode)
            TEST_MODE="$2"
            shift 2
            ;;
        -u|--url)
            BASE_URL="$2"
            shift 2
            ;;
        -p|--parallel)
            PARALLEL_REQUESTS="$2"
            shift 2
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