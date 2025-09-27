#!/bin/bash

# RustAxum /me Endpoint Tests using cURL
# Focused testing of the /api/me endpoint for authenticated user data retrieval

set -e  # Exit on any error

# Configuration
BASE_URL="${BASE_URL:-http://localhost:3000}"
TEMP_DIR="/tmp/rustaxum-me-tests"
RESULTS_FILE="$TEMP_DIR/me-test-results.json"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Create temp directory
mkdir -p "$TEMP_DIR"

# Logging function
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

# Test helper function
run_test() {
    local test_name="$1"
    local expected_status="$2"
    local curl_args="${@:3}"

    ((TOTAL_TESTS++))
    log "Running test: $test_name"

    # Run curl and capture response
    local response_file="$TEMP_DIR/response_$TOTAL_TESTS.json"
    local headers_file="$TEMP_DIR/headers_$TOTAL_TESTS.txt"
    local timing_file="$TEMP_DIR/timing_$TOTAL_TESTS.txt"

    # Capture timing information
    local start_time=$(date +%s%N)

    if HTTP_STATUS=$(curl -s -w "%{http_code}" -D "$headers_file" -o "$response_file" \
        -w "time_total: %{time_total}\ntime_connect: %{time_connect}\ntime_appconnect: %{time_appconnect}\ntime_pretransfer: %{time_pretransfer}\ntime_starttransfer: %{time_starttransfer}\nsize_download: %{size_download}\nspeed_download: %{speed_download}" \
        $curl_args > "$timing_file" 2>&1); then

        local end_time=$(date +%s%N)
        local duration=$(( (end_time - start_time) / 1000000 ))  # Convert to milliseconds

        if [[ "$HTTP_STATUS" == "$expected_status" ]]; then
            success "$test_name (Status: $HTTP_STATUS, Time: ${duration}ms)"

            # Log timing details for successful tests
            if [[ -f "$timing_file" ]]; then
                local total_time=$(grep "time_total:" "$timing_file" | cut -d' ' -f2)
                info "  Response time: ${total_time}s"
            fi

            return 0
        else
            error "$test_name (Expected: $expected_status, Got: $HTTP_STATUS, Time: ${duration}ms)"
            if [[ -f "$response_file" ]]; then
                echo "Response body:"
                cat "$response_file" | jq . 2>/dev/null || cat "$response_file"
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
        # Fallback without jq (basic parsing)
        grep -o "\"$key\":\"[^\"]*\"" "$json_file" | cut -d'"' -f4
    fi
}

# Validate JSON response structure
validate_me_response() {
    local response_file="$1"
    local expected_email="$2"

    if ! command -v jq >/dev/null 2>&1; then
        warning "jq not available, skipping detailed response validation"
        return 0
    fi

    local user_object=$(jq '.user' "$response_file" 2>/dev/null)
    if [[ "$user_object" == "null" ]]; then
        error "Response missing 'user' object"
        return 1
    fi

    local user_id=$(jq -r '.user.id // empty' "$response_file" 2>/dev/null)
    local user_name=$(jq -r '.user.name // empty' "$response_file" 2>/dev/null)
    local user_email=$(jq -r '.user.email // empty' "$response_file" 2>/dev/null)
    local created_at=$(jq -r '.user.created_at // empty' "$response_file" 2>/dev/null)
    local updated_at=$(jq -r '.user.updated_at // empty' "$response_file" 2>/dev/null)

    local validation_passed=true

    if [[ -z "$user_id" ]]; then
        error "  Missing user.id"
        validation_passed=false
    else
        success "  user.id: $user_id"
    fi

    if [[ -z "$user_name" ]]; then
        error "  Missing user.name"
        validation_passed=false
    else
        success "  user.name: $user_name"
    fi

    if [[ -z "$user_email" ]]; then
        error "  Missing user.email"
        validation_passed=false
    elif [[ "$expected_email" != "" && "$user_email" != "$expected_email" ]]; then
        error "  user.email mismatch (expected: $expected_email, got: $user_email)"
        validation_passed=false
    else
        success "  user.email: $user_email"
    fi

    if [[ -z "$created_at" ]]; then
        error "  Missing user.created_at"
        validation_passed=false
    else
        success "  user.created_at: $created_at"
    fi

    if [[ -z "$updated_at" ]]; then
        error "  Missing user.updated_at"
        validation_passed=false
    else
        success "  user.updated_at: $updated_at"
    fi

    # Check for sensitive fields that should NOT be present
    local password_field=$(jq -r '.user.password // empty' "$response_file" 2>/dev/null)
    if [[ -n "$password_field" ]]; then
        error "  Security issue: password field exposed in response"
        validation_passed=false
    else
        success "  Security: password field not exposed"
    fi

    if [[ "$validation_passed" == "true" ]]; then
        success "Response structure validation passed"
        return 0
    else
        error "Response structure validation failed"
        return 1
    fi
}

# Setup test user
setup_test_user() {
    log "Setting up test user for /me endpoint tests..."

    local timestamp=$(date +%s)
    local random=$(shuf -i 1000-9999 -n 1)
    local test_email="me_test_${timestamp}_${random}@example.com"
    local test_name="ME Test User"
    local test_password="metest123"

    # Register test user
    local reg_payload=$(cat <<EOF
{
    "name": "$test_name",
    "email": "$test_email",
    "password": "$test_password"
}
EOF
)

    log "Creating test user: $test_email"
    if HTTP_STATUS=$(curl -s -w "%{http_code}" \
        -o "$TEMP_DIR/setup_response.json" \
        -X POST \
        -H "Content-Type: application/json" \
        -d "$reg_payload" \
        "$BASE_URL/api/auth/register"); then

        if [[ "$HTTP_STATUS" == "201" ]]; then
            local access_token=$(get_json_value "$TEMP_DIR/setup_response.json" "access_token")
            local refresh_token=$(get_json_value "$TEMP_DIR/setup_response.json" "refresh_token")
            local user_id=$(get_json_value "$TEMP_DIR/setup_response.json" "user.id")

            if [[ -n "$access_token" && -n "$refresh_token" && -n "$user_id" ]]; then
                echo "$access_token" > "$TEMP_DIR/access_token.txt"
                echo "$refresh_token" > "$TEMP_DIR/refresh_token.txt"
                echo "$test_email" > "$TEMP_DIR/test_email.txt"
                echo "$test_name" > "$TEMP_DIR/test_name.txt"
                echo "$user_id" > "$TEMP_DIR/user_id.txt"

                success "Test user created successfully"
                info "  Email: $test_email"
                info "  User ID: $user_id"
                return 0
            else
                error "Registration response missing required fields"
                return 1
            fi
        else
            error "Failed to create test user (Status: $HTTP_STATUS)"
            cat "$TEMP_DIR/setup_response.json"
            return 1
        fi
    else
        error "Connection failed during test user setup"
        return 1
    fi
}

# Health check
health_check() {
    log "Performing health check..."
    if run_test "Health Check" "200" "$BASE_URL/api/docs" >/dev/null 2>&1; then
        success "API is accessible at $BASE_URL"
    else
        error "API health check failed. Is the server running at $BASE_URL?"
        exit 1
    fi
}

# Test /me endpoint with valid authentication
test_valid_authentication() {
    log "=== Testing /me Endpoint with Valid Authentication ==="

    if [[ ! -f "$TEMP_DIR/access_token.txt" ]]; then
        error "No access token available for testing"
        return 1
    fi

    local access_token=$(cat "$TEMP_DIR/access_token.txt")
    local expected_email=$(cat "$TEMP_DIR/test_email.txt")

    # Test basic /me endpoint functionality
    if run_test "GET /me with valid token" "200" \
        -X GET \
        -H "Authorization: Bearer $access_token" \
        -H "Content-Type: application/json" \
        "$BASE_URL/api/me"; then

        # Validate response structure
        local response_file="$TEMP_DIR/response_$TOTAL_TESTS.json"
        validate_me_response "$response_file" "$expected_email"
    fi

    # Test multiple rapid requests to check for consistency
    log "Testing multiple rapid /me requests..."
    for i in {1..3}; do
        run_test "Rapid /me request $i" "200" \
            -X GET \
            -H "Authorization: Bearer $access_token" \
            -H "Content-Type: application/json" \
            "$BASE_URL/api/me" >/dev/null 2>&1
        sleep 0.1
    done

    # Test with different HTTP headers
    run_test "/me with Accept: application/json" "200" \
        -X GET \
        -H "Authorization: Bearer $access_token" \
        -H "Content-Type: application/json" \
        -H "Accept: application/json" \
        "$BASE_URL/api/me"

    # Test with User-Agent header
    run_test "/me with custom User-Agent" "200" \
        -X GET \
        -H "Authorization: Bearer $access_token" \
        -H "Content-Type: application/json" \
        -H "User-Agent: RustAxum-curl-tests/1.0" \
        "$BASE_URL/api/me"
}

# Test /me endpoint with invalid authentication
test_invalid_authentication() {
    log "=== Testing /me Endpoint with Invalid Authentication ==="

    # No Authorization header
    run_test "/me without Authorization header" "401" \
        -X GET \
        -H "Content-Type: application/json" \
        "$BASE_URL/api/me"

    # Invalid Bearer token format
    run_test "/me with invalid Bearer token" "401" \
        -X GET \
        -H "Authorization: Bearer invalid.jwt.token" \
        -H "Content-Type: application/json" \
        "$BASE_URL/api/me"

    # Malformed Bearer token
    run_test "/me with malformed Bearer token" "401" \
        -X GET \
        -H "Authorization: Bearer malformed-token" \
        -H "Content-Type: application/json" \
        "$BASE_URL/api/me"

    # Empty Bearer token
    run_test "/me with empty Bearer token" "401" \
        -X GET \
        -H "Authorization: Bearer " \
        -H "Content-Type: application/json" \
        "$BASE_URL/api/me"

    # Wrong authentication method (Basic instead of Bearer)
    run_test "/me with Basic auth instead of Bearer" "401" \
        -X GET \
        -H "Authorization: Basic dGVzdDp0ZXN0" \
        -H "Content-Type: application/json" \
        "$BASE_URL/api/me"

    # Just "Authorization" header without value
    run_test "/me with empty Authorization header" "401" \
        -X GET \
        -H "Authorization: " \
        -H "Content-Type: application/json" \
        "$BASE_URL/api/me"

    # Expired token (mock expired token)
    local expired_token="eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIwMUs2NTZQUFEwRFc3Q0NXRTBGQ1RWM1dCOSIsImV4cCI6MTU5MDUwMDAwMCwiaWF0IjoxNTkwNDAwMDAwLCJqdGkiOiIwMUs2NTdHMVZDQjJDUzVQTVoyRVBLREU3RCJ9.fake-signature"

    run_test "/me with expired token" "401" \
        -X GET \
        -H "Authorization: Bearer $expired_token" \
        -H "Content-Type: application/json" \
        "$BASE_URL/api/me"
}

# Test HTTP methods not allowed for /me endpoint
test_http_methods() {
    log "=== Testing HTTP Methods on /me Endpoint ==="

    if [[ ! -f "$TEMP_DIR/access_token.txt" ]]; then
        error "No access token available for HTTP method testing"
        return 1
    fi

    local access_token=$(cat "$TEMP_DIR/access_token.txt")
    local auth_headers="-H \"Authorization: Bearer $access_token\" -H \"Content-Type: application/json\""

    # POST method (should not be allowed)
    run_test "POST /me (should be 405)" "405" \
        -X POST \
        -H "Authorization: Bearer $access_token" \
        -H "Content-Type: application/json" \
        -d '{}' \
        "$BASE_URL/api/me"

    # PUT method (should not be allowed)
    run_test "PUT /me (should be 405)" "405" \
        -X PUT \
        -H "Authorization: Bearer $access_token" \
        -H "Content-Type: application/json" \
        -d '{}' \
        "$BASE_URL/api/me"

    # DELETE method (should not be allowed)
    run_test "DELETE /me (should be 405)" "405" \
        -X DELETE \
        -H "Authorization: Bearer $access_token" \
        -H "Content-Type: application/json" \
        "$BASE_URL/api/me"

    # PATCH method (should not be allowed)
    run_test "PATCH /me (should be 405)" "405" \
        -X PATCH \
        -H "Authorization: Bearer $access_token" \
        -H "Content-Type: application/json" \
        -d '{}' \
        "$BASE_URL/api/me"

    # HEAD method (might be allowed)
    run_test "HEAD /me" "200" \
        -X HEAD \
        -H "Authorization: Bearer $access_token" \
        -H "Content-Type: application/json" \
        "$BASE_URL/api/me" || true  # Don't fail if HEAD is not supported

    # OPTIONS method (might be allowed for CORS)
    run_test "OPTIONS /me" "200" \
        -X OPTIONS \
        "$BASE_URL/api/me" || true  # Don't fail if OPTIONS is not supported
}

# Test edge cases and security
test_edge_cases() {
    log "=== Testing Edge Cases and Security ==="

    if [[ ! -f "$TEMP_DIR/access_token.txt" ]]; then
        error "No access token available for edge case testing"
        return 1
    fi

    local access_token=$(cat "$TEMP_DIR/access_token.txt")

    # Test with very long Authorization header
    local long_token=$(printf 'a%.0s' {1..10000})  # 10k character token
    run_test "/me with extremely long token" "401" \
        -X GET \
        -H "Authorization: Bearer $long_token" \
        -H "Content-Type: application/json" \
        "$BASE_URL/api/me"

    # Test with special characters in headers
    run_test "/me with special characters in User-Agent" "200" \
        -X GET \
        -H "Authorization: Bearer $access_token" \
        -H "Content-Type: application/json" \
        -H "User-Agent: Test/1.0 (Special: àáâãäå; ñüöß)" \
        "$BASE_URL/api/me"

    # Test multiple Authorization headers (potential security issue)
    run_test "/me with multiple Authorization headers" "200" \
        -X GET \
        -H "Authorization: Bearer invalid-token" \
        -H "Authorization: Bearer $access_token" \
        -H "Content-Type: application/json" \
        "$BASE_URL/api/me" || true  # Might fail depending on server behavior

    # Test case sensitivity of Authorization header
    run_test "/me with lowercase authorization header" "401" \
        -X GET \
        -H "authorization: Bearer $access_token" \
        -H "Content-Type: application/json" \
        "$BASE_URL/api/me" || true  # Might work depending on server

    # Test with no Content-Type header
    run_test "/me without Content-Type header" "200" \
        -X GET \
        -H "Authorization: Bearer $access_token" \
        "$BASE_URL/api/me"
}

# Test concurrent requests
test_concurrent_requests() {
    log "=== Testing Concurrent Requests ==="

    if [[ ! -f "$TEMP_DIR/access_token.txt" ]]; then
        error "No access token available for concurrent testing"
        return 1
    fi

    local access_token=$(cat "$TEMP_DIR/access_token.txt")

    log "Running 5 concurrent /me requests..."

    # Create background processes for concurrent requests
    local pids=()
    for i in {1..5}; do
        (
            curl -s -w "%{http_code}" \
                -o "$TEMP_DIR/concurrent_$i.json" \
                -X GET \
                -H "Authorization: Bearer $access_token" \
                -H "Content-Type: application/json" \
                "$BASE_URL/api/me" > "$TEMP_DIR/concurrent_status_$i.txt"
        ) &
        pids+=($!)
    done

    # Wait for all requests to complete
    for pid in "${pids[@]}"; do
        wait $pid
    done

    # Check results
    local concurrent_success=0
    for i in {1..5}; do
        if [[ -f "$TEMP_DIR/concurrent_status_$i.txt" ]]; then
            local status=$(cat "$TEMP_DIR/concurrent_status_$i.txt")
            if [[ "$status" == "200" ]]; then
                ((concurrent_success++))
            fi
        fi
    done

    if [[ $concurrent_success -eq 5 ]]; then
        success "All 5 concurrent requests succeeded"
    else
        warning "Only $concurrent_success out of 5 concurrent requests succeeded"
    fi
}

# Performance testing
test_performance() {
    log "=== Testing Performance ==="

    if [[ ! -f "$TEMP_DIR/access_token.txt" ]]; then
        error "No access token available for performance testing"
        return 1
    fi

    local access_token=$(cat "$TEMP_DIR/access_token.txt")

    log "Running 10 sequential requests to measure average response time..."

    local total_time=0
    local successful_requests=0

    for i in {1..10}; do
        local start_time=$(date +%s%N)

        if HTTP_STATUS=$(curl -s -w "%{http_code}" \
            -o "$TEMP_DIR/perf_$i.json" \
            -X GET \
            -H "Authorization: Bearer $access_token" \
            -H "Content-Type: application/json" \
            "$BASE_URL/api/me"); then

            local end_time=$(date +%s%N)
            local duration=$(( (end_time - start_time) / 1000000 ))  # Convert to milliseconds

            if [[ "$HTTP_STATUS" == "200" ]]; then
                total_time=$((total_time + duration))
                ((successful_requests++))
                info "  Request $i: ${duration}ms"
            fi
        fi

        sleep 0.1  # Small delay between requests
    done

    if [[ $successful_requests -gt 0 ]]; then
        local average_time=$((total_time / successful_requests))
        success "Performance test completed:"
        info "  Successful requests: $successful_requests/10"
        info "  Average response time: ${average_time}ms"

        if [[ $average_time -lt 100 ]]; then
            success "  Excellent performance (< 100ms average)"
        elif [[ $average_time -lt 300 ]]; then
            success "  Good performance (< 300ms average)"
        elif [[ $average_time -lt 500 ]]; then
            warning "  Acceptable performance (< 500ms average)"
        else
            warning "  Slow performance (> 500ms average)"
        fi
    else
        error "Performance test failed - no successful requests"
    fi
}

# Main test execution
main() {
    log "🚀 Starting RustAxum /me Endpoint Tests"
    log "Target: $BASE_URL"
    log "Temp directory: $TEMP_DIR"
    echo

    # Run tests
    health_check
    echo

    if setup_test_user; then
        echo

        test_valid_authentication
        echo

        test_invalid_authentication
        echo

        test_http_methods
        echo

        test_edge_cases
        echo

        test_concurrent_requests
        echo

        test_performance
        echo
    else
        error "Failed to setup test user. Cannot continue with /me endpoint tests."
        exit 1
    fi

    # Summary
    log "=== Test Summary ==="
    echo -e "Total tests: ${BLUE}$TOTAL_TESTS${NC}"
    echo -e "Passed: ${GREEN}$PASSED_TESTS${NC}"
    echo -e "Failed: ${RED}$FAILED_TESTS${NC}"

    if [[ $FAILED_TESTS -eq 0 ]]; then
        echo -e "\n${GREEN}🎉 All /me endpoint tests passed!${NC}"
        exit 0
    else
        echo -e "\n${RED}❌ Some /me endpoint tests failed.${NC}"
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

# Run main function
main "$@"