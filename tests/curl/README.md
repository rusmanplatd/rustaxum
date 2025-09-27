# cURL API Testing Suite

This directory contains comprehensive cURL-based tests for the RustAxum API, providing an alternative to k6 for testing authentication flows and API endpoints.

## 📁 Test Structure

```
tests/curl/
├── README.md                    # This documentation
├── run-tests.sh                 # Main test runner with reporting
├── auth-tests.sh               # Authentication flow tests
├── me-endpoint-tests.sh        # Focused /me endpoint tests
└── api-comprehensive-tests.sh  # Full API testing suite
```

## 🚀 Quick Start

### Prerequisites

1. **RustAxum server running**:
   ```bash
   cargo run --bin artisan -- serve --port 3000
   ```

2. **Required tools**:
   - `curl` (for HTTP requests)
   - `jq` (optional, for JSON parsing and enhanced reporting)
   - `bash` (shell for test execution)

### Basic Usage

```bash
# Run all tests
./tests/curl/run-tests.sh

# Run specific test suite
./tests/curl/run-tests.sh --type auth        # Authentication only
./tests/curl/run-tests.sh --type me          # /me endpoint only
./tests/curl/run-tests.sh --type smoke       # Quick smoke tests

# Test against different environment
./tests/curl/run-tests.sh --url http://localhost:8080

# Generate HTML report
./tests/curl/run-tests.sh --report html --output ./reports
```

## 📋 Test Suites

### 1. Authentication Tests (`auth-tests.sh`)

**Purpose**: Comprehensive testing of user registration, login, and token management.

**Features**:
- ✅ User registration with validation
- ✅ Login flow with valid/invalid credentials
- ✅ JWT token refresh functionality
- ✅ Error handling (malformed JSON, missing fields)
- ✅ Performance measurement
- ✅ Rate limiting validation

**Usage**:
```bash
# Direct execution
./tests/curl/auth-tests.sh

# With custom base URL
BASE_URL=http://localhost:8080 ./tests/curl/auth-tests.sh
```

**Sample Output**:
```
✓ Valid User Registration (Status: 201, Time: 145ms)
✓ Duplicate Email Registration (Status: 400, Time: 23ms)
✓ Valid User Login (Status: 200, Time: 89ms)
✓ Invalid Password (Status: 401, Time: 67ms)
Total tests: 15, Passed: 15, Failed: 0
```

### 2. ME Endpoint Tests (`me-endpoint-tests.sh`)

**Purpose**: Focused testing of the `/api/me` endpoint for authenticated user data retrieval.

**Features**:
- ✅ Valid JWT authentication scenarios
- ✅ Invalid/expired token handling
- ✅ HTTP method validation (GET only)
- ✅ Concurrent request testing
- ✅ Response structure validation
- ✅ Security checks (no password exposure)
- ✅ Performance benchmarking

**Usage**:
```bash
# Direct execution
./tests/curl/me-endpoint-tests.sh

# Test specific scenarios
BASE_URL=http://staging.example.com ./tests/curl/me-endpoint-tests.sh
```

**Sample Output**:
```
✓ GET /me with valid token (Status: 200, Time: 45ms)
✓ Response structure validation passed
✓ Security: password field not exposed
✓ All 5 concurrent requests succeeded
Average response time: 52ms
```

### 3. Comprehensive API Tests (`api-comprehensive-tests.sh`)

**Purpose**: Full API coverage with multiple test modes and endpoint validation.

**Test Modes**:
- **`full`**: Complete test suite (default)
- **`smoke`**: Basic functionality check (fast)
- **`endpoints`**: Endpoint availability and authentication

**Features**:
- ✅ Multiple user creation and management
- ✅ Protected endpoint testing
- ✅ OAuth endpoint validation
- ✅ Error handling verification
- ✅ CORS and HTTP method testing
- ✅ Concurrent request handling
- ✅ Performance analysis

**Usage**:
```bash
# Full comprehensive tests
./tests/curl/api-comprehensive-tests.sh

# Different modes
./tests/curl/api-comprehensive-tests.sh --mode smoke
./tests/curl/api-comprehensive-tests.sh --mode endpoints

# Custom configuration
./tests/curl/api-comprehensive-tests.sh --parallel 10 --url http://production.com
```

## 🎛️ Test Runner (`run-tests.sh`)

The main test orchestrator with advanced reporting and CI/CD integration.

### Command Line Options

```bash
./tests/curl/run-tests.sh [options]

Options:
  -h, --help              Show help message
  -t, --type TYPE         Test type: all, auth, me, comprehensive, smoke, endpoints
  -u, --url URL           Base URL (default: http://localhost:3000)
  -r, --report FORMAT     Report format: console, json, html, junit
  -o, --output DIR        Output directory (default: ./test-results)
  -p, --parallel          Run test suites in parallel
  -f, --fail-fast         Stop on first test suite failure
```

### Environment Variables

```bash
export BASE_URL="http://localhost:3000"    # API base URL
export TEST_TYPE="all"                     # Test type
export REPORT_FORMAT="console"             # Report format
export OUTPUT_DIR="./test-results"         # Output directory
export PARALLEL="false"                    # Parallel execution
export FAIL_FAST="false"                   # Fail fast mode
```

### Report Formats

#### Console Report (Default)
```bash
./tests/curl/run-tests.sh
```
Real-time colored output with immediate feedback.

#### JSON Report
```bash
./tests/curl/run-tests.sh --report json
```
Machine-readable JSON with detailed metrics:
```json
{
  "timestamp": "2025-09-27T09:30:00Z",
  "total_suites": 3,
  "passed_suites": 3,
  "failed_suites": 0,
  "total_execution_time": 45,
  "test_suites": [...]
}
```

#### HTML Report
```bash
./tests/curl/run-tests.sh --report html
```
Interactive HTML report with:
- Visual test results
- Performance metrics
- Detailed output logs
- Responsive design

#### JUnit XML Report
```bash
./tests/curl/run-tests.sh --report junit
```
CI/CD compatible XML format for integration with:
- Jenkins
- GitHub Actions
- GitLab CI
- Azure DevOps

## 🔧 Advanced Usage

### Parallel Execution

```bash
# Run test suites in parallel for faster execution
./tests/curl/run-tests.sh --parallel

# Combine with specific test type
./tests/curl/run-tests.sh --type comprehensive --parallel
```

### CI/CD Integration

#### GitHub Actions Example

```yaml
name: API Tests
on: [push, pull_request]

jobs:
  api-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Start RustAxum Server
        run: |
          cargo run --bin artisan -- serve --port 3000 &
          sleep 5

      - name: Run API Tests
        run: |
          ./tests/curl/run-tests.sh \
            --report junit \
            --output ./test-results \
            --fail-fast

      - name: Publish Test Results
        uses: dorny/test-reporter@v1
        if: always()
        with:
          name: API Test Results
          path: ./test-results/junit-report.xml
          reporter: java-junit
```

#### Docker Integration

```bash
# Test application running in Docker
docker run --rm --network=host \
  -v "$PWD/tests/curl:/tests" \
  curlimages/curl:latest \
  bash /tests/run-tests.sh --url http://localhost:3000
```

### Custom Test Scenarios

#### Staging Environment Testing
```bash
./tests/curl/run-tests.sh \
  --url https://staging.example.com \
  --type smoke \
  --report html \
  --output ./staging-results
```

#### Load Testing Simulation
```bash
# Run multiple concurrent test suites
for i in {1..5}; do
  ./tests/curl/run-tests.sh --type me --parallel &
done
wait
```

#### Performance Monitoring
```bash
# Generate performance reports
./tests/curl/run-tests.sh \
  --type comprehensive \
  --report json \
  --output ./perf-$(date +%Y%m%d-%H%M%S)
```

## 📊 Understanding Test Results

### Success Indicators
- ✅ **All tests pass**: Green checkmarks with response times
- 📊 **Performance metrics**: Average response times under thresholds
- 🔒 **Security validation**: No sensitive data exposure
- 🏃 **Concurrent handling**: Multiple simultaneous requests succeed

### Common Issues and Solutions

#### Connection Refused
```
✗ Health Check (Connection failed)
```
**Solution**: Ensure RustAxum server is running:
```bash
cargo run --bin artisan -- serve --port 3000
```

#### Authentication Failures
```
✗ GET /api/me (Expected: 200, Got: 401)
```
**Solution**: Check JWT configuration and token generation in auth tests.

#### Slow Response Times
```
⚠ Response time: 1250ms (over 1 second)
```
**Solution**:
- Check database performance
- Monitor server resources
- Verify network latency

#### Test Data Conflicts
```
✗ Valid User Registration (Expected: 201, Got: 400)
```
**Solution**: Tests create unique users with timestamps to avoid conflicts.

## 🛠️ Customization

### Adding New Test Cases

1. **Create test function** in appropriate script:
```bash
test_new_feature() {
    log "=== Testing New Feature ==="

    run_test "New Feature Test" "200" \
        -X GET \
        -H "Authorization: Bearer $token" \
        "$BASE_URL/api/new-feature"
}
```

2. **Add to test execution**:
```bash
# In main() function
test_new_feature
```

### Custom Validation

```bash
# Add response validation
validate_custom_response() {
    local response_file="$1"

    if command -v jq >/dev/null 2>&1; then
        local custom_field=$(jq -r '.custom_field' "$response_file")
        if [[ -n "$custom_field" ]]; then
            success "Custom field validation passed"
        else
            error "Custom field missing"
        fi
    fi
}
```

### Environment-Specific Configuration

```bash
# Create environment-specific configs
case "$ENVIRONMENT" in
    "production")
        BASE_URL="https://api.production.com"
        TEST_TYPE="smoke"
        ;;
    "staging")
        BASE_URL="https://staging.example.com"
        TEST_TYPE="comprehensive"
        ;;
    "local"|*)
        BASE_URL="http://localhost:3000"
        TEST_TYPE="all"
        ;;
esac
```

## 🔍 Debugging

### Verbose Output
```bash
# Enable detailed output
set -x  # Add to script for trace mode
./tests/curl/run-tests.sh --type auth
```

### Manual Test Execution
```bash
# Run individual curl commands
curl -X POST \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"password123"}' \
  http://localhost:3000/api/auth/login
```

### Log Analysis
```bash
# Check test output files
ls -la ./test-results/
cat ./test-results/authentication-tests-output.txt
```

## 📈 Performance Benchmarks

### Expected Response Times
- **Authentication**: < 200ms
- **Me endpoint**: < 100ms
- **List endpoints**: < 300ms
- **Public endpoints**: < 50ms

### Concurrent Request Handling
- **5 concurrent /me requests**: All should succeed
- **Response time consistency**: < 20% variance
- **No data corruption**: All responses identical

### Resource Usage
- **Memory**: Stable during concurrent tests
- **CPU**: Should not spike above 80%
- **Database connections**: Properly pooled and released

## 🆚 Comparison with k6 Tests

| Feature | cURL Tests | k6 Tests |
|---------|------------|-----------|
| **Setup** | Bash scripts, no dependencies | Requires k6 installation |
| **Performance** | Individual request testing | High-load simulation |
| **Debugging** | Easy bash debugging | JavaScript debugging |
| **CI/CD** | Native shell integration | Requires k6 in CI |
| **Reporting** | Multiple formats | Built-in metrics |
| **Use Case** | Functional testing | Performance testing |

**Recommendation**: Use cURL tests for functional validation and k6 tests for performance/load testing.

## 🤝 Contributing

1. **Add new test cases** following existing patterns
2. **Update documentation** for new features
3. **Test against multiple environments**
4. **Follow bash best practices** (shellcheck compliance)
5. **Include error handling** for edge cases

## 📚 Additional Resources

- [cURL Documentation](https://curl.se/docs/)
- [jq Manual](https://stedolan.github.io/jq/manual/)
- [Bash Scripting Guide](https://www.gnu.org/software/bash/manual/)
- [HTTP Status Codes](https://httpstatuses.com/)
- [JWT Tokens](https://jwt.io/)

---

For questions or issues, check the test output logs in the `test-results` directory or review the individual test script comments for detailed implementation notes.