# Advanced MFA Methods Implementation Guide

This document describes the implementation of Email OTP, WebAuthn (Physical Keys), and Biometric authentication methods for the RustAxum MFA system.

## Overview

The MFA system now supports five authentication methods:

1. **TOTP** (Time-based One-Time Password) - Existing
2. **Backup Codes** - Existing
3. **Email OTP** (One-Time Password) - NEW
4. **WebAuthn** (Physical Security Keys - FIDO2) - NEW
5. **Biometric** (Fingerprint, Face ID, Touch ID, etc.) - NEW

## Architecture

### Database Schema

#### New Tables

1. **`mfa_email_codes`** - Temporary email OTP storage
   - `id` - ULID primary key
   - `user_id` - Reference to sys_users
   - `code` - Plain text code (for sending via email)
   - `code_hash` - SHA256 hash of code
   - `expires_at` - Code expiration timestamp
   - `verified_at` - Verification timestamp
   - `is_used` - Boolean flag
   - `ip_address` - Request IP (optional)
   - `user_agent` - Request user agent (optional)

2. **`mfa_webauthn_credentials`** - FIDO2/WebAuthn credentials
   - `id` - ULID primary key
   - `user_id` - Reference to sys_users
   - `credential_id` - Base64 encoded credential ID (unique)
   - `public_key` - COSE public key (JSON)
   - `counter` - Signature counter (replay protection)
   - `device_name` - User-friendly name (e.g., "YubiKey 5")
   - `aaguid` - Authenticator Attestation GUID
   - `transports` - Array of transports (usb, nfc, ble, internal)
   - `attestation_format` - Attestation format
   - `is_backup_eligible` - Backup eligibility flag
   - `is_backup_state` - Backup state flag
   - `last_used_at` - Last authentication timestamp

3. **`mfa_biometric_credentials`** - Biometric authentication
   - `id` - ULID primary key
   - `user_id` - Reference to sys_users
   - `device_id` - Optional link to devices table
   - `biometric_type` - Type: fingerprint, face, iris, voice
   - `credential_id` - Platform credential identifier
   - `public_key` - Public key for verification
   - `platform` - Platform: ios, android, windows, macos, linux
   - `device_name` - User-friendly device name
   - `is_platform_authenticator` - Platform vs external
   - `counter` - Signature counter
   - `last_used_at` - Last authentication timestamp

4. **`mfa_webauthn_challenges`** - Temporary challenge storage
   - `id` - ULID primary key
   - `user_id` - Reference to sys_users
   - `challenge` - Base64 encoded challenge
   - `challenge_type` - Type: registration, authentication, biometric_registration, biometric_authentication
   - `expires_at` - Challenge expiration
   - `is_used` - Boolean flag

#### Updated Tables

1. **`mfa_methods`** - Added support for new method types
   - Updated constraint to allow: `totp`, `backup_codes`, `email`, `webauthn`, `biometric`
   - Added `metadata` JSONB column for method-specific data

### Models

#### New Model Files

1. **`mfa_email_code.rs`** ([src/app/models/mfa_email_code.rs](../src/app/models/mfa_email_code.rs))
   - `MfaEmailCode` - Main model
   - `NewMfaEmailCode` - Insertable model
   - `SendEmailCodeRequest` - Request DTO
   - `VerifyEmailCodeRequest` - Request DTO

2. **`mfa_webauthn.rs`** ([src/app/models/mfa_webauthn.rs](../src/app/models/mfa_webauthn.rs))
   - `MfaWebAuthnCredential` - Credential model
   - `MfaWebAuthnChallenge` - Challenge model
   - `WebAuthnRegistrationStartRequest` - Request DTO
   - `WebAuthnRegistrationFinishRequest` - Request DTO
   - `WebAuthnAuthenticationStartRequest` - Request DTO
   - `WebAuthnAuthenticationFinishRequest` - Request DTO
   - `WebAuthnCredentialResponse` - Response DTO

3. **`mfa_biometric.rs`** ([src/app/models/mfa_biometric.rs](../src/app/models/mfa_biometric.rs))
   - `MfaBiometricCredential` - Credential model
   - `BiometricRegistrationRequest` - Request DTO
   - `BiometricAuthenticationRequest` - Request DTO
   - `BiometricCredentialResponse` - Response DTO

### Services

#### 1. Email OTP Service ([src/app/services/mfa_email_service.rs](../src/app/services/mfa_email_service.rs))

**Features:**
- Generate 6-digit numeric codes
- SHA256 hashing for secure storage
- 10-minute expiration time
- Rate limiting: 5 codes per hour
- HTML email templates
- Automatic code invalidation

**Key Methods:**
```rust
pub async fn send_code(pool: &DbPool, user_id: String, ...) -> Result<()>
pub async fn verify_code(pool: &DbPool, user_id: String, code: &str) -> Result<bool>
pub fn is_enabled(pool: &DbPool, user_id: String) -> Result<bool>
pub async fn cleanup_expired_codes(pool: &DbPool) -> Result<usize>
```

**Security:**
- Codes hashed before storage (SHA256)
- One-time use enforcement
- Expiration validation
- Rate limiting per user
- IP and user agent logging

#### 2. WebAuthn Service ([src/app/services/mfa_webauthn_service.rs](../src/app/services/mfa_webauthn_service.rs))

**Features:**
- FIDO2/WebAuthn protocol support
- Hardware security key support
- Platform authenticators (Windows Hello, etc.)
- Signature counter for replay protection
- Multiple credentials per user
- Device name management

**Key Methods:**
```rust
pub async fn start_registration(pool: &DbPool, user_id: String, ...) -> Result<CreationChallengeResponse>
pub async fn finish_registration(pool: &DbPool, user_id: String, credential, ...) -> Result<()>
pub async fn start_authentication(pool: &DbPool, user_id: String) -> Result<RequestChallengeResponse>
pub async fn finish_authentication(pool: &DbPool, user_id: String, credential) -> Result<bool>
pub async fn list_credentials(pool: &DbPool, user_id: String) -> Result<Vec<MfaWebAuthnCredential>>
pub async fn delete_credential(pool: &DbPool, credential_id: &str) -> Result<()>
```

**Security:**
- Challenge-response protocol
- Cryptographic credential verification
- Counter-based replay protection
- Per-credential tracking
- Credential exclusion lists

**Supported Transports:**
- USB (YubiKey, etc.)
- NFC (contactless)
- Bluetooth Low Energy (BLE)
- Internal/Platform

#### 3. Biometric Service ([src/app/services/mfa_biometric_service.rs](../src/app/services/mfa_biometric_service.rs))

**Features:**
- Platform-specific biometric authentication
- Support for multiple biometric types
- Device binding
- Cross-platform support

**Supported Biometric Types:**
- **Fingerprint** - Touch ID, fingerprint sensors
- **Face** - Face ID, facial recognition
- **Iris** - Iris scanning
- **Voice** - Voice recognition

**Supported Platforms:**
- iOS (Touch ID, Face ID)
- Android (Fingerprint, Face Unlock)
- Windows (Windows Hello)
- macOS (Touch ID)
- Linux (fprintd, etc.)

**Key Methods:**
```rust
pub async fn start_registration(pool, user_id, biometric_type, platform, ...) -> Result<CreationChallengeResponse>
pub async fn finish_registration(pool, user_id, credential, biometric_type, platform, ...) -> Result<()>
pub async fn start_authentication(pool: &DbPool, user_id: String) -> Result<RequestChallengeResponse>
pub async fn finish_authentication(pool, user_id, credential) -> Result<bool>
pub async fn list_credentials(pool: &DbPool, user_id: String) -> Result<Vec<MfaBiometricCredential>>
pub async fn list_by_type(pool, user_id, bio_type) -> Result<Vec<MfaBiometricCredential>>
pub async fn delete_credential(pool: &DbPool, credential_id: &str) -> Result<()>
```

**Security:**
- WebAuthn protocol (same as physical keys)
- Platform authenticator validation
- Device-bound credentials
- Counter-based replay protection

### Controllers

#### MFA Controller Extensions ([src/app/http/controllers/mfa_controller_extensions.rs](../src/app/http/controllers/mfa_controller_extensions.rs))

**Email OTP Endpoints:**

```
POST /mfa/email/send
  - Requires: Authentication
  - Sends email code to user's email
  - Response: { "message": "Email code sent successfully" }

POST /mfa/email/verify
  - Requires: Authentication
  - Body: { "code": "123456" }
  - Response: { "verified": true, "message": "..." }
```

**WebAuthn Endpoints:**

```
POST /mfa/webauthn/register/start
  - Requires: Authentication
  - Body: { "device_name": "YubiKey 5" }
  - Response: CreationChallengeResponse (WebAuthn spec)

POST /mfa/webauthn/register/finish
  - Requires: Authentication
  - Body: RegisterPublicKeyCredential (WebAuthn spec)
  - Response: { "message": "Credential registered" }

POST /mfa/webauthn/auth/start
  - Requires: Authentication
  - Response: RequestChallengeResponse (WebAuthn spec)

POST /mfa/webauthn/auth/finish
  - Requires: Authentication
  - Body: PublicKeyCredential (WebAuthn spec)
  - Response: { "verified": true, "message": "..." }

GET /mfa/webauthn/credentials
  - Requires: Authentication
  - Response: { "credentials": [...] }

DELETE /mfa/webauthn/credentials/:id
  - Requires: Authentication
  - Response: { "message": "Credential deleted" }
```

**Biometric Endpoints:**

```
POST /mfa/biometric/register/start
  - Requires: Authentication
  - Body: { "biometric_type": "fingerprint", "platform": "ios", "device_name": "iPhone 13" }
  - Response: CreationChallengeResponse

POST /mfa/biometric/register/finish
  - Requires: Authentication
  - Body: BiometricRegistrationRequest
  - Response: { "message": "Credential registered" }

POST /mfa/biometric/auth/start
  - Requires: Authentication
  - Response: RequestChallengeResponse

POST /mfa/biometric/auth/finish
  - Requires: Authentication
  - Body: BiometricAuthenticationRequest
  - Response: { "verified": true, "message": "..." }

GET /mfa/biometric/credentials
  - Requires: Authentication
  - Response: { "credentials": [...] }

DELETE /mfa/biometric/credentials/:id
  - Requires: Authentication
  - Response: { "message": "Credential deleted" }
```

## Setup Instructions

### 1. Run Migration

```bash
cargo run --bin artisan -- migrate
```

This will create:
- `mfa_email_codes` table
- `mfa_webauthn_credentials` table
- `mfa_biometric_credentials` table
- `mfa_webauthn_challenges` table
- Update `mfa_methods` table

### 2. Update Environment Variables

Add to `.env`:

```env
# MFA Configuration
MFA_EMAIL_CODE_EXPIRY_MINUTES=10
MFA_EMAIL_CODE_LENGTH=6
MFA_EMAIL_MAX_CODES_PER_HOUR=5

# WebAuthn Configuration
WEBAUTHN_RP_NAME="RustAxum"
WEBAUTHN_RP_ID="localhost"  # Your domain
WEBAUTHN_RP_ORIGIN="http://localhost:3000"  # Your app URL

# For production, use your actual domain:
# WEBAUTHN_RP_ID="example.com"
# WEBAUTHN_RP_ORIGIN="https://example.com"
```

### 3. Update Routes

Add routes in `src/routes/web.rs` or `src/routes/api.rs`:

```rust
use crate::app::http::controllers::mfa_controller_extensions;

// Email OTP routes
.route("/mfa/email/send", post(mfa_controller_extensions::send_email_code))
.route("/mfa/email/verify", post(mfa_controller_extensions::verify_email_code))

// WebAuthn routes
.route("/mfa/webauthn/register/start", post(mfa_controller_extensions::webauthn_register_start))
.route("/mfa/webauthn/register/finish", post(mfa_controller_extensions::webauthn_register_finish))
.route("/mfa/webauthn/auth/start", post(mfa_controller_extensions::webauthn_auth_start))
.route("/mfa/webauthn/auth/finish", post(mfa_controller_extensions::webauthn_auth_finish))
.route("/mfa/webauthn/credentials", get(mfa_controller_extensions::webauthn_list_credentials))
.route("/mfa/webauthn/credentials/:id", delete(mfa_controller_extensions::webauthn_delete_credential))

// Biometric routes
.route("/mfa/biometric/register/start", post(mfa_controller_extensions::biometric_register_start))
.route("/mfa/biometric/auth/start", post(mfa_controller_extensions::biometric_auth_start))
.route("/mfa/biometric/credentials", get(mfa_controller_extensions::biometric_list_credentials))
.route("/mfa/biometric/credentials/:id", delete(mfa_controller_extensions::biometric_delete_credential))
```

## Usage Examples

### Email OTP Flow

**Setup:**
1. User enables Email MFA in settings
2. System creates entry in `mfa_methods` with `method_type = 'email'`

**Authentication:**
1. User logs in with email/password
2. System detects Email MFA enabled
3. Client calls `POST /mfa/email/send`
4. User receives email with 6-digit code
5. User enters code in UI
6. Client calls `POST /mfa/email/verify` with code
7. On success, user is authenticated

### WebAuthn Flow

**Registration:**
1. User clicks "Add Security Key"
2. Client calls `POST /mfa/webauthn/register/start`
3. Server returns WebAuthn challenge
4. Browser shows "Insert your security key" prompt
5. User inserts YubiKey/touches sensor
6. Client calls `POST /mfa/webauthn/register/finish` with credential
7. Server stores credential

**Authentication:**
1. User logs in with email/password
2. System detects WebAuthn MFA enabled
3. Client calls `POST /mfa/webauthn/auth/start`
4. Server returns challenge
5. Browser prompts for security key
6. User authenticates with key
7. Client calls `POST /mfa/webauthn/auth/finish`
8. On success, user is authenticated

### Biometric Flow

**Registration (iOS Example):**
1. User clicks "Enable Face ID"
2. Client calls `POST /mfa/biometric/register/start` with:
   ```json
   {
     "biometric_type": "face",
     "platform": "ios",
     "device_name": "iPhone 13 Pro"
   }
   ```
3. Server returns WebAuthn challenge
4. iOS shows Face ID prompt
5. User scans face
6. Client calls `POST /mfa/biometric/register/finish`
7. Server stores credential

**Authentication:**
1. User logs in
2. System detects Biometric MFA enabled
3. Client calls `POST /mfa/biometric/auth/start`
4. Server returns challenge
5. Device prompts for biometric
6. User authenticates (Face ID/Touch ID/Fingerprint)
7. Client calls `POST /mfa/biometric/auth/finish`
8. On success, user is authenticated

## Security Considerations

### Email OTP
- **Rate Limiting**: Maximum 5 codes per hour per user
- **Expiration**: Codes expire after 10 minutes
- **Hashing**: Codes are SHA256 hashed before storage
- **One-Time Use**: Codes are invalidated after verification
- **Email Security**: Ensure email transport is encrypted (TLS)

### WebAuthn/FIDO2
- **Replay Protection**: Signature counter prevents credential cloning
- **Challenge-Response**: Cryptographic challenge prevents replay attacks
- **Phishing Resistant**: Origin validation prevents phishing
- **Hardware Binding**: Credentials bound to physical devices
- **Attestation**: Optional attestation for enterprise deployments

### Biometric
- **Platform Security**: Relies on platform authenticator security
- **No Biometric Storage**: Biometric data never leaves device
- **Cryptographic Proof**: Uses public key cryptography
- **Device Binding**: Credentials bound to specific devices
- **Fallback Methods**: Always provide backup authentication method

## Frontend Integration

### Email OTP

```javascript
// Send code
async function sendEmailCode() {
    const response = await fetch('/mfa/email/send', {
        method: 'POST',
        headers: {
            'Authorization': `Bearer ${token}`,
            'Content-Type': 'application/json'
        }
    });
    const data = await response.json();
    console.log(data.message);
}

// Verify code
async function verifyEmailCode(code) {
    const response = await fetch('/mfa/email/verify', {
        method: 'POST',
        headers: {
            'Authorization': `Bearer ${token}`,
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({ code })
    });
    const data = await response.json();
    return data.verified;
}
```

### WebAuthn

```javascript
// Register security key
async function registerSecurityKey() {
    // Start registration
    const startResp = await fetch('/mfa/webauthn/register/start', {
        method: 'POST',
        headers: {
            'Authorization': `Bearer ${token}`,
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({ device_name: 'My YubiKey' })
    });
    const challenge = await startResp.json();

    // Create credential using WebAuthn API
    const credential = await navigator.credentials.create({
        publicKey: challenge.publicKey
    });

    // Finish registration
    const finishResp = await fetch('/mfa/webauthn/register/finish', {
        method: 'POST',
        headers: {
            'Authorization': `Bearer ${token}`,
            'Content-Type': 'application/json'
        },
        body: JSON.stringify(credential)
    });

    return await finishResp.json();
}

// Authenticate with security key
async function authenticateWithSecurityKey() {
    // Start authentication
    const startResp = await fetch('/mfa/webauthn/auth/start', {
        method: 'POST',
        headers: {
            'Authorization': `Bearer ${token}`
        }
    });
    const challenge = await startResp.json();

    // Get credential
    const credential = await navigator.credentials.get({
        publicKey: challenge.publicKey
    });

    // Finish authentication
    const finishResp = await fetch('/mfa/webauthn/auth/finish', {
        method: 'POST',
        headers: {
            'Authorization': `Bearer ${token}`,
            'Content-Type': 'application/json'
        },
        body: JSON.stringify(credential)
    });

    return await finishResp.json();
}
```

### Biometric (iOS/Android)

For biometric authentication, use the same WebAuthn API but with platform authenticator:

```javascript
// The browser/platform will automatically use biometric if available
const credential = await navigator.credentials.create({
    publicKey: {
        ...challenge.publicKey,
        authenticatorSelection: {
            authenticatorAttachment: 'platform', // Use platform authenticator
            userVerification: 'required' // Require biometric
        }
    }
});
```

## Testing

### Email OTP Testing

```bash
# Enable email MFA for test user
curl -X POST http://localhost:3000/mfa/email/send \
  -H "Authorization: Bearer $TOKEN"

# Verify code
curl -X POST http://localhost:3000/mfa/email/verify \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"code": "123456"}'
```

### WebAuthn Testing

Use a physical security key or virtual authenticator in Chrome DevTools:
1. Open Chrome DevTools
2. Go to "More tools" → "WebAuthn"
3. Click "Enable virtual authenticator environment"
4. Add a virtual authenticator
5. Test registration and authentication flows

### Biometric Testing

- **iOS**: Use iOS Simulator with simulated Face ID/Touch ID
- **Android**: Use Android Emulator with fingerprint sensor
- **Desktop**: Use Chrome/Edge with Windows Hello or macOS Touch ID

## Maintenance

### Cleanup Tasks

Add cron job or scheduled task:

```rust
// Cleanup expired email codes
MfaEmailService::cleanup_expired_codes(&pool).await?;

// Cleanup expired challenges
use crate::schema::mfa_webauthn_challenges::dsl::*;
diesel::delete(mfa_webauthn_challenges)
    .filter(expires_at.lt(chrono::Utc::now() - chrono::Duration::hours(1)))
    .execute(&mut conn)?;
```

### Monitoring

Monitor the following metrics:
- Email OTP send/verify success rates
- WebAuthn registration/authentication success rates
- Biometric authentication success rates
- Failed MFA attempts per user
- Expired code cleanup frequency

## Troubleshooting

### Email OTP Issues

**Problem**: Codes not being received
- Check email service configuration
- Verify email queue is processing
- Check spam folder
- Verify email template rendering

**Problem**: "Too many codes requested"
- Rate limit is 5 per hour
- Wait for rate limit to reset
- Check for potential abuse

### WebAuthn Issues

**Problem**: "Not allowed" error
- Ensure HTTPS (required for WebAuthn, except localhost)
- Check Relying Party ID matches domain
- Verify origin matches configuration

**Problem**: Credential not working
- Check signature counter hasn't decreased (cloning attack)
- Verify credential hasn't been deleted
- Ensure user is using same device

### Biometric Issues

**Problem**: Biometric prompt not showing
- Verify platform supports biometric
- Check device has biometric enrolled
- Ensure permissions granted

**Problem**: "User verification failed"
- User may have cancelled prompt
- Biometric may have changed
- Device may be locked

## Migration from Legacy MFA

If you have existing TOTP MFA users, they will continue to work. The new methods are additive:

```sql
-- Check users with TOTP enabled
SELECT id, email, mfa_enabled FROM sys_users WHERE mfa_enabled = true;

-- Users can have multiple MFA methods
SELECT user_id, method_type, is_enabled FROM mfa_methods;
```

## Future Enhancements

Potential improvements:
1. **SMS OTP** - Add SMS-based OTP
2. **Push Notifications** - Mobile push authentication
3. **Trusted Devices** - Remember device functionality
4. **Risk-Based Authentication** - Adaptive MFA based on risk
5. **Recovery Workflows** - Enhanced account recovery
6. **Admin Controls** - Force MFA for specific roles
7. **Passkeys** - Full passkey support (passwordless)

## References

- [WebAuthn Specification](https://www.w3.org/TR/webauthn-2/)
- [FIDO2 Project](https://fidoalliance.org/fido2/)
- [webauthn-rs Documentation](https://docs.rs/webauthn-rs/)
- [MDN WebAuthn API](https://developer.mozilla.org/en-US/docs/Web/API/Web_Authentication_API)

---

**Author**: Claude Code
**Date**: 2025-10-10
**Version**: 1.0.0
