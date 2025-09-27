# K6 Load Testing Suite

This directory contains k6 performance and load testing scripts for the RustAxum API.

## Prerequisites

1. Install k6: https://k6.io/docs/getting-started/installation/
2. Ensure the RustAxum application is running (locally or deployed)

## Test Scripts

### 1. Authentication Tests (`auth.js`)

Tests user registration, login, and authentication flows.

**Features:**
- User registration testing
- Login flow validation
- Invalid credential handling
- Malformed request testing
- Performance metrics for auth operations

**Usage:**
```bash
# Run with default settings (localhost:3000)
k6 run tests/k6/auth.js

# Run against different environment
k6 run --env BASE_URL=http://localhost:8080 tests/k6/auth.js

# Run with custom VU configuration
k6 run --vus 20 --duration 2m tests/k6/auth.js
```

**Metrics:**
- `login_duration`: Time taken for login requests
- `register_duration`: Time taken for registration requests
- `errors`: Rate of failed requests

### 2. /me Endpoint Tests (`me-endpoint.js`)

Focused testing of the `/api/me` endpoint for authenticated user data retrieval.

**Features:**
- Valid JWT token authentication
- Invalid/expired token handling
- Concurrent access testing
- HTTP method validation
- Response data validation

**Usage:**
```bash
# Run /me endpoint tests
k6 run tests/k6/me-endpoint.js

# Run with higher load
k6 run --vus 50 --duration 5m tests/k6/me-endpoint.js

# Run against staging environment
k6 run --env BASE_URL=https://staging.example.com tests/k6/me-endpoint.js
```

**Metrics:**
- `me_endpoint_duration`: Response time for /me requests
- `auth_failures`: Count of authentication failures
- `successful_requests`: Count of successful requests

### 3. Comprehensive API Tests (`api-comprehensive.js`)

Full API testing suite with multiple scenarios.

**Features:**
- Smoke testing (basic functionality)
- Load testing (normal expected load)
- Stress testing (beyond normal capacity)
- Multiple endpoint coverage
- Error handling validation
- Performance benchmarking

**Usage:**
```bash
# Run all scenarios
k6 run tests/k6/api-comprehensive.js

# Run specific scenario only
k6 run --env K6_SCENARIO=smoke_test tests/k6/api-comprehensive.js

# Run with custom think time
k6 run --env THINK_TIME=2 tests/k6/api-comprehensive.js

# Run against production (be careful!)
k6 run --env BASE_URL=https://api.production.com tests/k6/api-comprehensive.js
```

**Scenarios:**
- `smoke_test`: 1 VU for 30s (basic functionality)
- `load_test`: Ramp up to 10 VUs over 5 minutes
- `stress_test`: Ramp up to 30 VUs with stress phases

## Environment Variables

All test scripts support these environment variables:

- `BASE_URL`: Target API base URL (default: `http://localhost:3000`)
- `THINK_TIME`: Delay between requests in seconds (default: `1`)
- `K6_SCENARIO`: Specific scenario to run (comprehensive test only)

## Running Tests

### Local Development

1. Start the RustAxum application:
```bash
cargo run --bin artisan -- serve --port 3000
```

2. Run tests:
```bash
# Quick smoke test
k6 run --vus 1 --duration 30s tests/k6/auth.js

# Load test
k6 run tests/k6/api-comprehensive.js

# Focused /me endpoint test
k6 run tests/k6/me-endpoint.js
```

### CI/CD Integration

Example GitHub Actions workflow:

```yaml
- name: Run k6 Load Tests
  run: |
    # Start application in background
    cargo run --bin artisan -- serve --port 3000 &
    sleep 5

    # Run tests
    k6 run --out json=results.json tests/k6/auth.js
    k6 run --out json=results.json tests/k6/me-endpoint.js

    # Lightweight smoke test for CI
    k6 run --vus 1 --duration 10s tests/k6/api-comprehensive.js
```

### Docker Testing

```bash
# Run k6 in Docker against local app
docker run --rm -i --network=host \
  -v $PWD/tests/k6:/scripts \
  grafana/k6:latest run /scripts/auth.js

# Run against application in Docker Compose
docker run --rm -i --network=rustaxum_default \
  -v $PWD/tests/k6:/scripts \
  -e BASE_URL=http://app:3000 \
  grafana/k6:latest run /scripts/api-comprehensive.js
```

## Performance Thresholds

### Authentication Tests
- 95% of requests under 500ms
- Error rate under 5%

### /me Endpoint Tests
- 95% of requests under 300ms
- 99% of requests under 500ms
- Error rate under 2%

### Comprehensive Tests
- 95% of requests under 1000ms
- Error rate under 5%
- Auth success rate over 95%
- Endpoint availability over 98%

## Interpreting Results

### Key Metrics to Monitor

1. **Response Time Percentiles**: p(95), p(99)
2. **Error Rates**: HTTP errors, authentication failures
3. **Throughput**: Requests per second
4. **Resource Utilization**: CPU, memory during tests

### Common Issues

- **High Error Rates**: Check database connections, rate limiting
- **Slow Response Times**: Database query optimization, indexing
- **Authentication Failures**: JWT secret configuration, token expiry
- **502/503 Errors**: Server overload, increase capacity

### Sample Output

```
✓ login status is 200
✓ login returns access token
✓ /me status is 200
✓ /me returns user data

checks.........................: 100.00% ✓ 1247      ✗ 0
data_received..................: 892 kB  15 kB/s
data_sent......................: 445 kB  7.4 kB/s
http_req_duration..............: avg=89.32ms  min=12.45ms med=67.23ms max=445.67ms p(90)=156.78ms p(95)=234.56ms
http_reqs......................: 2494    41.566661/s
iteration_duration.............: avg=1.08s    min=1.01s   med=1.06s   max=1.52s    p(90)=1.15s    p(95)=1.23s
iterations.....................: 1247    20.783331/s
vus............................: 10      min=10      max=30
vus_max........................: 30      min=30      max=30
```

## Best Practices

1. **Start Small**: Begin with smoke tests before load testing
2. **Monitor Resources**: Watch CPU, memory, database connections
3. **Realistic Data**: Use production-like test data volumes
4. **Think Time**: Include realistic user behavior delays
5. **Gradual Ramp**: Don't jump to peak load immediately
6. **Clean Up**: Remove test data after runs when possible

## Troubleshooting

### Common Issues

**Connection Refused**:
```
ERRO[0000] GoError: Get "http://localhost:3000": dial tcp [::1]:3000: connect: connection refused
```
- Ensure application is running on specified port
- Check if port is correct in BASE_URL

**High Error Rates**:
- Check application logs for errors
- Verify database is running and accessible
- Check rate limiting configuration

**Performance Issues**:
- Monitor database query performance
- Check for N+1 query problems
- Verify adequate server resources

For more information on k6, visit: https://k6.io/docs/