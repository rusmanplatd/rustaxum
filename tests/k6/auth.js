import http from 'k6/http';
import { check, group } from 'k6';
import { Rate, Trend } from 'k6/metrics';

// Custom metrics
export let errorRate = new Rate('errors');
export let loginDuration = new Trend('login_duration');
export let registerDuration = new Trend('register_duration');

// Test configuration
export let options = {
  stages: [
    { duration: '30s', target: 10 },   // Ramp up to 10 users
    { duration: '1m', target: 10 },    // Stay at 10 users
    { duration: '20s', target: 0 },    // Ramp down to 0 users
  ],
  thresholds: {
    http_req_duration: ['p(95)<500'], // 95% of requests must complete below 500ms
    errors: ['rate<0.05'],            // Error rate must be less than 5%
  },
};

// Base URL configuration
const BASE_URL = __ENV.BASE_URL || 'http://localhost:3000';

// Test data
const testUsers = [
  { name: 'Test User 1', email: 'testuser1@example.com', password: 'password123' },
  { name: 'Test User 2', email: 'testuser2@example.com', password: 'password123' },
  { name: 'Test User 3', email: 'testuser3@example.com', password: 'password123' },
];

export function setup() {
  console.log(`Running authentication tests against ${BASE_URL}`);

  // Health check
  let healthCheck = http.get(`${BASE_URL}/api/docs`);
  if (healthCheck.status !== 200) {
    console.error(`Health check failed. Status: ${healthCheck.status}`);
    return null;
  }

  console.log('Health check passed');
  return { baseUrl: BASE_URL };
}

export default function (data) {
  if (!data) {
    console.error('Setup failed, skipping test');
    return;
  }

  const baseUrl = data.baseUrl;
  const userIndex = __VU % testUsers.length;
  const testUser = testUsers[userIndex];

  group('Authentication Flow', function () {

    group('User Registration', function () {
      const registrationPayload = JSON.stringify({
        name: testUser.name,
        email: `${Date.now()}_${testUser.email}`, // Make email unique
        password: testUser.password,
      });

      const registrationParams = {
        headers: {
          'Content-Type': 'application/json',
        },
      };

      const registrationStart = Date.now();
      let registrationResponse = http.post(
        `${baseUrl}/api/auth/register`,
        registrationPayload,
        registrationParams
      );
      const registrationEnd = Date.now();

      registerDuration.add(registrationEnd - registrationStart);

      let registrationSuccess = check(registrationResponse, {
        'registration status is 201': (r) => r.status === 201,
        'registration returns access token': (r) => {
          try {
            const body = JSON.parse(r.body);
            return body.access_token !== undefined;
          } catch (e) {
            return false;
          }
        },
        'registration returns user data': (r) => {
          try {
            const body = JSON.parse(r.body);
            return body.user && body.user.email && body.user.name;
          } catch (e) {
            return false;
          }
        },
        'registration response time < 1000ms': (r) => r.timings.duration < 1000,
      });

      if (!registrationSuccess) {
        errorRate.add(1);
        console.error(`Registration failed for ${testUser.email}: ${registrationResponse.body}`);
        return;
      }

      errorRate.add(0);
    });

    group('User Login', function () {
      const loginPayload = JSON.stringify({
        email: testUser.email,
        password: testUser.password,
      });

      const loginParams = {
        headers: {
          'Content-Type': 'application/json',
        },
      };

      const loginStart = Date.now();
      let loginResponse = http.post(
        `${baseUrl}/api/auth/login`,
        loginPayload,
        loginParams
      );
      const loginEnd = Date.now();

      loginDuration.add(loginEnd - loginStart);

      let loginSuccess = check(loginResponse, {
        'login status is 200': (r) => r.status === 200,
        'login returns access token': (r) => {
          try {
            const body = JSON.parse(r.body);
            return body.access_token !== undefined;
          } catch (e) {
            return false;
          }
        },
        'login returns refresh token': (r) => {
          try {
            const body = JSON.parse(r.body);
            return body.refresh_token !== undefined;
          } catch (e) {
            return false;
          }
        },
        'login returns user data': (r) => {
          try {
            const body = JSON.parse(r.body);
            return body.user && body.user.email === testUser.email;
          } catch (e) {
            return false;
          }
        },
        'login response time < 500ms': (r) => r.timings.duration < 500,
      });

      if (!loginSuccess) {
        errorRate.add(1);
        console.error(`Login failed for ${testUser.email}: ${loginResponse.body}`);
        return;
      }

      errorRate.add(0);
    });

    group('Invalid Login Attempts', function () {
      const invalidLoginPayload = JSON.stringify({
        email: testUser.email,
        password: 'wrongpassword',
      });

      const invalidLoginParams = {
        headers: {
          'Content-Type': 'application/json',
        },
      };

      let invalidLoginResponse = http.post(
        `${baseUrl}/api/auth/login`,
        invalidLoginPayload,
        invalidLoginParams
      );

      check(invalidLoginResponse, {
        'invalid login status is 401': (r) => r.status === 401,
        'invalid login returns error message': (r) => {
          try {
            const body = JSON.parse(r.body);
            return body.error !== undefined;
          } catch (e) {
            return false;
          }
        },
        'invalid login response time < 500ms': (r) => r.timings.duration < 500,
      });
    });

    group('Malformed Requests', function () {
      // Test missing fields
      let missingFieldsResponse = http.post(
        `${baseUrl}/api/auth/login`,
        JSON.stringify({ email: testUser.email }),
        { headers: { 'Content-Type': 'application/json' } }
      );

      check(missingFieldsResponse, {
        'missing fields returns 400 or 422': (r) => r.status === 400 || r.status === 422,
      });

      // Test invalid JSON
      let invalidJsonResponse = http.post(
        `${baseUrl}/api/auth/login`,
        '{ invalid json',
        { headers: { 'Content-Type': 'application/json' } }
      );

      check(invalidJsonResponse, {
        'invalid JSON returns 400': (r) => r.status === 400,
      });

      // Test invalid email format
      let invalidEmailResponse = http.post(
        `${baseUrl}/api/auth/login`,
        JSON.stringify({ email: 'not-an-email', password: 'password123' }),
        { headers: { 'Content-Type': 'application/json' } }
      );

      check(invalidEmailResponse, {
        'invalid email format handled gracefully': (r) => r.status >= 400 && r.status < 500,
      });
    });
  });
}

export function teardown(data) {
  console.log('Authentication tests completed');
}