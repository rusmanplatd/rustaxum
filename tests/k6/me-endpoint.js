import http from 'k6/http';
import { check, group } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';

// Custom metrics
export let errorRate = new Rate('errors');
export let meEndpointDuration = new Trend('me_endpoint_duration');
export let authFailures = new Counter('auth_failures');
export let successfulRequests = new Counter('successful_requests');

// Test configuration
export let options = {
  stages: [
    { duration: '10s', target: 5 },    // Ramp up to 5 users
    { duration: '30s', target: 15 },   // Ramp up to 15 users
    { duration: '1m', target: 15 },    // Stay at 15 users
    { duration: '10s', target: 5 },    // Ramp down to 5 users
    { duration: '10s', target: 0 },    // Ramp down to 0 users
  ],
  thresholds: {
    http_req_duration: ['p(95)<300'],  // 95% of requests must complete below 300ms
    errors: ['rate<0.02'],             // Error rate must be less than 2%
    me_endpoint_duration: ['p(99)<500'], // 99% of /me requests under 500ms
  },
};

// Base URL configuration
const BASE_URL = __ENV.BASE_URL || 'http://localhost:3000';

// Test user for authentication
const TEST_USER = {
  name: 'ME Test User',
  email: 'me-test@example.com',
  password: 'password123'
};

export function setup() {
  console.log(`Running /me endpoint tests against ${BASE_URL}`);

  // Health check
  let healthCheck = http.get(`${BASE_URL}/api/docs`);
  if (healthCheck.status !== 200) {
    console.error(`Health check failed. Status: ${healthCheck.status}`);
    return null;
  }

  // Create test user for authentication
  const registrationPayload = JSON.stringify({
    name: TEST_USER.name,
    email: `${Date.now()}_${TEST_USER.email}`,
    password: TEST_USER.password,
  });

  const registrationParams = {
    headers: { 'Content-Type': 'application/json' },
  };

  let registrationResponse = http.post(
    `${BASE_URL}/api/auth/register`,
    registrationPayload,
    registrationParams
  );

  if (registrationResponse.status !== 201) {
    console.error(`Failed to create test user: ${registrationResponse.body}`);
    return null;
  }

  const registrationData = JSON.parse(registrationResponse.body);

  console.log('Setup completed successfully');
  return {
    baseUrl: BASE_URL,
    testUser: {
      ...TEST_USER,
      email: `${Date.now()}_${TEST_USER.email}`,
    },
    accessToken: registrationData.access_token,
    refreshToken: registrationData.refresh_token,
  };
}

export default function (data) {
  if (!data) {
    console.error('Setup failed, skipping test');
    return;
  }

  const { baseUrl, testUser, accessToken } = data;

  group('/me Endpoint Tests', function () {

    group('Valid Authentication', function () {
      const meParams = {
        headers: {
          'Authorization': `Bearer ${accessToken}`,
          'Content-Type': 'application/json',
        },
      };

      const meStart = Date.now();
      let meResponse = http.get(`${baseUrl}/api/me`, meParams);
      const meEnd = Date.now();

      meEndpointDuration.add(meEnd - meStart);

      let meSuccess = check(meResponse, {
        '/me status is 200': (r) => r.status === 200,
        '/me returns user object': (r) => {
          try {
            const body = JSON.parse(r.body);
            return body.user !== undefined;
          } catch (e) {
            return false;
          }
        },
        '/me returns correct user email': (r) => {
          try {
            const body = JSON.parse(r.body);
            return body.user && body.user.email === testUser.email;
          } catch (e) {
            return false;
          }
        },
        '/me returns user id': (r) => {
          try {
            const body = JSON.parse(r.body);
            return body.user && body.user.id !== undefined;
          } catch (e) {
            return false;
          }
        },
        '/me returns user name': (r) => {
          try {
            const body = JSON.parse(r.body);
            return body.user && body.user.name === testUser.name;
          } catch (e) {
            return false;
          }
        },
        '/me includes timestamps': (r) => {
          try {
            const body = JSON.parse(r.body);
            return body.user && body.user.created_at && body.user.updated_at;
          } catch (e) {
            return false;
          }
        },
        '/me response time < 200ms': (r) => r.timings.duration < 200,
        '/me content-type is JSON': (r) => r.headers['Content-Type'] && r.headers['Content-Type'].includes('application/json'),
      });

      if (meSuccess) {
        successfulRequests.add(1);
        errorRate.add(0);
      } else {
        errorRate.add(1);
        console.error(`/me endpoint failed: ${meResponse.body}`);
      }
    });

    group('Invalid Authentication', function () {

      group('Invalid JWT Token', function () {
        const invalidTokenParams = {
          headers: {
            'Authorization': 'Bearer invalid.jwt.token',
            'Content-Type': 'application/json',
          },
        };

        let invalidTokenResponse = http.get(`${baseUrl}/api/me`, invalidTokenParams);

        check(invalidTokenResponse, {
          'invalid token returns 401': (r) => r.status === 401,
          'invalid token returns error message': (r) => {
            try {
              const body = JSON.parse(r.body);
              return body.error !== undefined;
            } catch (e) {
              return false;
            }
          },
          'invalid token response time < 200ms': (r) => r.timings.duration < 200,
        });

        authFailures.add(1);
      });

      group('Malformed JWT Token', function () {
        const malformedTokenParams = {
          headers: {
            'Authorization': 'Bearer malformed-token',
            'Content-Type': 'application/json',
          },
        };

        let malformedTokenResponse = http.get(`${baseUrl}/api/me`, malformedTokenParams);

        check(malformedTokenResponse, {
          'malformed token returns 401': (r) => r.status === 401,
          'malformed token returns error message': (r) => {
            try {
              const body = JSON.parse(r.body);
              return body.error !== undefined;
            } catch (e) {
              return false;
            }
          },
        });

        authFailures.add(1);
      });

      group('Expired JWT Token', function () {
        // Create an expired token (this is a mock expired token)
        const expiredToken = 'eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIwMUs2NTZQUFEwRFc3Q0NXRTBGQ1RWM1dCOSIsImV4cCI6MTU5MDUwMDAwMCwiaWF0IjoxNTkwNDAwMDAwLCJqdGkiOiIwMUs2NTdHMVZDQjJDUzVQTVoyRVBLREU3RCJ9.fake-signature';

        const expiredTokenParams = {
          headers: {
            'Authorization': `Bearer ${expiredToken}`,
            'Content-Type': 'application/json',
          },
        };

        let expiredTokenResponse = http.get(`${baseUrl}/api/me`, expiredTokenParams);

        check(expiredTokenResponse, {
          'expired token returns 401': (r) => r.status === 401,
          'expired token returns error message': (r) => {
            try {
              const body = JSON.parse(r.body);
              return body.error !== undefined;
            } catch (e) {
              return false;
            }
          },
        });

        authFailures.add(1);
      });

      group('Missing Authorization Header', function () {
        const noAuthParams = {
          headers: {
            'Content-Type': 'application/json',
          },
        };

        let noAuthResponse = http.get(`${baseUrl}/api/me`, noAuthParams);

        check(noAuthResponse, {
          'no auth header returns 401': (r) => r.status === 401,
          'no auth header returns error message': (r) => {
            try {
              const body = JSON.parse(r.body);
              return body.error !== undefined && body.message !== undefined;
            } catch (e) {
              return false;
            }
          },
          'no auth header response time < 100ms': (r) => r.timings.duration < 100,
        });

        authFailures.add(1);
      });

      group('Wrong Authorization Format', function () {
        const wrongFormatParams = {
          headers: {
            'Authorization': 'Basic dGVzdDp0ZXN0', // Basic auth instead of Bearer
            'Content-Type': 'application/json',
          },
        };

        let wrongFormatResponse = http.get(`${baseUrl}/api/me`, wrongFormatParams);

        check(wrongFormatResponse, {
          'wrong auth format returns 401': (r) => r.status === 401,
          'wrong auth format returns error message': (r) => {
            try {
              const body = JSON.parse(r.body);
              return body.error !== undefined;
            } catch (e) {
              return false;
            }
          },
        });

        authFailures.add(1);
      });
    });

    group('HTTP Method Tests', function () {
      const validAuthParams = {
        headers: {
          'Authorization': `Bearer ${accessToken}`,
          'Content-Type': 'application/json',
        },
      };

      // Test unsupported HTTP methods
      group('POST Method (Not Allowed)', function () {
        let postResponse = http.post(`${baseUrl}/api/me`, '', validAuthParams);

        check(postResponse, {
          'POST /me returns 405 Method Not Allowed': (r) => r.status === 405,
        });
      });

      group('PUT Method (Not Allowed)', function () {
        let putResponse = http.put(`${baseUrl}/api/me`, '', validAuthParams);

        check(putResponse, {
          'PUT /me returns 405 Method Not Allowed': (r) => r.status === 405,
        });
      });

      group('DELETE Method (Not Allowed)', function () {
        let deleteResponse = http.del(`${baseUrl}/api/me`, '', validAuthParams);

        check(deleteResponse, {
          'DELETE /me returns 405 Method Not Allowed': (r) => r.status === 405,
        });
      });
    });

    group('Concurrent Access', function () {
      // Simulate concurrent requests with the same token
      const validAuthParams = {
        headers: {
          'Authorization': `Bearer ${accessToken}`,
          'Content-Type': 'application/json',
        },
      };

      // Make multiple simultaneous requests
      let responses = http.batch([
        ['GET', `${baseUrl}/api/me`, null, validAuthParams],
        ['GET', `${baseUrl}/api/me`, null, validAuthParams],
        ['GET', `${baseUrl}/api/me`, null, validAuthParams],
      ]);

      for (let i = 0; i < responses.length; i++) {
        check(responses[i], {
          [`concurrent request ${i + 1} status is 200`]: (r) => r.status === 200,
          [`concurrent request ${i + 1} returns user data`]: (r) => {
            try {
              const body = JSON.parse(r.body);
              return body.user && body.user.email === testUser.email;
            } catch (e) {
              return false;
            }
          },
        });
      }
    });
  });
}

export function teardown(data) {
  if (data) {
    console.log('Test Summary:');
    console.log(`- Base URL: ${data.baseUrl}`);
    console.log(`- Test User: ${data.testUser.email}`);
    console.log('/me endpoint tests completed');
  }
}