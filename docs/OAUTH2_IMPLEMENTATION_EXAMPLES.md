# OAuth 2.1 Implementation Examples

Complete implementation examples for integrating with the Rustaxum OAuth 2.1 server.

## Table of Contents

1. [Web Application Integration](#web-application-integration)
2. [Mobile Application (iOS/Android)](#mobile-application-integration)
3. [Single Page Application (SPA)](#single-page-application-spa)
4. [Server-to-Server Integration](#server-to-server-integration)
5. [Personal Access Tokens](#personal-access-tokens)
6. [Middleware Implementation](#middleware-implementation)
7. [Testing Examples](#testing-examples)

---

## Web Application Integration

### Frontend (JavaScript/TypeScript)

#### PKCE Authorization Flow

```typescript
// oauth-client.ts
interface PKCEChallenge {
  codeVerifier: string;
  codeChallenge: string;
}

interface TokenResponse {
  access_token: string;
  token_type: string;
  expires_in: number;
  refresh_token: string;
  scope: string;
}

class OAuth2Client {
  private clientId: string;
  private redirectUri: string;
  private authServerUrl: string;

  constructor(clientId: string, redirectUri: string, authServerUrl: string) {
    this.clientId = clientId;
    this.redirectUri = redirectUri;
    this.authServerUrl = authServerUrl;
  }

  // Generate PKCE challenge according to RFC 7636
  private async generatePKCE(): Promise<PKCEChallenge> {
    const codeVerifier = this.base64URLEncode(
      crypto.getRandomValues(new Uint8Array(32))
    );

    const encoder = new TextEncoder();
    const data = encoder.encode(codeVerifier);
    const digest = await crypto.subtle.digest('SHA-256', data);

    const codeChallenge = this.base64URLEncode(new Uint8Array(digest));

    return { codeVerifier, codeChallenge };
  }

  private base64URLEncode(buffer: Uint8Array): string {
    return btoa(String.fromCharCode(...buffer))
      .replace(/\+/g, '-')
      .replace(/\//g, '_')
      .replace(/=/g, '');
  }

  // Step 1: Redirect to authorization server
  async authorize(scopes: string[] = ['read']): Promise<void> {
    const { codeVerifier, codeChallenge } = await this.generatePKCE();
    const state = this.base64URLEncode(crypto.getRandomValues(new Uint8Array(16)));

    // Store PKCE verifier and state for callback
    sessionStorage.setItem('oauth_code_verifier', codeVerifier);
    sessionStorage.setItem('oauth_state', state);

    const authUrl = new URL('/oauth/authorize', this.authServerUrl);
    authUrl.searchParams.set('response_type', 'code');
    authUrl.searchParams.set('client_id', this.clientId);
    authUrl.searchParams.set('redirect_uri', this.redirectUri);
    authUrl.searchParams.set('code_challenge', codeChallenge);
    authUrl.searchParams.set('code_challenge_method', 'S256');
    authUrl.searchParams.set('scope', scopes.join(' '));
    authUrl.searchParams.set('state', state);

    window.location.href = authUrl.toString();
  }

  // Step 2: Handle callback and exchange code for tokens
  async handleCallback(): Promise<TokenResponse> {
    const urlParams = new URLSearchParams(window.location.search);
    const code = urlParams.get('code');
    const state = urlParams.get('state');
    const error = urlParams.get('error');

    if (error) {
      throw new Error(`OAuth error: ${error} - ${urlParams.get('error_description')}`);
    }

    if (!code) {
      throw new Error('Authorization code not found in callback');
    }

    const savedState = sessionStorage.getItem('oauth_state');
    if (state !== savedState) {
      throw new Error('State parameter mismatch - possible CSRF attack');
    }

    const codeVerifier = sessionStorage.getItem('oauth_code_verifier');
    if (!codeVerifier) {
      throw new Error('PKCE code verifier not found');
    }

    // Exchange code for tokens
    const response = await fetch('/oauth/token', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: new URLSearchParams({
        grant_type: 'authorization_code',
        code,
        client_id: this.clientId,
        redirect_uri: this.redirectUri,
        code_verifier: codeVerifier,
      }),
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(`Token exchange failed: ${error.error_description}`);
    }

    const tokens: TokenResponse = await response.json();

    // Store tokens securely
    this.storeTokens(tokens);

    // Clean up temporary storage
    sessionStorage.removeItem('oauth_code_verifier');
    sessionStorage.removeItem('oauth_state');

    return tokens;
  }

  // Refresh access token
  async refreshToken(): Promise<TokenResponse> {
    const refreshToken = this.getRefreshToken();
    if (!refreshToken) {
      throw new Error('No refresh token available');
    }

    const response = await fetch('/oauth/token', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: new URLSearchParams({
        grant_type: 'refresh_token',
        refresh_token: refreshToken,
        client_id: this.clientId,
      }),
    });

    if (!response.ok) {
      const error = await response.json();
      if (response.status === 400 && error.error === 'invalid_grant') {
        // Refresh token expired, need to reauthorize
        this.clearTokens();
        await this.authorize();
        throw new Error('Refresh token expired, redirecting to login');
      }
      throw new Error(`Token refresh failed: ${error.error_description}`);
    }

    const tokens: TokenResponse = await response.json();
    this.storeTokens(tokens);
    return tokens;
  }

  // Make authenticated API request with automatic token refresh
  async apiRequest(url: string, options: RequestInit = {}): Promise<Response> {
    let token = this.getAccessToken();

    if (!token) {
      throw new Error('No access token available');
    }

    // First attempt
    let response = await fetch(url, {
      ...options,
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
        ...options.headers,
      },
    });

    // If unauthorized, try to refresh token
    if (response.status === 401) {
      try {
        await this.refreshToken();
        token = this.getAccessToken();

        // Retry with new token
        response = await fetch(url, {
          ...options,
          headers: {
            'Authorization': `Bearer ${token}`,
            'Content-Type': 'application/json',
            ...options.headers,
          },
        });
      } catch (error) {
        // Refresh failed, redirect to login
        await this.authorize();
        throw error;
      }
    }

    return response;
  }

  private storeTokens(tokens: TokenResponse): void {
    const expiresAt = Date.now() + (tokens.expires_in * 1000);

    sessionStorage.setItem('oauth_access_token', tokens.access_token);
    sessionStorage.setItem('oauth_refresh_token', tokens.refresh_token);
    sessionStorage.setItem('oauth_token_expires_at', expiresAt.toString());
    sessionStorage.setItem('oauth_scope', tokens.scope);
  }

  private getAccessToken(): string | null {
    const token = sessionStorage.getItem('oauth_access_token');
    const expiresAt = sessionStorage.getItem('oauth_token_expires_at');

    if (!token || !expiresAt) {
      return null;
    }

    // Check if token is expired (with 5 minute buffer)
    const now = Date.now();
    const expires = parseInt(expiresAt, 10);

    if (now >= expires - 300000) { // 5 minutes buffer
      return null;
    }

    return token;
  }

  private getRefreshToken(): string | null {
    return sessionStorage.getItem('oauth_refresh_token');
  }

  private clearTokens(): void {
    sessionStorage.removeItem('oauth_access_token');
    sessionStorage.removeItem('oauth_refresh_token');
    sessionStorage.removeItem('oauth_token_expires_at');
    sessionStorage.removeItem('oauth_scope');
  }

  // Check if user is authenticated
  isAuthenticated(): boolean {
    return this.getAccessToken() !== null;
  }

  // Logout
  async logout(): Promise<void> {
    const token = this.getAccessToken();

    if (token) {
      // Revoke token on server
      try {
        await fetch('/oauth/revoke', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/x-www-form-urlencoded',
          },
          body: new URLSearchParams({
            token,
          }),
        });
      } catch (error) {
        console.warn('Token revocation failed:', error);
      }
    }

    this.clearTokens();
    window.location.href = '/login';
  }
}

// Usage example
const oauth = new OAuth2Client(
  'your-client-id',
  'https://yourapp.com/oauth/callback',
  'https://auth.yourcompany.com'
);

// On login page
document.getElementById('login-btn')?.addEventListener('click', () => {
  oauth.authorize(['read', 'write']);
});

// On callback page
if (window.location.pathname === '/oauth/callback') {
  oauth.handleCallback()
    .then(() => {
      window.location.href = '/dashboard';
    })
    .catch(error => {
      console.error('OAuth callback failed:', error);
      window.location.href = '/login?error=oauth_failed';
    });
}

// Making API requests
async function loadUserData() {
  try {
    const response = await oauth.apiRequest('/api/user');
    const userData = await response.json();
    return userData;
  } catch (error) {
    console.error('Failed to load user data:', error);
    throw error;
  }
}
```

### Backend (Node.js Express)

```javascript
// server.js
const express = require('express');
const session = require('express-session');
const axios = require('axios');

const app = express();

app.use(session({
  secret: 'your-session-secret',
  resave: false,
  saveUninitialized: false,
  cookie: { secure: false, httpOnly: true, maxAge: 24 * 60 * 60 * 1000 }
}));

// OAuth configuration
const oauthConfig = {
  clientId: process.env.OAUTH_CLIENT_ID,
  clientSecret: process.env.OAUTH_CLIENT_SECRET,
  redirectUri: process.env.OAUTH_REDIRECT_URI,
  authServerUrl: process.env.OAUTH_AUTH_SERVER_URL,
};

// OAuth callback endpoint
app.get('/oauth/callback', async (req, res) => {
  const { code, state, error } = req.query;

  if (error) {
    return res.status(400).json({
      error: 'OAuth authorization failed',
      details: req.query.error_description
    });
  }

  if (!code) {
    return res.status(400).json({ error: 'Authorization code missing' });
  }

  // Verify state parameter
  if (state !== req.session.oauthState) {
    return res.status(400).json({ error: 'State parameter mismatch' });
  }

  try {
    // Exchange code for tokens
    const tokenResponse = await axios.post(`${oauthConfig.authServerUrl}/oauth/token`, {
      grant_type: 'authorization_code',
      code,
      client_id: oauthConfig.clientId,
      client_secret: oauthConfig.clientSecret,
      redirect_uri: oauthConfig.redirectUri,
      code_verifier: req.session.pkceVerifier,
    }, {
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
    });

    const { access_token, refresh_token, expires_in } = tokenResponse.data;

    // Store tokens in session
    req.session.accessToken = access_token;
    req.session.refreshToken = refresh_token;
    req.session.tokenExpiresAt = Date.now() + (expires_in * 1000);

    // Clean up temporary data
    delete req.session.oauthState;
    delete req.session.pkceVerifier;

    res.redirect('/dashboard');
  } catch (error) {
    console.error('Token exchange failed:', error.response?.data || error.message);
    res.status(500).json({ error: 'Token exchange failed' });
  }
});

// Middleware to check authentication
function requireAuth(req, res, next) {
  if (!req.session.accessToken) {
    return res.status(401).json({ error: 'Not authenticated' });
  }

  // Check if token is expired
  if (Date.now() >= req.session.tokenExpiresAt - 300000) { // 5 min buffer
    return refreshTokenMiddleware(req, res, next);
  }

  next();
}

// Token refresh middleware
async function refreshTokenMiddleware(req, res, next) {
  if (!req.session.refreshToken) {
    return res.status(401).json({ error: 'No refresh token available' });
  }

  try {
    const tokenResponse = await axios.post(`${oauthConfig.authServerUrl}/oauth/token`, {
      grant_type: 'refresh_token',
      refresh_token: req.session.refreshToken,
      client_id: oauthConfig.clientId,
      client_secret: oauthConfig.clientSecret,
    }, {
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
    });

    const { access_token, refresh_token, expires_in } = tokenResponse.data;

    req.session.accessToken = access_token;
    req.session.refreshToken = refresh_token;
    req.session.tokenExpiresAt = Date.now() + (expires_in * 1000);

    next();
  } catch (error) {
    console.error('Token refresh failed:', error.response?.data || error.message);
    delete req.session.accessToken;
    delete req.session.refreshToken;
    delete req.session.tokenExpiresAt;
    res.status(401).json({ error: 'Token refresh failed' });
  }
}

// Protected API endpoint
app.get('/api/user', requireAuth, async (req, res) => {
  try {
    const userResponse = await axios.get(`${oauthConfig.authServerUrl}/api/user`, {
      headers: {
        'Authorization': `Bearer ${req.session.accessToken}`,
      },
    });

    res.json(userResponse.data);
  } catch (error) {
    console.error('Failed to fetch user data:', error.response?.data || error.message);
    res.status(500).json({ error: 'Failed to fetch user data' });
  }
});

app.listen(3000, () => {
  console.log('Server running on http://localhost:3000');
});
```

---

## Mobile Application Integration

### iOS (Swift)

```swift
// OAuthManager.swift
import Foundation
import AuthenticationServices
import CryptoKit

class OAuthManager: NSObject, ObservableObject {
    @Published var isAuthenticated = false
    @Published var accessToken: String?

    private let clientId: String
    private let redirectUri: String
    private let authServerUrl: String

    private var codeVerifier: String?
    private var refreshToken: String?

    init(clientId: String, redirectUri: String, authServerUrl: String) {
        self.clientId = clientId
        self.redirectUri = redirectUri
        self.authServerUrl = authServerUrl
        super.init()
        loadStoredTokens()
    }

    // MARK: - PKCE Implementation

    private func generatePKCE() -> (verifier: String, challenge: String) {
        // Generate code verifier (128 characters, base64url encoded)
        let codeVerifierData = Data((0..<32).map { _ in UInt8.random(in: 0...255) })
        let codeVerifier = codeVerifierData.base64URLEncodedString()

        // Generate code challenge (SHA256 hash of verifier)
        let challengeData = Data(codeVerifier.utf8)
        let hash = SHA256.hash(data: challengeData)
        let codeChallenge = Data(hash).base64URLEncodedString()

        return (verifier: codeVerifier, challenge: codeChallenge)
    }

    // MARK: - OAuth Flow

    func authenticate() {
        let pkce = generatePKCE()
        self.codeVerifier = pkce.verifier

        let state = Data((0..<16).map { _ in UInt8.random(in: 0...255) })
            .base64URLEncodedString()

        var components = URLComponents(string: "\\(authServerUrl)/oauth/authorize")!
        components.queryItems = [
            URLQueryItem(name: "response_type", value: "code"),
            URLQueryItem(name: "client_id", value: clientId),
            URLQueryItem(name: "redirect_uri", value: redirectUri),
            URLQueryItem(name: "code_challenge", value: pkce.challenge),
            URLQueryItem(name: "code_challenge_method", value: "S256"),
            URLQueryItem(name: "scope", value: "read write"),
            URLQueryItem(name: "state", value: state)
        ]

        let session = ASWebAuthenticationSession(
            url: components.url!,
            callbackURLScheme: URL(string: redirectUri)?.scheme
        ) { [weak self] callbackURL, error in
            if let error = error {
                print("Authentication error: \\(error)")
                return
            }

            guard let callbackURL = callbackURL else {
                print("No callback URL received")
                return
            }

            self?.handleCallback(url: callbackURL, expectedState: state)
        }

        session.presentationContextProvider = self
        session.prefersEphemeralWebBrowserSession = false
        session.start()
    }

    private func handleCallback(url: URL, expectedState: String) {
        guard let components = URLComponents(url: url, resolvingAgainstBaseURL: false),
              let queryItems = components.queryItems else {
            print("Invalid callback URL")
            return
        }

        let params = Dictionary(uniqueKeysWithValues: queryItems.compactMap { item in
            guard let value = item.value else { return nil }
            return (item.name, value)
        })

        // Check for errors
        if let error = params["error"] {
            print("OAuth error: \\(error) - \\(params["error_description"] ?? "")")
            return
        }

        // Verify state parameter
        guard let state = params["state"], state == expectedState else {
            print("State parameter mismatch")
            return
        }

        guard let code = params["code"], let codeVerifier = self.codeVerifier else {
            print("Authorization code or PKCE verifier missing")
            return
        }

        exchangeCodeForTokens(code: code, codeVerifier: codeVerifier)
    }

    private func exchangeCodeForTokens(code: String, codeVerifier: String) {
        guard let url = URL(string: "\\(authServerUrl)/oauth/token") else { return }

        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/x-www-form-urlencoded", forHTTPHeaderField: "Content-Type")

        let parameters = [
            "grant_type": "authorization_code",
            "code": code,
            "client_id": clientId,
            "redirect_uri": redirectUri,
            "code_verifier": codeVerifier
        ]

        let paramString = parameters.map { "\\($0.key)=\\($0.value.addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed) ?? "")" }
            .joined(separator: "&")

        request.httpBody = paramString.data(using: .utf8)

        URLSession.shared.dataTask(with: request) { [weak self] data, response, error in
            if let error = error {
                print("Token exchange error: \\(error)")
                return
            }

            guard let data = data else {
                print("No data received from token endpoint")
                return
            }

            do {
                let tokenResponse = try JSONDecoder().decode(TokenResponse.self, from: data)

                DispatchQueue.main.async {
                    self?.accessToken = tokenResponse.accessToken
                    self?.refreshToken = tokenResponse.refreshToken
                    self?.isAuthenticated = true
                    self?.storeTokens(tokenResponse)
                }
            } catch {
                print("Failed to decode token response: \\(error)")
            }
        }.resume()
    }

    // MARK: - Token Management

    private func storeTokens(_ tokens: TokenResponse) {
        let keychain = Keychain()
        keychain.set(tokens.accessToken, forKey: "oauth_access_token")
        keychain.set(tokens.refreshToken, forKey: "oauth_refresh_token")

        let expiresAt = Date().addingTimeInterval(TimeInterval(tokens.expiresIn))
        UserDefaults.standard.set(expiresAt, forKey: "oauth_token_expires_at")
    }

    private func loadStoredTokens() {
        let keychain = Keychain()

        if let accessToken = keychain.get("oauth_access_token"),
           let expiresAt = UserDefaults.standard.object(forKey: "oauth_token_expires_at") as? Date,
           Date() < expiresAt.addingTimeInterval(-300) { // 5 min buffer

            self.accessToken = accessToken
            self.refreshToken = keychain.get("oauth_refresh_token")
            self.isAuthenticated = true
        }
    }

    func refreshAccessToken() {
        guard let refreshToken = self.refreshToken,
              let url = URL(string: "\\(authServerUrl)/oauth/token") else {
            logout()
            return
        }

        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/x-www-form-urlencoded", forHTTPHeaderField: "Content-Type")

        let parameters = [
            "grant_type": "refresh_token",
            "refresh_token": refreshToken,
            "client_id": clientId
        ]

        let paramString = parameters.map { "\\($0.key)=\\($0.value.addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed) ?? "")" }
            .joined(separator: "&")

        request.httpBody = paramString.data(using: .utf8)

        URLSession.shared.dataTask(with: request) { [weak self] data, response, error in
            if let httpResponse = response as? HTTPURLResponse, httpResponse.statusCode == 401 {
                DispatchQueue.main.async {
                    self?.logout()
                }
                return
            }

            guard let data = data else { return }

            do {
                let tokenResponse = try JSONDecoder().decode(TokenResponse.self, from: data)

                DispatchQueue.main.async {
                    self?.accessToken = tokenResponse.accessToken
                    self?.refreshToken = tokenResponse.refreshToken
                    self?.storeTokens(tokenResponse)
                }
            } catch {
                print("Failed to refresh token: \\(error)")
                DispatchQueue.main.async {
                    self?.logout()
                }
            }
        }.resume()
    }

    func logout() {
        let keychain = Keychain()
        keychain.delete("oauth_access_token")
        keychain.delete("oauth_refresh_token")
        UserDefaults.standard.removeObject(forKey: "oauth_token_expires_at")

        accessToken = nil
        refreshToken = nil
        isAuthenticated = false
    }

    // MARK: - API Requests

    func makeAuthenticatedRequest(url: URL, method: String = "GET", body: Data? = nil) async throws -> Data {
        guard let accessToken = accessToken else {
            throw OAuthError.notAuthenticated
        }

        var request = URLRequest(url: url)
        request.httpMethod = method
        request.setValue("Bearer \\(accessToken)", forHTTPHeaderField: "Authorization")
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")

        if let body = body {
            request.httpBody = body
        }

        do {
            let (data, response) = try await URLSession.shared.data(for: request)

            if let httpResponse = response as? HTTPURLResponse, httpResponse.statusCode == 401 {
                // Token expired, try to refresh
                refreshAccessToken()
                throw OAuthError.tokenExpired
            }

            return data
        } catch {
            throw error
        }
    }
}

// MARK: - Supporting Types

struct TokenResponse: Codable {
    let accessToken: String
    let tokenType: String
    let expiresIn: Int
    let refreshToken: String
    let scope: String

    enum CodingKeys: String, CodingKey {
        case accessToken = "access_token"
        case tokenType = "token_type"
        case expiresIn = "expires_in"
        case refreshToken = "refresh_token"
        case scope
    }
}

enum OAuthError: Error {
    case notAuthenticated
    case tokenExpired
    case invalidResponse
}

// MARK: - Extensions

extension Data {
    func base64URLEncodedString() -> String {
        return self.base64EncodedString()
            .replacingOccurrences(of: "+", with: "-")
            .replacingOccurrences(of: "/", with: "_")
            .replacingOccurrences(of: "=", with: "")
    }
}

extension OAuthManager: ASWebAuthenticationPresentationContextProviding {
    func presentationAnchor(for session: ASWebAuthenticationSession) -> ASPresentationAnchor {
        return UIApplication.shared.windows.first { $0.isKeyWindow } ?? ASPresentationAnchor()
    }
}

// Simple Keychain wrapper
class Keychain {
    func set(_ value: String, forKey key: String) {
        let data = value.data(using: .utf8)!

        let query = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrAccount as String: key,
            kSecValueData as String: data
        ] as [String: Any]

        SecItemDelete(query as CFDictionary)
        SecItemAdd(query as CFDictionary, nil)
    }

    func get(_ key: String) -> String? {
        let query = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrAccount as String: key,
            kSecReturnData as String: kCFBooleanTrue!,
            kSecMatchLimit as String: kSecMatchLimitOne
        ] as [String: Any]

        var dataTypeRef: AnyObject? = nil

        let status: OSStatus = SecItemCopyMatching(query as CFDictionary, &dataTypeRef)

        if status == noErr, let data = dataTypeRef as? Data {
            return String(data: data, encoding: .utf8)
        }

        return nil
    }

    func delete(_ key: String) {
        let query = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrAccount as String: key
        ] as [String: Any]

        SecItemDelete(query as CFDictionary)
    }
}
```

### Android (Kotlin)

```kotlin
// OAuthManager.kt
import android.content.Context
import android.content.SharedPreferences
import android.net.Uri
import androidx.browser.customtabs.CustomTabsIntent
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.FormBody
import okhttp3.OkHttpClient
import okhttp3.Request
import com.google.gson.Gson
import com.google.gson.annotations.SerializedName
import java.security.MessageDigest
import java.security.SecureRandom
import android.util.Base64

class OAuthManager(
    private val context: Context,
    private val clientId: String,
    private val redirectUri: String,
    private val authServerUrl: String
) {
    private val prefs: SharedPreferences = context.getSharedPreferences("oauth_prefs", Context.MODE_PRIVATE)
    private val httpClient = OkHttpClient()
    private val gson = Gson()

    var accessToken: String? = null
        private set

    var isAuthenticated: Boolean = false
        private set

    init {
        loadStoredTokens()
    }

    // PKCE Implementation
    private data class PKCEChallenge(
        val verifier: String,
        val challenge: String
    )

    private fun generatePKCE(): PKCEChallenge {
        val codeVerifier = ByteArray(32).apply {
            SecureRandom().nextBytes(this)
        }.let { Base64.encodeToString(it, Base64.URL_SAFE or Base64.NO_WRAP) }

        val digest = MessageDigest.getInstance("SHA-256")
        val hash = digest.digest(codeVerifier.toByteArray())
        val codeChallenge = Base64.encodeToString(hash, Base64.URL_SAFE or Base64.NO_WRAP)

        return PKCEChallenge(codeVerifier, codeChallenge)
    }

    // OAuth Flow
    fun authenticate() {
        val pkce = generatePKCE()
        val state = ByteArray(16).apply {
            SecureRandom().nextBytes(this)
        }.let { Base64.encodeToString(it, Base64.URL_SAFE or Base64.NO_WRAP) }

        // Store PKCE verifier and state
        prefs.edit()
            .putString("pkce_verifier", pkce.verifier)
            .putString("oauth_state", state)
            .apply()

        val authUrl = Uri.Builder()
            .scheme("https")
            .authority(Uri.parse(authServerUrl).authority)
            .path("/oauth/authorize")
            .appendQueryParameter("response_type", "code")
            .appendQueryParameter("client_id", clientId)
            .appendQueryParameter("redirect_uri", redirectUri)
            .appendQueryParameter("code_challenge", pkce.challenge)
            .appendQueryParameter("code_challenge_method", "S256")
            .appendQueryParameter("scope", "read write")
            .appendQueryParameter("state", state)
            .build()

        // Launch Custom Tab
        val customTabsIntent = CustomTabsIntent.Builder()
            .setShowTitle(true)
            .build()

        customTabsIntent.launchUrl(context, authUrl)
    }

    suspend fun handleCallback(callbackUri: Uri): Result<Unit> = withContext(Dispatchers.IO) {
        try {
            val code = callbackUri.getQueryParameter("code")
            val state = callbackUri.getQueryParameter("state")
            val error = callbackUri.getQueryParameter("error")

            if (error != null) {
                val errorDescription = callbackUri.getQueryParameter("error_description")
                return@withContext Result.failure(Exception("OAuth error: $error - $errorDescription"))
            }

            if (code == null) {
                return@withContext Result.failure(Exception("Authorization code not found"))
            }

            val savedState = prefs.getString("oauth_state", null)
            if (state != savedState) {
                return@withContext Result.failure(Exception("State parameter mismatch"))
            }

            val codeVerifier = prefs.getString("pkce_verifier", null)
            if (codeVerifier == null) {
                return@withContext Result.failure(Exception("PKCE verifier not found"))
            }

            val tokenResponse = exchangeCodeForTokens(code, codeVerifier)
            storeTokens(tokenResponse)

            // Clean up temporary data
            prefs.edit()
                .remove("pkce_verifier")
                .remove("oauth_state")
                .apply()

            Result.success(Unit)
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    private suspend fun exchangeCodeForTokens(code: String, codeVerifier: String): TokenResponse = withContext(Dispatchers.IO) {
        val requestBody = FormBody.Builder()
            .add("grant_type", "authorization_code")
            .add("code", code)
            .add("client_id", clientId)
            .add("redirect_uri", redirectUri)
            .add("code_verifier", codeVerifier)
            .build()

        val request = Request.Builder()
            .url("$authServerUrl/oauth/token")
            .post(requestBody)
            .build()

        val response = httpClient.newCall(request).execute()

        if (!response.isSuccessful) {
            val errorBody = response.body?.string() ?: "Unknown error"
            throw Exception("Token exchange failed: $errorBody")
        }

        val responseBody = response.body?.string() ?: throw Exception("Empty response")
        gson.fromJson(responseBody, TokenResponse::class.java)
    }

    suspend fun refreshToken(): Result<Unit> = withContext(Dispatchers.IO) {
        try {
            val refreshToken = prefs.getString("refresh_token", null)
                ?: return@withContext Result.failure(Exception("No refresh token available"))

            val requestBody = FormBody.Builder()
                .add("grant_type", "refresh_token")
                .add("refresh_token", refreshToken)
                .add("client_id", clientId)
                .build()

            val request = Request.Builder()
                .url("$authServerUrl/oauth/token")
                .post(requestBody)
                .build()

            val response = httpClient.newCall(request).execute()

            if (response.code == 401) {
                logout()
                return@withContext Result.failure(Exception("Refresh token expired"))
            }

            if (!response.isSuccessful) {
                val errorBody = response.body?.string() ?: "Unknown error"
                return@withContext Result.failure(Exception("Token refresh failed: $errorBody"))
            }

            val responseBody = response.body?.string() ?: throw Exception("Empty response")
            val tokenResponse = gson.fromJson(responseBody, TokenResponse::class.java)

            storeTokens(tokenResponse)
            Result.success(Unit)
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    suspend fun makeAuthenticatedRequest(url: String, method: String = "GET", body: String? = null): Result<String> = withContext(Dispatchers.IO) {
        try {
            val token = accessToken ?: return@withContext Result.failure(Exception("Not authenticated"))

            val requestBuilder = Request.Builder()
                .url(url)
                .addHeader("Authorization", "Bearer $token")
                .addHeader("Content-Type", "application/json")

            when (method.uppercase()) {
                "GET" -> requestBuilder.get()
                "POST" -> requestBuilder.post(okhttp3.RequestBody.create(null, body ?: ""))
                "PUT" -> requestBuilder.put(okhttp3.RequestBody.create(null, body ?: ""))
                "DELETE" -> requestBuilder.delete()
            }

            val response = httpClient.newCall(requestBuilder.build()).execute()

            if (response.code == 401) {
                // Token expired, try to refresh
                val refreshResult = refreshToken()
                if (refreshResult.isFailure) {
                    return@withContext Result.failure(Exception("Token expired and refresh failed"))
                }

                // Retry with new token
                val newToken = accessToken ?: return@withContext Result.failure(Exception("Token refresh failed"))
                val retryRequest = requestBuilder
                    .removeHeader("Authorization")
                    .addHeader("Authorization", "Bearer $newToken")
                    .build()

                val retryResponse = httpClient.newCall(retryRequest).execute()
                val responseBody = retryResponse.body?.string() ?: ""

                if (retryResponse.isSuccessful) {
                    Result.success(responseBody)
                } else {
                    Result.failure(Exception("Request failed: ${retryResponse.code} $responseBody"))
                }
            } else {
                val responseBody = response.body?.string() ?: ""
                if (response.isSuccessful) {
                    Result.success(responseBody)
                } else {
                    Result.failure(Exception("Request failed: ${response.code} $responseBody"))
                }
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    private fun storeTokens(tokens: TokenResponse) {
        val expiresAt = System.currentTimeMillis() + (tokens.expiresIn * 1000L)

        prefs.edit()
            .putString("access_token", tokens.accessToken)
            .putString("refresh_token", tokens.refreshToken)
            .putLong("token_expires_at", expiresAt)
            .putString("scope", tokens.scope)
            .apply()

        accessToken = tokens.accessToken
        isAuthenticated = true
    }

    private fun loadStoredTokens() {
        val token = prefs.getString("access_token", null)
        val expiresAt = prefs.getLong("token_expires_at", 0L)

        if (token != null && System.currentTimeMillis() < expiresAt - 300000L) { // 5 min buffer
            accessToken = token
            isAuthenticated = true
        }
    }

    fun logout() {
        prefs.edit().clear().apply()
        accessToken = null
        isAuthenticated = false
    }
}

// Data classes
data class TokenResponse(
    @SerializedName("access_token") val accessToken: String,
    @SerializedName("token_type") val tokenType: String,
    @SerializedName("expires_in") val expiresIn: Int,
    @SerializedName("refresh_token") val refreshToken: String,
    val scope: String
)

// Usage in Activity
class MainActivity : AppCompatActivity() {
    private lateinit var oauthManager: OAuthManager

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        oauthManager = OAuthManager(
            context = this,
            clientId = "your-client-id",
            redirectUri = "yourapp://oauth/callback",
            authServerUrl = "https://auth.yourcompany.com"
        )

        // Check if this is a callback from OAuth
        if (intent.action == Intent.ACTION_VIEW && intent.data != null) {
            handleOAuthCallback(intent.data!!)
        }
    }

    private fun handleOAuthCallback(uri: Uri) {
        lifecycleScope.launch {
            val result = oauthManager.handleCallback(uri)
            if (result.isSuccess) {
                // Authentication successful
                startActivity(Intent(this@MainActivity, DashboardActivity::class.java))
                finish()
            } else {
                // Handle error
                Toast.makeText(this@MainActivity, "Authentication failed", Toast.LENGTH_LONG).show()
            }
        }
    }

    private fun login() {
        oauthManager.authenticate()
    }
}
```

---

## Single Page Application (SPA)

### React Implementation

```tsx
// hooks/useOAuth.ts
import { useState, useEffect, useCallback, useContext, createContext } from 'react';

interface OAuthConfig {
  clientId: string;
  redirectUri: string;
  authServerUrl: string;
  scopes?: string[];
}

interface TokenResponse {
  access_token: string;
  token_type: string;
  expires_in: number;
  refresh_token: string;
  scope: string;
}

interface OAuthContextType {
  isAuthenticated: boolean;
  isLoading: boolean;
  accessToken: string | null;
  login: () => Promise<void>;
  logout: () => Promise<void>;
  apiRequest: (url: string, options?: RequestInit) => Promise<Response>;
}

const OAuthContext = createContext<OAuthContextType | null>(null);

export const useOAuth = () => {
  const context = useContext(OAuthContext);
  if (!context) {
    throw new Error('useOAuth must be used within an OAuthProvider');
  }
  return context;
};

// PKCE utility functions
const generateCodeVerifier = (): string => {
  const array = new Uint8Array(32);
  crypto.getRandomValues(array);
  return btoa(String.fromCharCode(...array))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=/g, '');
};

const generateCodeChallenge = async (verifier: string): Promise<string> => {
  const encoder = new TextEncoder();
  const data = encoder.encode(verifier);
  const digest = await crypto.subtle.digest('SHA-256', data);
  const array = new Uint8Array(digest);
  return btoa(String.fromCharCode(...array))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=/g, '');
};

export const OAuthProvider: React.FC<{ config: OAuthConfig; children: React.ReactNode }> = ({
  config,
  children,
}) => {
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [accessToken, setAccessToken] = useState<string | null>(null);

  // Check for stored tokens on initialization
  useEffect(() => {
    const token = localStorage.getItem('oauth_access_token');
    const expiresAt = localStorage.getItem('oauth_token_expires_at');

    if (token && expiresAt) {
      const now = Date.now();
      const expires = parseInt(expiresAt, 10);

      if (now < expires - 300000) { // 5 minutes buffer
        setAccessToken(token);
        setIsAuthenticated(true);
      } else {
        // Try to refresh token
        refreshToken();
      }
    }

    setIsLoading(false);
  }, []);

  // Handle OAuth callback
  useEffect(() => {
    const handleCallback = async () => {
      const urlParams = new URLSearchParams(window.location.search);
      const code = urlParams.get('code');
      const state = urlParams.get('state');
      const error = urlParams.get('error');

      if (error) {
        console.error('OAuth error:', error, urlParams.get('error_description'));
        return;
      }

      if (code && window.location.pathname === '/oauth/callback') {
        setIsLoading(true);

        const savedState = sessionStorage.getItem('oauth_state');
        if (state !== savedState) {
          console.error('State parameter mismatch');
          setIsLoading(false);
          return;
        }

        const codeVerifier = sessionStorage.getItem('oauth_code_verifier');
        if (!codeVerifier) {
          console.error('PKCE code verifier not found');
          setIsLoading(false);
          return;
        }

        try {
          await exchangeCodeForTokens(code, codeVerifier);

          // Clean up URL and temporary storage
          sessionStorage.removeItem('oauth_code_verifier');
          sessionStorage.removeItem('oauth_state');
          window.history.replaceState({}, document.title, '/');
        } catch (error) {
          console.error('Token exchange failed:', error);
        }

        setIsLoading(false);
      }
    };

    handleCallback();
  }, []);

  const exchangeCodeForTokens = async (code: string, codeVerifier: string) => {
    const response = await fetch(`${config.authServerUrl}/oauth/token`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: new URLSearchParams({
        grant_type: 'authorization_code',
        code,
        client_id: config.clientId,
        redirect_uri: config.redirectUri,
        code_verifier: codeVerifier,
      }),
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(`Token exchange failed: ${error.error_description}`);
    }

    const tokens: TokenResponse = await response.json();
    storeTokens(tokens);
  };

  const refreshToken = async () => {
    const refreshToken = localStorage.getItem('oauth_refresh_token');
    if (!refreshToken) {
      logout();
      return;
    }

    try {
      const response = await fetch(`${config.authServerUrl}/oauth/token`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded',
        },
        body: new URLSearchParams({
          grant_type: 'refresh_token',
          refresh_token: refreshToken,
          client_id: config.clientId,
        }),
      });

      if (!response.ok) {
        if (response.status === 401) {
          logout();
          return;
        }
        throw new Error('Token refresh failed');
      }

      const tokens: TokenResponse = await response.json();
      storeTokens(tokens);
    } catch (error) {
      console.error('Token refresh failed:', error);
      logout();
    }
  };

  const storeTokens = (tokens: TokenResponse) => {
    const expiresAt = Date.now() + (tokens.expires_in * 1000);

    localStorage.setItem('oauth_access_token', tokens.access_token);
    localStorage.setItem('oauth_refresh_token', tokens.refresh_token);
    localStorage.setItem('oauth_token_expires_at', expiresAt.toString());
    localStorage.setItem('oauth_scope', tokens.scope);

    setAccessToken(tokens.access_token);
    setIsAuthenticated(true);
  };

  const login = useCallback(async () => {
    const codeVerifier = generateCodeVerifier();
    const codeChallenge = await generateCodeChallenge(codeVerifier);
    const state = generateCodeVerifier(); // Reuse for simplicity

    sessionStorage.setItem('oauth_code_verifier', codeVerifier);
    sessionStorage.setItem('oauth_state', state);

    const authUrl = new URL(`${config.authServerUrl}/oauth/authorize`);
    authUrl.searchParams.set('response_type', 'code');
    authUrl.searchParams.set('client_id', config.clientId);
    authUrl.searchParams.set('redirect_uri', config.redirectUri);
    authUrl.searchParams.set('code_challenge', codeChallenge);
    authUrl.searchParams.set('code_challenge_method', 'S256');
    authUrl.searchParams.set('scope', (config.scopes || ['read']).join(' '));
    authUrl.searchParams.set('state', state);

    window.location.href = authUrl.toString();
  }, [config]);

  const logout = useCallback(async () => {
    const token = localStorage.getItem('oauth_access_token');

    if (token) {
      try {
        await fetch(`${config.authServerUrl}/oauth/revoke`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/x-www-form-urlencoded',
          },
          body: new URLSearchParams({
            token,
          }),
        });
      } catch (error) {
        console.warn('Token revocation failed:', error);
      }
    }

    localStorage.removeItem('oauth_access_token');
    localStorage.removeItem('oauth_refresh_token');
    localStorage.removeItem('oauth_token_expires_at');
    localStorage.removeItem('oauth_scope');

    setAccessToken(null);
    setIsAuthenticated(false);
  }, [config.authServerUrl]);

  const apiRequest = useCallback(async (url: string, options: RequestInit = {}): Promise<Response> => {
    let token = accessToken;

    if (!token) {
      throw new Error('Not authenticated');
    }

    // First attempt
    let response = await fetch(url, {
      ...options,
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
        ...options.headers,
      },
    });

    // If unauthorized, try to refresh token
    if (response.status === 401) {
      await refreshToken();
      token = localStorage.getItem('oauth_access_token');

      if (token) {
        // Retry with new token
        response = await fetch(url, {
          ...options,
          headers: {
            'Authorization': `Bearer ${token}`,
            'Content-Type': 'application/json',
            ...options.headers,
          },
        });
      } else {
        throw new Error('Authentication failed');
      }
    }

    return response;
  }, [accessToken]);

  const value: OAuthContextType = {
    isAuthenticated,
    isLoading,
    accessToken,
    login,
    logout,
    apiRequest,
  };

  return <OAuthContext.Provider value={value}>{children}</OAuthContext.Provider>;
};

// Usage components
const LoginPage: React.FC = () => {
  const { login } = useOAuth();

  return (
    <div className="login-page">
      <h1>Welcome</h1>
      <button onClick={login} className="login-button">
        Login with OAuth
      </button>
    </div>
  );
};

const Dashboard: React.FC = () => {
  const { logout, apiRequest } = useOAuth();
  const [userData, setUserData] = useState(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const loadUserData = async () => {
      try {
        const response = await apiRequest('/api/user');
        const data = await response.json();
        setUserData(data);
      } catch (error) {
        console.error('Failed to load user data:', error);
      } finally {
        setLoading(false);
      }
    };

    loadUserData();
  }, [apiRequest]);

  if (loading) {
    return <div>Loading...</div>;
  }

  return (
    <div className="dashboard">
      <header>
        <h1>Dashboard</h1>
        <button onClick={logout}>Logout</button>
      </header>
      <main>
        <pre>{JSON.stringify(userData, null, 2)}</pre>
      </main>
    </div>
  );
};

// App component
const App: React.FC = () => {
  const { isAuthenticated, isLoading } = useOAuth();

  if (isLoading) {
    return <div className="loading">Loading...</div>;
  }

  return (
    <div className="app">
      {isAuthenticated ? <Dashboard /> : <LoginPage />}
    </div>
  );
};

// Root component with provider
const Root: React.FC = () => {
  const oauthConfig: OAuthConfig = {
    clientId: 'your-spa-client-id',
    redirectUri: `${window.location.origin}/oauth/callback`,
    authServerUrl: 'https://auth.yourcompany.com',
    scopes: ['read', 'write'],
  };

  return (
    <OAuthProvider config={oauthConfig}>
      <App />
    </OAuthProvider>
  );
};

export default Root;
```

---

## Server-to-Server Integration

### Python Client

```python
# oauth_client.py
import requests
import time
import json
from typing import Optional, Dict, Any
from dataclasses import dataclass
from urllib.parse import urljoin

@dataclass
class TokenResponse:
    access_token: str
    token_type: str
    expires_in: int
    scope: str
    expires_at: float = 0

    def __post_init__(self):
        self.expires_at = time.time() + self.expires_in - 300  # 5 min buffer

    def is_expired(self) -> bool:
        return time.time() >= self.expires_at

class OAuth2ClientCredentialsClient:
    """OAuth 2.1 Client Credentials Grant implementation."""

    def __init__(
        self,
        client_id: str,
        client_secret: str,
        auth_server_url: str,
        scopes: Optional[list] = None,
        timeout: int = 30
    ):
        self.client_id = client_id
        self.client_secret = client_secret
        self.auth_server_url = auth_server_url.rstrip('/')
        self.scopes = scopes or []
        self.timeout = timeout
        self.session = requests.Session()
        self._token: Optional[TokenResponse] = None

    def authenticate(self) -> TokenResponse:
        """Obtain access token using client credentials grant."""
        token_url = urljoin(self.auth_server_url + '/', 'oauth/token')

        data = {
            'grant_type': 'client_credentials',
            'client_id': self.client_id,
            'client_secret': self.client_secret,
        }

        if self.scopes:
            data['scope'] = ' '.join(self.scopes)

        try:
            response = self.session.post(
                token_url,
                data=data,
                headers={'Content-Type': 'application/x-www-form-urlencoded'},
                timeout=self.timeout
            )
            response.raise_for_status()

            token_data = response.json()
            self._token = TokenResponse(**token_data)

            return self._token

        except requests.exceptions.RequestException as e:
            raise OAuth2Error(f"Authentication failed: {e}") from e

    def get_valid_token(self) -> str:
        """Get a valid access token, refreshing if necessary."""
        if self._token is None or self._token.is_expired():
            self.authenticate()

        return self._token.access_token

    def make_request(
        self,
        method: str,
        url: str,
        **kwargs
    ) -> requests.Response:
        """Make authenticated API request."""
        token = self.get_valid_token()

        headers = kwargs.get('headers', {})
        headers['Authorization'] = f'Bearer {token}'
        kwargs['headers'] = headers

        try:
            response = self.session.request(method, url, timeout=self.timeout, **kwargs)

            # If token expired, try once more with new token
            if response.status_code == 401:
                self._token = None  # Force re-authentication
                token = self.get_valid_token()
                headers['Authorization'] = f'Bearer {token}'
                response = self.session.request(method, url, timeout=self.timeout, **kwargs)

            return response

        except requests.exceptions.RequestException as e:
            raise OAuth2Error(f"API request failed: {e}") from e

    def get(self, url: str, **kwargs) -> requests.Response:
        """Make authenticated GET request."""
        return self.make_request('GET', url, **kwargs)

    def post(self, url: str, **kwargs) -> requests.Response:
        """Make authenticated POST request."""
        return self.make_request('POST', url, **kwargs)

    def put(self, url: str, **kwargs) -> requests.Response:
        """Make authenticated PUT request."""
        return self.make_request('PUT', url, **kwargs)

    def delete(self, url: str, **kwargs) -> requests.Response:
        """Make authenticated DELETE request."""
        return self.make_request('DELETE', url, **kwargs)

    def introspect_token(self, token: Optional[str] = None) -> Dict[str, Any]:
        """Introspect access token to get metadata."""
        token = token or self.get_valid_token()
        introspect_url = urljoin(self.auth_server_url + '/', 'oauth/introspect')

        data = {
            'token': token,
            'token_type_hint': 'access_token'
        }

        try:
            response = self.session.post(
                introspect_url,
                data=data,
                headers={'Content-Type': 'application/x-www-form-urlencoded'},
                timeout=self.timeout
            )
            response.raise_for_status()

            return response.json()

        except requests.exceptions.RequestException as e:
            raise OAuth2Error(f"Token introspection failed: {e}") from e

    def revoke_token(self, token: Optional[str] = None) -> bool:
        """Revoke access token."""
        token = token or (self._token.access_token if self._token else None)
        if not token:
            return True  # No token to revoke

        revoke_url = urljoin(self.auth_server_url + '/', 'oauth/revoke')

        data = {'token': token}

        try:
            response = self.session.post(
                revoke_url,
                data=data,
                headers={'Content-Type': 'application/x-www-form-urlencoded'},
                timeout=self.timeout
            )

            # OAuth spec says revoke should return 200 even for invalid tokens
            self._token = None
            return response.status_code == 200

        except requests.exceptions.RequestException as e:
            raise OAuth2Error(f"Token revocation failed: {e}") from e

class OAuth2Error(Exception):
    """OAuth 2.1 related errors."""
    pass

# Usage examples
if __name__ == "__main__":
    # Initialize client
    oauth_client = OAuth2ClientCredentialsClient(
        client_id="your-server-client-id",
        client_secret="your-server-client-secret",
        auth_server_url="https://auth.yourcompany.com",
        scopes=["read", "write", "admin"]
    )

    try:
        # Authenticate
        token = oauth_client.authenticate()
        print(f"Authenticated successfully. Token expires at: {time.ctime(token.expires_at)}")

        # Make API requests
        response = oauth_client.get("https://api.yourcompany.com/users")
        if response.status_code == 200:
            users = response.json()
            print(f"Retrieved {len(users)} users")

        # Create new user
        new_user_data = {
            "name": "John Doe",
            "email": "john@example.com"
        }

        response = oauth_client.post(
            "https://api.yourcompany.com/users",
            json=new_user_data
        )

        if response.status_code == 201:
            user = response.json()
            print(f"Created user: {user['id']}")

        # Introspect token
        token_info = oauth_client.introspect_token()
        print(f"Token info: {json.dumps(token_info, indent=2)}")

        # Revoke token when done
        oauth_client.revoke_token()
        print("Token revoked successfully")

    except OAuth2Error as e:
        print(f"OAuth error: {e}")
    except Exception as e:
        print(f"Unexpected error: {e}")
```

### Go Client

```go
// oauth_client.go
package main

import (
    "bytes"
    "encoding/json"
    "fmt"
    "io"
    "net/http"
    "net/url"
    "strings"
    "sync"
    "time"
)

type TokenResponse struct {
    AccessToken string `json:"access_token"`
    TokenType   string `json:"token_type"`
    ExpiresIn   int    `json:"expires_in"`
    Scope       string `json:"scope"`
    ExpiresAt   time.Time
}

type OAuth2Client struct {
    clientID     string
    clientSecret string
    authServerURL string
    scopes       []string
    httpClient   *http.Client
    token        *TokenResponse
    tokenMutex   sync.RWMutex
}

type OAuth2Error struct {
    Error            string `json:"error"`
    ErrorDescription string `json:"error_description"`
}

func (e OAuth2Error) String() string {
    if e.ErrorDescription != "" {
        return fmt.Sprintf("%s: %s", e.Error, e.ErrorDescription)
    }
    return e.Error
}

func NewOAuth2Client(clientID, clientSecret, authServerURL string, scopes []string) *OAuth2Client {
    return &OAuth2Client{
        clientID:      clientID,
        clientSecret:  clientSecret,
        authServerURL: strings.TrimSuffix(authServerURL, "/"),
        scopes:        scopes,
        httpClient:    &http.Client{Timeout: 30 * time.Second},
    }
}

func (c *OAuth2Client) Authenticate() (*TokenResponse, error) {
    c.tokenMutex.Lock()
    defer c.tokenMutex.Unlock()

    tokenURL := c.authServerURL + "/oauth/token"

    data := url.Values{}
    data.Set("grant_type", "client_credentials")
    data.Set("client_id", c.clientID)
    data.Set("client_secret", c.clientSecret)

    if len(c.scopes) > 0 {
        data.Set("scope", strings.Join(c.scopes, " "))
    }

    req, err := http.NewRequest("POST", tokenURL, strings.NewReader(data.Encode()))
    if err != nil {
        return nil, fmt.Errorf("failed to create request: %w", err)
    }

    req.Header.Set("Content-Type", "application/x-www-form-urlencoded")

    resp, err := c.httpClient.Do(req)
    if err != nil {
        return nil, fmt.Errorf("authentication request failed: %w", err)
    }
    defer resp.Body.Close()

    body, err := io.ReadAll(resp.Body)
    if err != nil {
        return nil, fmt.Errorf("failed to read response: %w", err)
    }

    if resp.StatusCode != http.StatusOK {
        var oauthErr OAuth2Error
        if json.Unmarshal(body, &oauthErr) == nil {
            return nil, fmt.Errorf("authentication failed: %s", oauthErr.String())
        }
        return nil, fmt.Errorf("authentication failed with status %d: %s", resp.StatusCode, string(body))
    }

    var token TokenResponse
    if err := json.Unmarshal(body, &token); err != nil {
        return nil, fmt.Errorf("failed to parse token response: %w", err)
    }

    // Set expiration time with 5-minute buffer
    token.ExpiresAt = time.Now().Add(time.Duration(token.ExpiresIn-300) * time.Second)
    c.token = &token

    return &token, nil
}

func (c *OAuth2Client) getValidToken() (string, error) {
    c.tokenMutex.RLock()
    if c.token != nil && time.Now().Before(c.token.ExpiresAt) {
        token := c.token.AccessToken
        c.tokenMutex.RUnlock()
        return token, nil
    }
    c.tokenMutex.RUnlock()

    // Need to authenticate
    _, err := c.Authenticate()
    if err != nil {
        return "", err
    }

    c.tokenMutex.RLock()
    token := c.token.AccessToken
    c.tokenMutex.RUnlock()

    return token, nil
}

func (c *OAuth2Client) MakeRequest(method, url string, body []byte) (*http.Response, error) {
    token, err := c.getValidToken()
    if err != nil {
        return nil, fmt.Errorf("failed to get valid token: %w", err)
    }

    var reqBody io.Reader
    if body != nil {
        reqBody = bytes.NewReader(body)
    }

    req, err := http.NewRequest(method, url, reqBody)
    if err != nil {
        return nil, fmt.Errorf("failed to create request: %w", err)
    }

    req.Header.Set("Authorization", "Bearer "+token)
    req.Header.Set("Content-Type", "application/json")

    resp, err := c.httpClient.Do(req)
    if err != nil {
        return nil, fmt.Errorf("request failed: %w", err)
    }

    // If unauthorized, try once more with new token
    if resp.StatusCode == http.StatusUnauthorized {
        resp.Body.Close()

        // Force re-authentication
        c.tokenMutex.Lock()
        c.token = nil
        c.tokenMutex.Unlock()

        token, err := c.getValidToken()
        if err != nil {
            return nil, fmt.Errorf("failed to refresh token: %w", err)
        }

        // Retry request
        if body != nil {
            reqBody = bytes.NewReader(body)
        }

        req, err := http.NewRequest(method, url, reqBody)
        if err != nil {
            return nil, fmt.Errorf("failed to create retry request: %w", err)
        }

        req.Header.Set("Authorization", "Bearer "+token)
        req.Header.Set("Content-Type", "application/json")

        resp, err = c.httpClient.Do(req)
        if err != nil {
            return nil, fmt.Errorf("retry request failed: %w", err)
        }
    }

    return resp, nil
}

func (c *OAuth2Client) Get(url string) (*http.Response, error) {
    return c.MakeRequest("GET", url, nil)
}

func (c *OAuth2Client) Post(url string, body []byte) (*http.Response, error) {
    return c.MakeRequest("POST", url, body)
}

func (c *OAuth2Client) Put(url string, body []byte) (*http.Response, error) {
    return c.MakeRequest("PUT", url, body)
}

func (c *OAuth2Client) Delete(url string) (*http.Response, error) {
    return c.MakeRequest("DELETE", url, nil)
}

func (c *OAuth2Client) IntrospectToken(token string) (map[string]interface{}, error) {
    if token == "" {
        var err error
        token, err = c.getValidToken()
        if err != nil {
            return nil, fmt.Errorf("failed to get token for introspection: %w", err)
        }
    }

    introspectURL := c.authServerURL + "/oauth/introspect"

    data := url.Values{}
    data.Set("token", token)
    data.Set("token_type_hint", "access_token")

    req, err := http.NewRequest("POST", introspectURL, strings.NewReader(data.Encode()))
    if err != nil {
        return nil, fmt.Errorf("failed to create introspect request: %w", err)
    }

    req.Header.Set("Content-Type", "application/x-www-form-urlencoded")

    resp, err := c.httpClient.Do(req)
    if err != nil {
        return nil, fmt.Errorf("introspect request failed: %w", err)
    }
    defer resp.Body.Close()

    body, err := io.ReadAll(resp.Body)
    if err != nil {
        return nil, fmt.Errorf("failed to read introspect response: %w", err)
    }

    if resp.StatusCode != http.StatusOK {
        return nil, fmt.Errorf("introspect failed with status %d: %s", resp.StatusCode, string(body))
    }

    var result map[string]interface{}
    if err := json.Unmarshal(body, &result); err != nil {
        return nil, fmt.Errorf("failed to parse introspect response: %w", err)
    }

    return result, nil
}

func (c *OAuth2Client) RevokeToken(token string) error {
    if token == "" {
        c.tokenMutex.RLock()
        if c.token != nil {
            token = c.token.AccessToken
        }
        c.tokenMutex.RUnlock()

        if token == "" {
            return nil // No token to revoke
        }
    }

    revokeURL := c.authServerURL + "/oauth/revoke"

    data := url.Values{}
    data.Set("token", token)

    req, err := http.NewRequest("POST", revokeURL, strings.NewReader(data.Encode()))
    if err != nil {
        return fmt.Errorf("failed to create revoke request: %w", err)
    }

    req.Header.Set("Content-Type", "application/x-www-form-urlencoded")

    resp, err := c.httpClient.Do(req)
    if err != nil {
        return fmt.Errorf("revoke request failed: %w", err)
    }
    defer resp.Body.Close()

    // Clear cached token
    c.tokenMutex.Lock()
    c.token = nil
    c.tokenMutex.Unlock()

    // OAuth spec says revoke should return 200 even for invalid tokens
    return nil
}

// Usage example
func main() {
    client := NewOAuth2Client(
        "your-server-client-id",
        "your-server-client-secret",
        "https://auth.yourcompany.com",
        []string{"read", "write", "admin"},
    )

    // Authenticate
    token, err := client.Authenticate()
    if err != nil {
        fmt.Printf("Authentication failed: %v\n", err)
        return
    }

    fmt.Printf("Authenticated successfully. Token expires at: %v\n", token.ExpiresAt)

    // Make API requests
    resp, err := client.Get("https://api.yourcompany.com/users")
    if err != nil {
        fmt.Printf("GET request failed: %v\n", err)
        return
    }
    defer resp.Body.Close()

    if resp.StatusCode == http.StatusOK {
        body, _ := io.ReadAll(resp.Body)
        fmt.Printf("API Response: %s\n", string(body))
    }

    // Create new user
    userData := map[string]string{
        "name":  "John Doe",
        "email": "john@example.com",
    }

    userJSON, _ := json.Marshal(userData)
    resp, err = client.Post("https://api.yourcompany.com/users", userJSON)
    if err != nil {
        fmt.Printf("POST request failed: %v\n", err)
        return
    }
    defer resp.Body.Close()

    if resp.StatusCode == http.StatusCreated {
        fmt.Println("User created successfully")
    }

    // Introspect token
    tokenInfo, err := client.IntrospectToken("")
    if err != nil {
        fmt.Printf("Token introspection failed: %v\n", err)
        return
    }

    fmt.Printf("Token info: %+v\n", tokenInfo)

    // Revoke token
    if err := client.RevokeToken(""); err != nil {
        fmt.Printf("Token revocation failed: %v\n", err)
        return
    }

    fmt.Println("Token revoked successfully")
}
```

---

## Personal Access Tokens

### Creating Personal Access Tokens

```bash
# CLI method
cargo run --bin artisan passport personal-access-token \
  --user-id "01HPQR3TUVWXYZ" \
  --name "CLI Access Token" \
  --scopes "read,write" \
  --expires-in 86400
```

### API Method

```typescript
// Create personal access token via API
async function createPersonalAccessToken(
  name: string,
  scopes: string[],
  expiresIn?: number
): Promise<string> {
  const response = await fetch('/oauth/personal-access-tokens', {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${userAccessToken}`,
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      name,
      scopes,
      expires_in: expiresIn,
    }),
  });

  if (!response.ok) {
    throw new Error('Failed to create personal access token');
  }

  const result = await response.json();
  return result.access_token;
}

// Usage
const apiToken = await createPersonalAccessToken('My API Token', ['read', 'write'], 86400);
console.log('Your API token:', apiToken);
```

---

## Testing Examples

### Integration Tests

```typescript
// oauth.test.ts
import { describe, it, expect, beforeAll, afterAll } from '@jest/globals';

describe('OAuth 2.1 Integration Tests', () => {
  let testClient: OAuth2TestClient;

  beforeAll(async () => {
    testClient = new OAuth2TestClient({
      clientId: 'test-client-id',
      clientSecret: 'test-client-secret',
      authServerUrl: 'http://localhost:3000',
    });
  });

  afterAll(async () => {
    await testClient.cleanup();
  });

  describe('Client Credentials Flow', () => {
    it('should authenticate successfully', async () => {
      const token = await testClient.authenticate();

      expect(token.access_token).toBeDefined();
      expect(token.token_type).toBe('Bearer');
      expect(token.expires_in).toBeGreaterThan(0);
    });

    it('should make authenticated API requests', async () => {
      const response = await testClient.get('/api/test');
      expect(response.status).toBe(200);
    });

    it('should handle token expiration', async () => {
      // Force token expiration
      testClient.expireToken();

      const response = await testClient.get('/api/test');
      expect(response.status).toBe(200); // Should auto-refresh
    });
  });

  describe('Authorization Code Flow', () => {
    it('should handle PKCE correctly', async () => {
      const { codeVerifier, codeChallenge } = await testClient.generatePKCE();

      expect(codeVerifier).toMatch(/^[A-Za-z0-9_-]{128}$/);
      expect(codeChallenge).toMatch(/^[A-Za-z0-9_-]{43}$/);

      // Verify challenge generation
      const expectedChallenge = await testClient.generateCodeChallenge(codeVerifier);
      expect(codeChallenge).toBe(expectedChallenge);
    });

    it('should reject requests without PKCE', async () => {
      const response = await fetch('/oauth/authorize?' + new URLSearchParams({
        response_type: 'code',
        client_id: 'test-client-id',
        redirect_uri: 'http://localhost/callback',
        // Missing code_challenge
      }));

      expect(response.status).toBe(302);

      const location = response.headers.get('location');
      expect(location).toContain('error=invalid_request');
      expect(location).toContain('PKCE+code_challenge+is+required');
    });
  });

  describe('Token Introspection', () => {
    it('should return token metadata', async () => {
      const token = await testClient.authenticate();
      const metadata = await testClient.introspect(token.access_token);

      expect(metadata.active).toBe(true);
      expect(metadata.client_id).toBe('test-client-id');
      expect(metadata.exp).toBeGreaterThan(Date.now() / 1000);
    });

    it('should return inactive for invalid tokens', async () => {
      const metadata = await testClient.introspect('invalid-token');
      expect(metadata.active).toBe(false);
    });
  });

  describe('Token Revocation', () => {
    it('should revoke tokens successfully', async () => {
      const token = await testClient.authenticate();

      await testClient.revoke(token.access_token);

      const metadata = await testClient.introspect(token.access_token);
      expect(metadata.active).toBe(false);
    });
  });
});

// Test utility class
class OAuth2TestClient {
  constructor(private config: OAuth2Config) {}

  async authenticate() {
    const response = await fetch('/oauth/token', {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: new URLSearchParams({
        grant_type: 'client_credentials',
        client_id: this.config.clientId,
        client_secret: this.config.clientSecret,
        scope: 'read write',
      }),
    });

    return response.json();
  }

  async generatePKCE() {
    const codeVerifier = btoa(String.fromCharCode(...crypto.getRandomValues(new Uint8Array(32))))
      .replace(/\+/g, '-')
      .replace(/\//g, '_')
      .replace(/=/g, '');

    const encoder = new TextEncoder();
    const data = encoder.encode(codeVerifier);
    const digest = await crypto.subtle.digest('SHA-256', data);
    const codeChallenge = btoa(String.fromCharCode(...new Uint8Array(digest)))
      .replace(/\+/g, '-')
      .replace(/\//g, '_')
      .replace(/=/g, '');

    return { codeVerifier, codeChallenge };
  }

  async generateCodeChallenge(verifier: string): Promise<string> {
    const encoder = new TextEncoder();
    const data = encoder.encode(verifier);
    const digest = await crypto.subtle.digest('SHA-256', data);
    return btoa(String.fromCharCode(...new Uint8Array(digest)))
      .replace(/\+/g, '-')
      .replace(/\//g, '_')
      .replace(/=/g, '');
  }

  // ... other test methods
}
```

### Load Testing

```typescript
// load-test.ts - Using Artillery.io
import { check } from 'k6';
import http from 'k6/http';

export let options = {
  stages: [
    { duration: '30s', target: 10 },   // Warm up
    { duration: '1m', target: 50 },    // Ramp up to 50 users
    { duration: '2m', target: 100 },   // Stay at 100 users
    { duration: '30s', target: 0 },    // Ramp down
  ],
  thresholds: {
    http_req_duration: ['p(95)<500'], // 95% of requests under 500ms
    http_req_failed: ['rate<0.02'],   // Error rate under 2%
  },
};

// Test data
const CLIENT_ID = 'load-test-client';
const CLIENT_SECRET = 'load-test-secret';
const AUTH_SERVER = 'https://auth.example.com';

export default function () {
  // Client credentials flow
  const tokenResponse = http.post(`${AUTH_SERVER}/oauth/token`, {
    grant_type: 'client_credentials',
    client_id: CLIENT_ID,
    client_secret: CLIENT_SECRET,
    scope: 'read',
  }, {
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
  });

  check(tokenResponse, {
    'token request successful': (r) => r.status === 200,
    'has access token': (r) => r.json('access_token') !== undefined,
  });

  if (tokenResponse.status === 200) {
    const token = tokenResponse.json('access_token');

    // Make authenticated API request
    const apiResponse = http.get('https://api.example.com/users', {
      headers: { 'Authorization': `Bearer ${token}` },
    });

    check(apiResponse, {
      'API request successful': (r) => r.status === 200,
      'response time OK': (r) => r.timings.duration < 500,
    });

    // Token introspection
    const introspectResponse = http.post(`${AUTH_SERVER}/oauth/introspect`, {
      token: token,
      token_type_hint: 'access_token',
    }, {
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
    });

    check(introspectResponse, {
      'introspection successful': (r) => r.status === 200,
      'token is active': (r) => r.json('active') === true,
    });
  }
}
```

---

## RFC 8628: Device Authorization Grant

### Device Flow for Input-Constrained Devices

#### Device Application (Smart TV, IoT Device)

```typescript
// device-client.ts
interface DeviceAuthResponse {
  device_code: string;
  user_code: string;
  verification_uri: string;
  verification_uri_complete?: string;
  expires_in: number;
  interval: number;
}

interface DeviceTokenResponse {
  access_token?: string;
  token_type?: string;
  expires_in?: number;
  refresh_token?: string;
  scope?: string;
  error?: string;
  error_description?: string;
}

class DeviceFlowClient {
  private clientId: string;
  private authServerUrl: string;

  constructor(clientId: string, authServerUrl: string) {
    this.clientId = clientId;
    this.authServerUrl = authServerUrl;
  }

  // Step 1: Request device authorization
  async requestDeviceAuthorization(scope?: string): Promise<DeviceAuthResponse> {
    const params = new URLSearchParams({
      client_id: this.clientId,
    });

    if (scope) {
      params.append('scope', scope);
    }

    const response = await fetch(`${this.authServerUrl}/oauth/device/code`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: params.toString(),
    });

    if (!response.ok) {
      throw new Error(`Device authorization failed: ${response.statusText}`);
    }

    return response.json();
  }

  // Step 2: Poll for token (with proper interval timing)
  async pollForToken(
    deviceCode: string,
    interval: number = 5
  ): Promise<DeviceTokenResponse> {
    return new Promise((resolve, reject) => {
      const poll = async () => {
        try {
          const params = new URLSearchParams({
            grant_type: 'urn:ietf:params:oauth:grant-type:device_code',
            device_code: deviceCode,
            client_id: this.clientId,
          });

          const response = await fetch(`${this.authServerUrl}/oauth/token`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
            body: params.toString(),
          });

          const result = await response.json();

          if (response.ok && result.access_token) {
            resolve(result);
            return;
          }

          // Handle different error responses
          switch (result.error) {
            case 'authorization_pending':
              // Continue polling
              setTimeout(poll, interval * 1000);
              break;
            case 'slow_down':
              // Increase polling interval
              setTimeout(poll, (interval + 5) * 1000);
              break;
            case 'expired_token':
            case 'access_denied':
              reject(new Error(`Device authorization ${result.error}: ${result.error_description}`));
              break;
            default:
              reject(new Error(`Unknown error: ${result.error}`));
          }
        } catch (error) {
          reject(error);
        }
      };

      poll();
    });
  }

  // Complete device flow
  async authorize(scope?: string): Promise<DeviceTokenResponse> {
    console.log('Starting device authorization flow...');

    // Step 1: Get device code
    const deviceAuth = await this.requestDeviceAuthorization(scope);

    console.log(`\n🔗 Please visit: ${deviceAuth.verification_uri}`);
    console.log(`📱 Enter code: ${deviceAuth.user_code}`);

    if (deviceAuth.verification_uri_complete) {
      console.log(`📎 Or visit: ${deviceAuth.verification_uri_complete}`);
    }

    console.log(`⏱️  Code expires in ${Math.floor(deviceAuth.expires_in / 60)} minutes`);
    console.log('Waiting for authorization...\n');

    // Step 2: Poll for token
    return this.pollForToken(deviceAuth.device_code, deviceAuth.interval);
  }
}

// Usage example
async function main() {
  const client = new DeviceFlowClient(
    '01HPQS4VWXYZ01',
    'https://auth.rustaxum.dev'
  );

  try {
    const tokenResponse = await client.authorize('read write');
    console.log('✅ Authorization successful!');
    console.log('Access Token:', tokenResponse.access_token);
    console.log('Expires in:', tokenResponse.expires_in, 'seconds');
  } catch (error) {
    console.error('❌ Authorization failed:', error.message);
  }
}

main();
```

#### Python Implementation for IoT Devices

```python
# device_flow.py
import time
import requests
from typing import Optional, Dict, Any

class DeviceFlowClient:
    def __init__(self, client_id: str, auth_server_url: str):
        self.client_id = client_id
        self.auth_server_url = auth_server_url

    def request_device_authorization(self, scope: Optional[str] = None) -> Dict[str, Any]:
        """Step 1: Request device authorization"""
        data = {'client_id': self.client_id}
        if scope:
            data['scope'] = scope

        response = requests.post(
            f"{self.auth_server_url}/oauth/device/code",
            data=data,
            headers={'Content-Type': 'application/x-www-form-urlencoded'}
        )
        response.raise_for_status()
        return response.json()

    def poll_for_token(self, device_code: str, interval: int = 5) -> Dict[str, Any]:
        """Step 2: Poll for access token"""
        while True:
            data = {
                'grant_type': 'urn:ietf:params:oauth:grant-type:device_code',
                'device_code': device_code,
                'client_id': self.client_id
            }

            response = requests.post(
                f"{self.auth_server_url}/oauth/token",
                data=data,
                headers={'Content-Type': 'application/x-www-form-urlencoded'}
            )

            result = response.json()

            if response.ok and 'access_token' in result:
                return result

            error = result.get('error')
            if error == 'authorization_pending':
                time.sleep(interval)
                continue
            elif error == 'slow_down':
                interval += 5
                time.sleep(interval)
                continue
            elif error in ['expired_token', 'access_denied']:
                raise Exception(f"Authorization {error}: {result.get('error_description')}")
            else:
                raise Exception(f"Unknown error: {error}")

    def authorize(self, scope: Optional[str] = None) -> Dict[str, Any]:
        """Complete device authorization flow"""
        print("Starting device authorization flow...")

        # Get device code
        device_auth = self.request_device_authorization(scope)

        print(f"\n🔗 Please visit: {device_auth['verification_uri']}")
        print(f"📱 Enter code: {device_auth['user_code']}")

        if 'verification_uri_complete' in device_auth:
            print(f"📎 Or visit: {device_auth['verification_uri_complete']}")

        print(f"⏱️  Code expires in {device_auth['expires_in'] // 60} minutes")
        print("Waiting for authorization...\n")

        # Poll for token
        return self.poll_for_token(device_auth['device_code'], device_auth['interval'])

# Usage
if __name__ == "__main__":
    client = DeviceFlowClient(
        client_id='01HPQS4VWXYZ01',
        auth_server_url='https://auth.rustaxum.dev'
    )

    try:
        token_response = client.authorize('read write')
        print("✅ Authorization successful!")
        print(f"Access Token: {token_response['access_token']}")
        print(f"Expires in: {token_response['expires_in']} seconds")
    except Exception as e:
        print(f"❌ Authorization failed: {e}")
```

---

## RFC 9449: DPoP (Demonstrating Proof of Possession)

### DPoP-Enhanced Token Security

#### TypeScript/JavaScript Implementation

```typescript
// dpop-client.ts
import { createHash } from 'crypto';

interface JWK {
  kty: string;
  crv?: string;
  x?: string;
  y?: string;
  n?: string;
  e?: string;
}

interface DPoPHeader {
  alg: string;
  typ: string;
  jwk: JWK;
}

interface DPoPClaims {
  jti: string;
  htm: string;
  htu: string;
  iat: number;
  ath?: string; // Access token hash
  nonce?: string;
}

class DPoPClient {
  private keyPair: CryptoKeyPair;
  private jwk: JWK;

  constructor() {
    // Will be initialized in init()
  }

  async init(): Promise<void> {
    // Generate EC P-256 key pair for DPoP
    this.keyPair = await crypto.subtle.generateKey(
      {
        name: 'ECDSA',
        namedCurve: 'P-256',
      },
      false, // extractable
      ['sign', 'verify']
    );

    // Export JWK for DPoP proof header
    const publicKeyJwk = await crypto.subtle.exportKey('jwk', this.keyPair.publicKey);
    this.jwk = {
      kty: publicKeyJwk.kty!,
      crv: publicKeyJwk.crv!,
      x: publicKeyJwk.x!,
      y: publicKeyJwk.y!,
    };
  }

  private async createDPoPProof(
    httpMethod: string,
    httpUrl: string,
    accessToken?: string,
    nonce?: string
  ): Promise<string> {
    const jti = crypto.getRandomValues(new Uint8Array(16));
    const jtiB64 = btoa(String.fromCharCode(...jti));

    const header: DPoPHeader = {
      alg: 'ES256',
      typ: 'dpop+jwt',
      jwk: this.jwk,
    };

    const claims: DPoPClaims = {
      jti: jtiB64,
      htm: httpMethod,
      htu: httpUrl,
      iat: Math.floor(Date.now() / 1000),
    };

    // Add access token hash if provided
    if (accessToken) {
      const encoder = new TextEncoder();
      const tokenBytes = encoder.encode(accessToken);
      const hashBuffer = await crypto.subtle.digest('SHA-256', tokenBytes);
      const hashArray = new Uint8Array(hashBuffer);
      claims.ath = btoa(String.fromCharCode(...hashArray))
        .replace(/\+/g, '-')
        .replace(/\//g, '_')
        .replace(/=/g, '');
    }

    if (nonce) {
      claims.nonce = nonce;
    }

    // Create JWT
    const headerB64 = btoa(JSON.stringify(header))
      .replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '');
    const claimsB64 = btoa(JSON.stringify(claims))
      .replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '');

    const signingInput = `${headerB64}.${claimsB64}`;
    const encoder = new TextEncoder();
    const signature = await crypto.subtle.sign(
      { name: 'ECDSA', hash: 'SHA-256' },
      this.keyPair.privateKey,
      encoder.encode(signingInput)
    );

    const signatureArray = new Uint8Array(signature);
    const signatureB64 = btoa(String.fromCharCode(...signatureArray))
      .replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '');

    return `${signingInput}.${signatureB64}`;
  }

  async requestToken(
    authCode: string,
    clientId: string,
    redirectUri: string,
    codeVerifier: string,
    tokenEndpoint: string
  ): Promise<any> {
    const dpopProof = await this.createDPoPProof('POST', tokenEndpoint);

    const params = new URLSearchParams({
      grant_type: 'authorization_code',
      code: authCode,
      client_id: clientId,
      redirect_uri: redirectUri,
      code_verifier: codeVerifier,
    });

    const response = await fetch(tokenEndpoint, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
        'DPoP': dpopProof,
      },
      body: params.toString(),
    });

    if (!response.ok) {
      throw new Error(`Token request failed: ${response.statusText}`);
    }

    return response.json();
  }

  async makeAuthenticatedRequest(
    method: string,
    url: string,
    accessToken: string,
    body?: any,
    nonce?: string
  ): Promise<Response> {
    const dpopProof = await this.createDPoPProof(method, url, accessToken, nonce);

    const headers: Record<string, string> = {
      'Authorization': `DPoP ${accessToken}`,
      'DPoP': dpopProof,
    };

    if (body) {
      headers['Content-Type'] = 'application/json';
    }

    return fetch(url, {
      method,
      headers,
      body: body ? JSON.stringify(body) : undefined,
    });
  }
}

// Usage example
async function main() {
  const dpopClient = new DPoPClient();
  await dpopClient.init();

  try {
    // 1. Get DPoP-bound access token
    const tokenResponse = await dpopClient.requestToken(
      'auth_code_here',
      '01HPQS4VWXYZ01',
      'https://myapp.example.com/callback',
      'code_verifier_here',
      'https://auth.rustaxum.dev/oauth/token'
    );

    console.log('Token Type:', tokenResponse.token_type); // Should be "DPoP"
    console.log('Access Token:', tokenResponse.access_token);

    // 2. Make authenticated API requests
    const apiResponse = await dpopClient.makeAuthenticatedRequest(
      'GET',
      'https://api.example.com/user/profile',
      tokenResponse.access_token
    );

    if (apiResponse.ok) {
      const userData = await apiResponse.json();
      console.log('User Data:', userData);
    } else {
      console.error('API request failed:', apiResponse.statusText);
    }
  } catch (error) {
    console.error('DPoP flow failed:', error);
  }
}

main();
```

#### Go Implementation for Server-Side Applications

```go
// dpop_client.go
package main

import (
    "crypto/ecdsa"
    "crypto/elliptic"
    "crypto/rand"
    "crypto/sha256"
    "encoding/base64"
    "encoding/json"
    "fmt"
    "net/http"
    "net/url"
    "strings"
    "time"

    "github.com/golang-jwt/jwt/v5"
)

type DPoPClient struct {
    privateKey *ecdsa.PrivateKey
    publicKey  *ecdsa.PublicKey
    jwk        map[string]interface{}
}

type DPoPClaims struct {
    JTI   string `json:"jti"`
    HTM   string `json:"htm"`
    HTU   string `json:"htu"`
    IAT   int64  `json:"iat"`
    ATH   string `json:"ath,omitempty"`
    Nonce string `json:"nonce,omitempty"`
    jwt.RegisteredClaims
}

func NewDPoPClient() (*DPoPClient, error) {
    // Generate ECDSA key pair
    privateKey, err := ecdsa.GenerateKey(elliptic.P256(), rand.Reader)
    if err != nil {
        return nil, fmt.Errorf("failed to generate key pair: %w", err)
    }

    // Create JWK representation
    jwk := map[string]interface{}{
        "kty": "EC",
        "crv": "P-256",
        "x":   base64.RawURLEncoding.EncodeToString(privateKey.PublicKey.X.Bytes()),
        "y":   base64.RawURLEncoding.EncodeToString(privateKey.PublicKey.Y.Bytes()),
    }

    return &DPoPClient{
        privateKey: privateKey,
        publicKey:  &privateKey.PublicKey,
        jwk:        jwk,
    }, nil
}

func (c *DPoPClient) createDPoPProof(httpMethod, httpURL, accessToken, nonce string) (string, error) {
    // Generate random JTI
    jti := make([]byte, 16)
    if _, err := rand.Read(jti); err != nil {
        return "", err
    }

    claims := DPoPClaims{
        JTI: base64.RawURLEncoding.EncodeToString(jti),
        HTM: httpMethod,
        HTU: httpURL,
        IAT: time.Now().Unix(),
    }

    // Add access token hash if provided
    if accessToken != "" {
        hash := sha256.Sum256([]byte(accessToken))
        claims.ATH = base64.RawURLEncoding.EncodeToString(hash[:])
    }

    if nonce != "" {
        claims.Nonce = nonce
    }

    // Create token with custom headers
    token := jwt.NewWithClaims(&jwt.SigningMethodECDSA{}, claims)
    token.Header["typ"] = "dpop+jwt"
    token.Header["jwk"] = c.jwk

    // Sign the token
    tokenString, err := token.SignedString(c.privateKey)
    if err != nil {
        return "", fmt.Errorf("failed to sign DPoP proof: %w", err)
    }

    return tokenString, nil
}

func (c *DPoPClient) RequestToken(authCode, clientID, redirectURI, codeVerifier, tokenEndpoint string) (map[string]interface{}, error) {
    dpopProof, err := c.createDPoPProof("POST", tokenEndpoint, "", "")
    if err != nil {
        return nil, err
    }

    data := url.Values{
        "grant_type":    {"authorization_code"},
        "code":          {authCode},
        "client_id":     {clientID},
        "redirect_uri":  {redirectURI},
        "code_verifier": {codeVerifier},
    }

    req, err := http.NewRequest("POST", tokenEndpoint, strings.NewReader(data.Encode()))
    if err != nil {
        return nil, err
    }

    req.Header.Set("Content-Type", "application/x-www-form-urlencoded")
    req.Header.Set("DPoP", dpopProof)

    client := &http.Client{}
    resp, err := client.Do(req)
    if err != nil {
        return nil, err
    }
    defer resp.Body.Close()

    var result map[string]interface{}
    if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
        return nil, err
    }

    if resp.StatusCode != http.StatusOK {
        return nil, fmt.Errorf("token request failed: %s", result["error"])
    }

    return result, nil
}

func (c *DPoPClient) MakeAuthenticatedRequest(method, url, accessToken string, body []byte, nonce string) (*http.Response, error) {
    dpopProof, err := c.createDPoPProof(method, url, accessToken, nonce)
    if err != nil {
        return nil, err
    }

    var reqBody *strings.Reader
    if body != nil {
        reqBody = strings.NewReader(string(body))
    }

    req, err := http.NewRequest(method, url, reqBody)
    if err != nil {
        return nil, err
    }

    req.Header.Set("Authorization", "DPoP "+accessToken)
    req.Header.Set("DPoP", dpopProof)

    if body != nil {
        req.Header.Set("Content-Type", "application/json")
    }

    client := &http.Client{}
    return client.Do(req)
}

func main() {
    client, err := NewDPoPClient()
    if err != nil {
        fmt.Printf("Failed to create DPoP client: %v\n", err)
        return
    }

    // Example usage
    tokenResponse, err := client.RequestToken(
        "auth_code_here",
        "01HPQS4VWXYZ01",
        "https://myapp.example.com/callback",
        "code_verifier_here",
        "https://auth.rustaxum.dev/oauth/token",
    )

    if err != nil {
        fmt.Printf("Token request failed: %v\n", err)
        return
    }

    fmt.Printf("Token Type: %s\n", tokenResponse["token_type"])
    fmt.Printf("Access Token: %s\n", tokenResponse["access_token"])

    // Make authenticated request
    accessToken := tokenResponse["access_token"].(string)
    resp, err := client.MakeAuthenticatedRequest(
        "GET",
        "https://api.example.com/user/profile",
        accessToken,
        nil,
        "",
    )

    if err != nil {
        fmt.Printf("API request failed: %v\n", err)
        return
    }
    defer resp.Body.Close()

    fmt.Printf("API Response Status: %s\n", resp.Status)
}
```

---

This comprehensive implementation guide provides production-ready examples for integrating with the Rustaxum OAuth 2.1 server across different platforms and use cases, including the latest RFC implementations. Each example includes proper error handling, security best practices, and follows the OAuth 2.1 specification requirements along with RFC 8628 (Device Authorization Grant) and RFC 9449 (DPoP) standards.
