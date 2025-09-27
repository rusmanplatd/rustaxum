import http from 'k6/http';
import { check, group, sleep } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';

// Custom metrics
export const errorRate = new Rate('errors');
export const apiResponseTime = new Trend('api_response_time');
export const authSuccessRate = new Rate('auth_success_rate');
export const endpointAvailability = new Rate('endpoint_availability');
export const totalRequests = new Counter('total_requests');

// Test configuration for comprehensive API testing
export const options = {
  scenarios: {
    // Smoke test - basic functionality
    smoke_test: {
      executor: 'constant-vus',
      vus: 1,
      duration: '30s',
      tags: { test_type: 'smoke' },
    },
    // Load test - normal expected load
    load_test: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '1m', target: 10 },   // Ramp up
        { duration: '3m', target: 10 },   // Stay at load
        { duration: '1m', target: 0 },    // Ramp down
      ],
      tags: { test_type: 'load' },
    },
    // Stress test - beyond normal capacity
    stress_test: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '30s', target: 10 },
        { duration: '1m', target: 20 },
        { duration: '30s', target: 30 },
        { duration: '1m', target: 30 },
        { duration: '30s', target: 0 },
      ],
      tags: { test_type: 'stress' },
    },
  },
  thresholds: {
    http_req_duration: ['p(95)<1000'], // 95% of requests under 1s
    errors: ['rate<0.05'],             // Error rate under 5%
    auth_success_rate: ['rate>0.95'],  // Auth success rate over 95%
    endpoint_availability: ['rate>0.98'], // Endpoint availability over 98%
  },
};

// Configuration
const BASE_URL = __ENV.BASE_URL || 'http://localhost:3000';
const THINK_TIME = parseFloat(__ENV.THINK_TIME) || 1;

// Test data
const TEST_USERS = [
  { name: 'Load Test User 1', email: 'loadtest1@example.com', password: 'password123' },
  { name: 'Load Test User 2', email: 'loadtest2@example.com', password: 'password123' },
  { name: 'Load Test User 3', email: 'loadtest3@example.com', password: 'password123' },
  { name: 'Load Test User 4', email: 'loadtest4@example.com', password: 'password123' },
  { name: 'Load Test User 5', email: 'loadtest5@example.com', password: 'password123' },
];

export function setup() {
  console.log(`Starting comprehensive API tests against ${BASE_URL}`);

  // Health check
  const healthResponse = http.get(`${BASE_URL}/api/docs`);
  if (healthResponse.status !== 200) {
    console.error(`Health check failed. Status: ${healthResponse.status}`);
    return null;
  }

  // Pre-create test users
  const createdUsers = [];
  for (let i = 0; i < TEST_USERS.length; i++) {
    const user = TEST_USERS[i];
    const uniqueEmail = `${Date.now()}_${i}_${user.email}`;

    const registrationPayload = JSON.stringify({
      name: user.name,
      email: uniqueEmail,
      password: user.password,
    });

    const registrationResponse = http.post(
      `${BASE_URL}/api/auth/register`,
      registrationPayload,
      { headers: { 'Content-Type': 'application/json' } }
    );

    if (registrationResponse.status === 201) {
      const registrationData = JSON.parse(registrationResponse.body);
      createdUsers.push({
        ...user,
        email: uniqueEmail,
        accessToken: registrationData.access_token,
        refreshToken: registrationData.refresh_token,
        userId: registrationData.user.id,
      });
    } else {
      console.warn(`Failed to create user ${uniqueEmail}: ${registrationResponse.body}`);
    }
  }

  console.log(`Setup completed. Created ${createdUsers.length} test users.`);
  return {
    baseUrl: BASE_URL,
    users: createdUsers,
  };
}

export default function (data) {
  if (!data || !data.users.length) {
    console.error('Setup failed or no users available, skipping test');
    return;
  }

  const { baseUrl, users } = data;
  const user = users[__VU % users.length];
  const testType = __ENV.K6_SCENARIO || 'default';

  totalRequests.add(1);

  group(`API Comprehensive Test - ${testType}`, function () {

    group('Authentication Flow', function () {
      // Login test
      const loginPayload = JSON.stringify({
        email: user.email,
        password: user.password,
      });

      const loginStart = Date.now();
      const loginResponse = http.post(
        `${baseUrl}/api/auth/login`,
        loginPayload,
        { headers: { 'Content-Type': 'application/json' } }
      );
      const loginEnd = Date.now();

      apiResponseTime.add(loginEnd - loginStart);

      const loginSuccess = check(loginResponse, {
        'login successful': (r) => r.status === 200,
        'login returns token': (r) => {
          try {
            const body = JSON.parse(r.body);
            return body.access_token !== undefined;
          } catch (e) {
            return false;
          }
        },
      });

      authSuccessRate.add(loginSuccess);
      endpointAvailability.add(loginResponse.status < 500);

      if (!loginSuccess) {
        errorRate.add(1);
        return;
      }

      const loginData = JSON.parse(loginResponse.body);
      const accessToken = loginData.access_token;

      // Think time
      sleep(THINK_TIME);

      // Test /me endpoint
      group('Current User Data', function () {
        const meStart = Date.now();
        const meResponse = http.get(`${baseUrl}/api/me`, {
          headers: {
            'Authorization': `Bearer ${accessToken}`,
            'Content-Type': 'application/json',
          },
        });
        const meEnd = Date.now();

        apiResponseTime.add(meEnd - meStart);

        const meSuccess = check(meResponse, {
          '/me status is 200': (r) => r.status === 200,
          '/me returns user data': (r) => {
            try {
              const body = JSON.parse(r.body);
              return body.user && body.user.email === user.email;
            } catch (e) {
              return false;
            }
          },
          '/me response time acceptable': (r) => r.timings.duration < 500,
        });

        endpointAvailability.add(meResponse.status < 500);
        if (!meSuccess) errorRate.add(1);
      });

      sleep(THINK_TIME);

      // Test protected endpoints
      group('Protected Endpoints', function () {
        const authHeaders = {
          headers: {
            'Authorization': `Bearer ${accessToken}`,
            'Content-Type': 'application/json',
          },
        };

        // Test various protected endpoints
        const endpoints = [
          { method: 'GET', url: '/api/users', name: 'users_list' },
          { method: 'GET', url: '/api/countries', name: 'countries_list' },
          { method: 'GET', url: '/api/roles', name: 'roles_list' },
          { method: 'GET', url: '/api/permissions', name: 'permissions_list' },
        ];

        endpoints.forEach(endpoint => {
          const endpointStart = Date.now();
          let response;

          if (endpoint.method === 'GET') {
            response = http.get(`${baseUrl}${endpoint.url}`, authHeaders);
          } else if (endpoint.method === 'POST') {
            response = http.post(`${baseUrl}${endpoint.url}`, '{}', authHeaders);
          }

          const endpointEnd = Date.now();
          apiResponseTime.add(endpointEnd - endpointStart);

          check(response, {
            [`${endpoint.name} accessible`]: (r) => r.status < 500,
            [`${endpoint.name} authenticated`]: (r) => r.status !== 401,
            [`${endpoint.name} response time OK`]: (r) => r.timings.duration < 1000,
          });

          endpointAvailability.add(response.status < 500);
          if (response.status >= 400) errorRate.add(1);

          // Small delay between endpoint calls
          sleep(0.1);
        });
      });

      sleep(THINK_TIME);

      // Test refresh token
      group('Token Refresh', function () {
        const refreshPayload = JSON.stringify({
          refresh_token: user.refreshToken,
        });

        const refreshStart = Date.now();
        const refreshResponse = http.post(
          `${baseUrl}/api/auth/refresh-token`,
          refreshPayload,
          { headers: { 'Content-Type': 'application/json' } }
        );
        const refreshEnd = Date.now();

        apiResponseTime.add(refreshEnd - refreshStart);

        const refreshSuccess = check(refreshResponse, {
          'refresh token works': (r) => r.status === 200,
          'refresh returns new token': (r) => {
            try {
              const body = JSON.parse(r.body);
              return body.access_token !== undefined;
            } catch (e) {
              return false;
            }
          },
        });

        endpointAvailability.add(refreshResponse.status < 500);
        if (!refreshSuccess) errorRate.add(1);
      });
    });

    group('Error Handling', function () {
      // Test invalid endpoints
      const invalidResponse = http.get(`${baseUrl}/api/nonexistent`);
      check(invalidResponse, {
        'invalid endpoint returns 404': (r) => r.status === 404,
      });

      // Test unauthorized access
      const unauthorizedResponse = http.get(`${baseUrl}/api/me`);
      check(unauthorizedResponse, {
        'unauthorized access returns 401': (r) => r.status === 401,
      });

      // Test malformed JSON
      const malformedResponse = http.post(
        `${baseUrl}/api/auth/login`,
        '{ malformed json',
        { headers: { 'Content-Type': 'application/json' } }
      );
      check(malformedResponse, {
        'malformed JSON handled': (r) => r.status === 400,
      });
    });

    group('Performance Checks', function () {
      // Test static content or documentation
      const docsStart = Date.now();
      const docsResponse = http.get(`${baseUrl}/api/docs`);
      const docsEnd = Date.now();

      apiResponseTime.add(docsEnd - docsStart);

      check(docsResponse, {
        'docs endpoint fast': (r) => r.timings.duration < 100,
        'docs endpoint available': (r) => r.status === 200,
      });

      endpointAvailability.add(docsResponse.status < 500);
    });
  });

  // Random think time between iterations
  sleep(Math.random() * 2);
}

export function teardown(data) {
  if (data) {
    console.log('Comprehensive API Test Summary:');
    console.log(`- Base URL: ${data.baseUrl}`);
    console.log(`- Test Users Created: ${data.users.length}`);
    console.log(`- Scenarios: ${Object.keys(options.scenarios).join(', ')}`);

    // Cleanup could be performed here if needed
    console.log('Comprehensive API tests completed');
  }
}