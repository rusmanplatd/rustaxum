#!/bin/bash

# RustAxum Authentication Tests using cURL
# Tests user registration, login, and authentication flows

set -e  # Exit on any error

# Configuration
BASE_URL="${BASE_URL:-http://localhost:3000}"
TEMP_DIR="/tmp/rustaxum-curl-tests"
RESULTS_FILE="$TEMP_DIR/auth-test-results.json"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
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

    if HTTP_STATUS=$(curl -s -w "%{http_code}" -D "$headers_file" -o "$response_file" $curl_args); then
        if [[ "$HTTP_STATUS" == "$expected_status" ]]; then
            success "$test_name (Status: $HTTP_STATUS)"
            return 0
        else
            error "$test_name (Expected: $expected_status, Got: $HTTP_STATUS)"
            if [[ -f "$response_file" ]]; then
                echo "Response body:"
                cat "$response_file" | jq . 2>/dev/null || cat "$response_file"
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

# Health check
health_check() {
    log "Performing health check..."
    if run_test "Health Check" "200" "$BASE_URL/api/docs"; then
        log "✓ API is accessible at $BASE_URL"
    else
        error "✗ API health check failed. Is the server running at $BASE_URL?"
        exit 1
    fi
}

# Generate unique test data
generate_test_user() {
    local timestamp=$(date +%s)
    local random=$(shuf -i 1000-9999 -n 1)
    echo "auth_test_${timestamp}_${random}@example.com"
}

# User Registration Tests
test_user_registration() {
    log "=== Testing User Registration ==="

    local test_email=$(generate_test_user)
    local test_name="Test User Registration"
    local test_password="testpassword123"

    # Valid registration
    local reg_payload=$(cat <<EOF
{
    "name": "$test_name",
    "email": "$test_email",
    "password": "$test_password"
}
EOF
)

    if run_test "Valid User Registration" "201" \
        -X POST \
        -H "Content-Type: application/json" \
        -d "$reg_payload" \
        "$BASE_URL/api/auth/register"; then

        # Check response contains required fields
        local response_file="$TEMP_DIR/response_$TOTAL_TESTS.json"
        local access_token=$(get_json_value "$response_file" "access_token")
        local user_email=$(get_json_value "$response_file" "user.email")

        if [[ -n "$access_token" && "$user_email" == "$test_email" ]]; then
            success "Registration response contains access token and correct user data"
            echo "$access_token" > "$TEMP_DIR/access_token.txt"
            echo "$test_email" > "$TEMP_DIR/test_email.txt"
            echo "$test_password" > "$TEMP_DIR/test_password.txt"
        else
            error "Registration response missing required fields"
        fi
    fi

    # Duplicate email registration
    run_test "Duplicate Email Registration" "400" \
        -X POST \
        -H "Content-Type: application/json" \
        -d "$reg_payload" \
        "$BASE_URL/api/auth/register"

    # Invalid email format
    local invalid_email_payload=$(cat <<EOF
{
    "name": "Test User",
    "email": "invalid-email",
    "password": "password123"
}
EOF
)

    run_test "Invalid Email Format" "400" \
        -X POST \
        -H "Content-Type: application/json" \
        -d "$invalid_email_payload" \
        "$BASE_URL/api/auth/register"

    # Missing required fields
    local missing_fields_payload='{"name": "Test User"}'

    run_test "Missing Required Fields" "400" \
        -X POST \
        -H "Content-Type: application/json" \
        -d "$missing_fields_payload" \
        "$BASE_URL/api/auth/register"

    # Invalid JSON
    run_test "Invalid JSON" "400" \
        -X POST \
        -H "Content-Type: application/json" \
        -d '{"invalid": json}' \
        "$BASE_URL/api/auth/register"
}

# User Login Tests
test_user_login() {
    log "=== Testing User Login ==="

    if [[ ! -f "$TEMP_DIR/test_email.txt" || ! -f "$TEMP_DIR/test_password.txt" ]]; then
        error "Cannot run login tests - registration test data not available"
        return 1
    fi

    local test_email=$(cat "$TEMP_DIR/test_email.txt")
    local test_password=$(cat "$TEMP_DIR/test_password.txt")

    # Valid login
    local login_payload=$(cat <<EOF
{
    "email": "$test_email",
    "password": "$test_password"
}
EOF
)

    if run_test "Valid User Login" "200" \
        -X POST \
        -H "Content-Type: application/json" \
        -d "$login_payload" \
        "$BASE_URL/api/auth/login"; then

        # Extract and save new access token
        local response_file="$TEMP_DIR/response_$TOTAL_TESTS.json"
        local access_token=$(get_json_value "$response_file" "access_token")
        local refresh_token=$(get_json_value "$response_file" "refresh_token")

        if [[ -n "$access_token" && -n "$refresh_token" ]]; then
            success "Login response contains access and refresh tokens"
            echo "$access_token" > "$TEMP_DIR/login_access_token.txt"
            echo "$refresh_token" > "$TEMP_DIR/refresh_token.txt"
        else
            error "Login response missing required tokens"
        fi
    fi

    # Invalid credentials
    local invalid_login_payload=$(cat <<EOF
{
    "email": "$test_email",
    "password": "wrongpassword"
}
EOF
)

    run_test "Invalid Password" "401" \
        -X POST \
        -H "Content-Type: application/json" \
        -d "$invalid_login_payload" \
        "$BASE_URL/api/auth/login"

    # Non-existent user
    local nonexistent_user_payload=$(cat <<EOF
{
    "email": "nonexistent@example.com",
    "password": "password123"
}
EOF
)

    run_test "Non-existent User" "401" \
        -X POST \
        -H "Content-Type: application/json" \
        -d "$nonexistent_user_payload" \
        "$BASE_URL/api/auth/login"

    # Missing email
    local missing_email_payload='{"password": "password123"}'

    run_test "Missing Email" "400" \
        -X POST \
        -H "Content-Type: application/json" \
        -d "$missing_email_payload" \
        "$BASE_URL/api/auth/login"

    # Missing password
    local missing_password_payload="{\"email\": \"$test_email\"}"

    run_test "Missing Password" "400" \
        -X POST \
        -H "Content-Type: application/json" \
        -d "$missing_password_payload" \
        "$BASE_URL/api/auth/login"
}

# Token Refresh Tests
test_token_refresh() {
    log "=== Testing Token Refresh ==="

    if [[ ! -f "$TEMP_DIR/refresh_token.txt" ]]; then
        warning "Cannot run refresh token tests - no refresh token available"
        return 0
    fi

    local refresh_token=$(cat "$TEMP_DIR/refresh_token.txt")

    # Valid refresh
    local refresh_payload=$(cat <<EOF
{
    "refresh_token": "$refresh_token"
}
EOF
)

    if run_test "Valid Token Refresh" "200" \
        -X POST \
        -H "Content-Type: application/json" \
        -d "$refresh_payload" \
        "$BASE_URL/api/auth/refresh-token"; then

        # Save new tokens
        local response_file="$TEMP_DIR/response_$TOTAL_TESTS.json"
        local new_access_token=$(get_json_value "$response_file" "access_token")

        if [[ -n "$new_access_token" ]]; then
            success "Token refresh returned new access token"
            echo "$new_access_token" > "$TEMP_DIR/refreshed_access_token.txt"
        else
            error "Token refresh response missing access token"
        fi
    fi

    # Invalid refresh token
    local invalid_refresh_payload='{"refresh_token": "invalid_token"}'

    run_test "Invalid Refresh Token" "401" \
        -X POST \
        -H "Content-Type: application/json" \
        -d "$invalid_refresh_payload" \
        "$BASE_URL/api/auth/refresh-token"

    # Missing refresh token
    run_test "Missing Refresh Token" "400" \
        -X POST \
        -H "Content-Type: application/json" \
        -d '{}' \
        "$BASE_URL/api/auth/refresh-token"
}

# Performance and Rate Limiting Tests
test_performance() {
    log "=== Testing Performance and Rate Limiting ==="

    local test_email=$(generate_test_user)
    local login_payload=$(cat <<EOF
{
    "email": "$test_email",
    "password": "wrongpassword"
}
EOF
)

    # Multiple rapid requests (rate limiting test)
    log "Testing multiple rapid login attempts..."
    local rate_limit_failures=0

    for i in {1..5}; do
        if ! run_test "Rapid Login Attempt $i" "401" \
            -X POST \
            -H "Content-Type: application/json" \
            -d "$login_payload" \
            "$BASE_URL/api/auth/login" >/dev/null 2>&1; then
            ((rate_limit_failures++))
        fi
        sleep 0.1  # Small delay between requests
    done

    if [[ $rate_limit_failures -eq 0 ]]; then
        success "Rate limiting test completed (all requests handled properly)"
    else
        warning "Some rapid requests failed (this might indicate rate limiting)"
    fi

    # Response time test
    log "Testing response time..."
    local start_time=$(date +%s%N)

    run_test "Response Time Test" "401" \
        -X POST \
        -H "Content-Type: application/json" \
        -d "$login_payload" \
        "$BASE_URL/api/auth/login" >/dev/null 2>&1

    local end_time=$(date +%s%N)
    local duration=$(( (end_time - start_time) / 1000000 ))  # Convert to milliseconds

    if [[ $duration -lt 1000 ]]; then
        success "Response time: ${duration}ms (under 1 second)"
    else
        warning "Response time: ${duration}ms (over 1 second)"
    fi
}

# Main test execution
main() {
    log "🚀 Starting RustAxum Authentication Tests"
    log "Target: $BASE_URL"
    log "Temp directory: $TEMP_DIR"
    echo

    # Run tests
    health_check
    echo

    test_user_registration
    echo

    test_user_login
    echo

    test_token_refresh
    echo

    test_performance
    echo

    # Summary
    log "=== Test Summary ==="
    echo -e "Total tests: ${BLUE}$TOTAL_TESTS${NC}"
    echo -e "Passed: ${GREEN}$PASSED_TESTS${NC}"
    echo -e "Failed: ${RED}$FAILED_TESTS${NC}"

    if [[ $FAILED_TESTS -eq 0 ]]; then
        echo -e "\n${GREEN}🎉 All tests passed!${NC}"
        exit 0
    else
        echo -e "\n${RED}❌ Some tests failed.${NC}"
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