# MFA System Architecture Diagram

## 🏗️ Complete System Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          RUSTAXUM MFA SYSTEM                                 │
│                       (Multi-Factor Authentication)                          │
└─────────────────────────────────────────────────────────────────────────────┘

┌───────────────────────────────────────────────────────────────────────────────┐
│                              CLIENT LAYER                                      │
├───────────────────────────────────────────────────────────────────────────────┤
│  Web Browser  │  Mobile App  │  Desktop App  │  API Clients  │  IoT Devices  │
│  ────────────   ────────────   ─────────────   ─────────────   ──────────────│
│  JavaScript   │  iOS/Android │  Electron     │  REST API     │  Embedded     │
│  WebAuthn API │  Biometric   │  Native Auth  │  OAuth 2.1    │  Limited UI   │
└───────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ↓
┌───────────────────────────────────────────────────────────────────────────────┐
│                              API GATEWAY                                       │
├───────────────────────────────────────────────────────────────────────────────┤
│                         Axum HTTP Server (Rust)                               │
│  ┌─────────────┐  ┌──────────────┐  ┌─────────────┐  ┌──────────────────┐  │
│  │ Rate Limiter│  │ CORS Handler │  │ Auth Guard  │  │ Correlation ID   │  │
│  └─────────────┘  └──────────────┘  └─────────────┘  └──────────────────┘  │
└───────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ↓
┌───────────────────────────────────────────────────────────────────────────────┐
│                           CONTROLLER LAYER                                     │
├───────────────────────────────────────────────────────────────────────────────┤
│  ┌────────────────────┐  ┌─────────────────────┐  ┌───────────────────────┐ │
│  │ AuthController     │  │ MfaController       │  │ MfaExtensionsCtrl     │ │
│  │ ─────────────      │  │ ────────────        │  │ ────────────────      │ │
│  │ • Login           │  │ • Setup TOTP        │  │ • Email OTP           │ │
│  │ • MFA Login       │  │ • Verify TOTP       │  │ • SMS OTP             │ │
│  │ • Logout          │  │ • Backup Codes      │  │ • WebAuthn            │ │
│  └────────────────────┘  │ • Disable MFA       │  │ • Biometric           │ │
│                          └─────────────────────┘  │ • Push                │ │
│                                                    │ • Backup Email        │ │
│                                                    └───────────────────────┘ │
└───────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ↓
┌───────────────────────────────────────────────────────────────────────────────┐
│                            SERVICE LAYER                                       │
├───────────────────────────────────────────────────────────────────────────────┤
│                                                                                │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │              MFA MANAGER SERVICE (Coordinator)                        │   │
│  │  ──────────────────────────────────────────────────────────────────  │   │
│  │  • Unified MFA Interface                                              │   │
│  │  • Method Selection & Recommendation                                  │   │
│  │  • User Preferences Management                                        │   │
│  │  • Trusted Device Management                                          │   │
│  │  • Audit Logging                                                      │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                                      │                                         │
│                   ┌──────────────────┼──────────────────┐                    │
│                   │                  │                  │                     │
│  ┌────────────────▼───┐  ┌──────────▼──────┐  ┌───────▼──────────┐         │
│  │  MfaService        │  │ EmailService    │  │ SmsService       │         │
│  │  (TOTP/Backup)     │  │ (Email OTP)     │  │ (SMS OTP)        │         │
│  │  ────────────      │  │ ────────────    │  │ ────────────     │         │
│  │  • TOTP Gen        │  │ • Code Gen      │  │ • Code Gen       │         │
│  │  • QR Code         │  │ • Email Send    │  │ • SMS Send       │         │
│  │  • Backup Codes    │  │ • Rate Limit    │  │ • Phone Valid    │         │
│  └────────────────────┘  └─────────────────┘  └──────────────────┘         │
│                                                                                │
│  ┌────────────────────┐  ┌─────────────────┐  ┌──────────────────┐         │
│  │ WebAuthnService    │  │ BiometricService│  │ PushService      │         │
│  │ (Physical Keys)    │  │ (Face/Touch ID) │  │ (Mobile Approve) │         │
│  │ ────────────       │  │ ────────────    │  │ ────────────     │         │
│  │  • FIDO2 Protocol  │  │ • Platform Auth │  │ • FCM/APNS       │         │
│  │  • Credential Mgmt │  │ • Biometric Type│  │ • Challenge Send │         │
│  │  • Challenge/Resp  │  │ • Device Binding│  │ • Approval Track │         │
│  └────────────────────┘  └─────────────────┘  └──────────────────┘         │
│                                                                                │
│  ┌────────────────────┐  ┌─────────────────┐  ┌──────────────────┐         │
│  │ BackupEmailService │  │ UserService     │  │ ActivityLogService│        │
│  │ (Recovery Email)   │  │ (User Mgmt)     │  │ (Audit Trail)    │         │
│  └────────────────────┘  └─────────────────┘  └──────────────────┘         │
└───────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ↓
┌───────────────────────────────────────────────────────────────────────────────┐
│                             DATA LAYER (Diesel ORM)                            │
├───────────────────────────────────────────────────────────────────────────────┤
│                                                                                │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                        POSTGRESQL DATABASE                              │ │
│  │  ────────────────────────────────────────────────────────────────────  │ │
│  │                                                                          │ │
│  │  Core Tables:                      Advanced Tables:                     │ │
│  │  ─────────────                     ────────────────                     │ │
│  │  • sys_users                       • mfa_push_devices                   │ │
│  │  • mfa_methods                     • mfa_push_challenges                │ │
│  │  • mfa_attempts                    • mfa_backup_emails                  │ │
│  │                                    • mfa_backup_email_codes             │ │
│  │  Authentication Methods:           • mfa_recovery_methods               │ │
│  │  ─────────────────────            • mfa_trusted_devices                │ │
│  │  • mfa_email_codes                 • mfa_audit_log                      │ │
│  │  • mfa_sms_codes                                                        │ │
│  │  • mfa_webauthn_credentials                                             │ │
│  │  • mfa_webauthn_challenges                                              │ │
│  │  • mfa_biometric_credentials                                            │ │
│  │                                                                          │ │
│  │  Indexes: 30+ performance indexes for fast lookups                      │ │
│  │  Constraints: Foreign keys, check constraints, unique constraints       │ │
│  │  Functions: cleanup_expired_mfa_data_extended()                         │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
└───────────────────────────────────────────────────────────────────────────────┘
                                      │
                         ┌────────────┴────────────┐
                         │                         │
                         ↓                         ↓
┌──────────────────────────────┐    ┌──────────────────────────────┐
│   EXTERNAL INTEGRATIONS      │    │    MONITORING & ALERTS       │
├──────────────────────────────┤    ├──────────────────────────────┤
│  • Email Provider (SMTP)     │    │  • Prometheus Metrics        │
│  • SMS Provider (Twilio)     │    │  • Grafana Dashboards        │
│  • Push Notifications        │    │  • Error Tracking (Sentry)   │
│    - FCM (Android)           │    │  • Log Aggregation (ELK)     │
│    - APNS (iOS)              │    │  • Security Alerts           │
│  • WebAuthn Providers        │    │  • Audit Log Analysis        │
└──────────────────────────────┘    └──────────────────────────────┘
```

---

## 🔄 MFA Flow Diagrams

### 1. Login Flow with MFA

```
┌──────┐                                                           ┌──────────┐
│      │  1. POST /auth/login (email, password)                   │          │
│      ├──────────────────────────────────────────────────────────>          │
│      │                                                           │          │
│      │  2. Check credentials & MFA status                       │  Axum    │
│ User │     if MFA enabled → return MFA_REQUIRED                 │  Server  │
│      │                                                           │          │
│      │  3. Response: { requires_mfa: true, methods: [...] }     │          │
│      │<──────────────────────────────────────────────────────────          │
│      │                                                           │          │
│      │  4. User selects method (e.g., "email")                  │          │
│      ├──────────────────────────────────────────────────────────>          │
│      │     POST /mfa/challenge { method: "email" }              │          │
│      │                                                           │          │
│      │  5. Send code via email                                  │          │
│      │<──────────────────────────────────────────────────────────          │
│      │     Response: { message: "Code sent" }                   │          │
│      │                                                           │          │
│      │  6. User enters code                                     │          │
│      ├──────────────────────────────────────────────────────────>          │
│      │     POST /mfa/verify { code: "123456" }                  │          │
│      │                                                           │          │
│      │  7. Verify code + generate JWT                           │          │
│      │                                                           │          │
│      │  8. Response: { access_token, refresh_token }            │          │
│      │<──────────────────────────────────────────────────────────          │
└──────┘                                                           └──────────┘
```

### 2. Trusted Device Flow

```
┌──────┐                                                           ┌──────────┐
│      │  1. MFA verification with trust_device=true               │          │
│      ├──────────────────────────────────────────────────────────>          │
│ User │     POST /mfa/verify {                                    │  Server  │
│      │       code: "123456",                                     │          │
│      │       trust_device: true,                                 │          │
│      │       device_fingerprint: "abc123"                        │          │
│      │     }                                                      │          │
│      │                                                           │          │
│      │  2. Verify code → Generate trust token                   │          │
│      │                                                           │          │
│      │  3. Response: { verified: true, trust_token: "..." }     │          │
│      │<──────────────────────────────────────────────────────────          │
│      │                                                           │          │
│      │  4. Store trust_token in cookie/localStorage             │          │
│      │                                                           │          │
│      │                                                           │          │
│      │  ═══════════ Next Login (within 30 days) ════════════    │          │
│      │                                                           │          │
│      │  5. Login with trust_token                                │          │
│      ├──────────────────────────────────────────────────────────>          │
│      │     Headers: X-Trust-Token: "..."                         │          │
│      │                                                           │          │
│      │  6. Validate trust token → Skip MFA                       │          │
│      │                                                           │          │
│      │  7. Direct access granted                                 │          │
│      │<──────────────────────────────────────────────────────────          │
└──────┘                                                           └──────────┘
```

---

## 🔐 Security Layers

```
┌────────────────────────────────────────────────────────────────┐
│                      Security Stack                             │
├────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Layer 1: Transport Security                                   │
│  ────────────────────────                                      │
│  ✓ HTTPS/TLS 1.3                                              │
│  ✓ Certificate Pinning (Mobile)                               │
│  ✓ HSTS Headers                                               │
│                                                                 │
│  Layer 2: Authentication                                        │
│  ────────────────────────                                      │
│  ✓ Argon2 Password Hashing                                    │
│  ✓ JWT Token-Based Auth                                       │
│  ✓ Token Rotation                                             │
│  ✓ Refresh Token Validation                                   │
│                                                                 │
│  Layer 3: Multi-Factor Authentication                           │
│  ────────────────────────────────────                         │
│  ✓ 10 MFA Methods                                             │
│  ✓ Method Diversity (Something you know/have/are)            │
│  ✓ Adaptive MFA (Risk-based)                                  │
│  ✓ Fallback Methods                                           │
│                                                                 │
│  Layer 4: Rate Limiting                                         │
│  ───────────────────                                           │
│  ✓ Per-User Rate Limits                                       │
│  ✓ Per-IP Rate Limits                                         │
│  ✓ Progressive Delays                                         │
│  ✓ Account Lockouts                                           │
│                                                                 │
│  Layer 5: Audit & Monitoring                                    │
│  ────────────────────────                                      │
│  ✓ Comprehensive Logging                                      │
│  ✓ Anomaly Detection                                          │
│  ✓ Real-time Alerts                                           │
│  ✓ Compliance Reporting                                       │
│                                                                 │
│  Layer 6: Data Protection                                       │
│  ─────────────────────                                         │
│  ✓ Encryption at Rest (Database)                              │
│  ✓ Encryption in Transit (TLS)                                │
│  ✓ Code Hashing (SHA256)                                      │
│  ✓ Key Rotation                                               │
│                                                                 │
└────────────────────────────────────────────────────────────────┘
```

---

## 📊 Data Flow

```
Request Flow:
─────────────

Client → Rate Limiter → CORS → Auth Guard → Controller → Service → Database
   │           │          │         │            │          │         │
   │           ↓          ↓         ↓            ↓          ↓         ↓
   │      Check limit  Validate  Verify JWT  Business  Query/    PostgreSQL
   │                   origin    token       logic     Update
   │
   └─── Response ←─────────────────────────────────────────────────────┘


MFA Verification Flow:
──────────────────────

1. User Request
   └─> Controller receives MFA challenge request

2. Service Layer
   ├─> MfaManagerService.send_challenge()
   ├─> Determines method (email, sms, totp, etc.)
   ├─> Delegates to specific service
   │   ├─> MfaEmailService → Generate code → Send email
   │   ├─> MfaSmsService → Generate code → Send SMS
   │   ├─> MfaWebAuthnService → Create challenge → Return to client
   │   └─> MfaBiometricService → Create challenge → Return to client
   │
   └─> Store challenge/code in database

3. User Responds
   └─> Controller receives verification request

4. Verification
   ├─> MfaManagerService.verify_challenge()
   ├─> Delegates to specific service
   ├─> Validates code/response
   ├─> Checks expiration
   ├─> Checks rate limits
   └─> Returns success/failure

5. Post-Verification
   ├─> Log attempt (mfa_audit_log)
   ├─> Update last_used_at
   ├─> Optional: Create trusted device
   └─> Return JWT tokens if successful
```

---

## 🎯 Method Selection Logic

```
┌─────────────────────────────────────────────────────────┐
│        MFA Method Selection Algorithm                    │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  1. Check User Preferences                               │
│     ├─> Primary Method Set? → Use Primary              │
│     └─> No Primary → Go to Step 2                      │
│                                                          │
│  2. Check Trusted Device                                 │
│     ├─> Device Trusted & Valid? → Skip MFA             │
│     └─> Not Trusted → Go to Step 3                     │
│                                                          │
│  3. Risk Assessment (Optional)                           │
│     ├─> Low Risk (known IP, device) → Email/SMS        │
│     ├─> Medium Risk → TOTP/Email                       │
│     └─> High Risk → WebAuthn/Biometric                 │
│                                                          │
│  4. Available Methods                                    │
│     Priority Order:                                      │
│     1. WebAuthn (Highest Security)                      │
│     2. Biometric (High Security, Convenient)            │
│     3. Push (High Security, User-Friendly)              │
│     4. TOTP (High Security, Offline)                    │
│     5. Email OTP (Medium Security, Accessible)          │
│     6. SMS OTP (Medium Security, Mobile)                │
│     7. Backup Codes (Recovery Only)                     │
│                                                          │
│  5. Fallback                                             │
│     └─> If primary fails → Offer backup method          │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

---

## 🔄 State Management

```
MFA Session States:
──────────────────

┌─────────────┐
│  Not Setup  │ → User has no MFA methods enabled
└──────┬──────┘
       │ User enables MFA
       ↓
┌─────────────┐
│ Setup Pending│ → QR code shown, awaiting verification
└──────┬──────┘
       │ User verifies
       ↓
┌─────────────┐
│   Enabled   │ → MFA active, login requires verification
└──────┬──────┘
       │
       ├─────────────┐
       │             │
       ↓             ↓
┌─────────────┐  ┌──────────────┐
│ Challenge   │  │ Trusted Device│
│   Sent      │  │   Recognized  │
└──────┬──────┘  └───────┬───────┘
       │                 │
       │ Verify          │ Skip MFA
       ↓                 ↓
┌─────────────┐  ┌──────────────┐
│ Authenticated│  │ Authenticated│
└─────────────┘  └──────────────┘
```

---

**This diagram provides a complete visual representation of:**
- ✅ System architecture (all layers)
- ✅ Service interactions
- ✅ Data flow
- ✅ Security layers
- ✅ MFA flows
- ✅ Method selection logic
- ✅ State management

**Reference this diagram for:**
- System understanding
- Onboarding new developers
- Architecture reviews
- Security audits
- Documentation
