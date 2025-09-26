# Complete Web Authentication System

This document demonstrates the complete web-based authentication system with HTML forms, session management, and all authentication features.

## 🌐 Web Authentication Features

The web authentication system provides a complete Laravel-like experience with:

### **Authentication Pages**

- **Login Page**: `/auth/login` - Sign in with email/password
- **Register Page**: `/auth/register` - Create new account with validation
- **Forgot Password**: `/auth/forgot-password` - Request password reset link
- **Reset Password**: `/auth/reset-password/{token}` - Set new password with token
- **Change Password**: `/auth/change-password` - Change password when logged in
- **Dashboard**: `/dashboard` - Protected user dashboard

### **Session Management**

- **Session-based Authentication**: Uses server-side sessions with cookies
- **Flash Messages**: Laravel-style success/error messages
- **Form Validation**: Server-side validation with error display
- **CSRF Protection**: Built-in CSRF token validation
- **Remember Me**: Optional persistent login sessions

## 🚀 Getting Started

### 1. Start the Server

```bash
# Using Docker Compose (recommended)
docker-compose up -d

# Or directly with Cargo
cargo run
```

Server will be available at: `http://localhost:3000`

### 2. Access Authentication Pages

- **Home**: http://localhost:3000/
- **Login**: http://localhost:3000/auth/login
- **Register**: http://localhost:3000/auth/register
- **Dashboard**: http://localhost:3000/dashboard (requires login)

## 📋 Complete Authentication Flow

### **User Registration**

1. Visit http://localhost:3000/auth/register
2. Fill out the registration form:

   - **Full Name**: Your full name
   - **Email**: Valid email address
   - **Password**: At least 6 characters
   - **Confirm Password**: Must match password
   - **Terms**: Must accept terms of service
   - **Newsletter**: Optional newsletter signup

3. **Form Validation**:

   - Client-side validation with Bootstrap
   - Server-side validation with Laravel-style error messages
   - Flash messages for success/error states

4. **Success Flow**:
   - User account created with hashed password
   - Automatic login after registration
   - Session created with user data
   - Redirect to dashboard with welcome message

### **User Login**

1. Visit http://localhost:3000/auth/login
2. Enter credentials:

   - **Email**: Your registered email
   - **Password**: Your password
   - **Remember Me**: Optional persistent session

3. **Authentication Process**:

   - Password verification with Argon2
   - Account lockout after failed attempts (5 tries)
   - Session creation with user context
   - Activity logging for security

4. **Success Flow**:
   - Session authenticated flag set
   - User data stored in session
   - Redirect to intended page or dashboard

### **Password Reset Flow**

1. **Request Reset**: Visit `/auth/forgot-password`

   - Enter email address
   - System sends reset link (if email exists)
   - Always shows success message (security)

2. **Reset Password**: Click link in email `/auth/reset-password/{token}`

   - Token validation (24-hour expiry)
   - New password form with confirmation
   - Password complexity requirements

3. **Complete Reset**:
   - Password updated with new hash
   - Token invalidated
   - Redirect to login with success message

### **Change Password (Authenticated)**

1. Login and visit `/auth/change-password`
2. Form requires:

   - **Current Password**: For verification
   - **New Password**: At least 6 characters
   - **Confirm Password**: Must match new password

3. **Validation Process**:
   - Current password verification
   - New password complexity check
   - Real-time password matching

### **Session Management**

- **Session Creation**: Automatic on login/register
- **Session Storage**: Database-backed session store
- **Session Data**:
  ```json
  {
    "user_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
    "authenticated": true,
    "user_name": "John Doe",
    "user_email": "john@example.com"
  }
  ```
- **Session Security**: ID regeneration on auth state changes

## 🎨 User Interface Features

### **Modern Design**

- **Bootstrap 5**: Responsive, mobile-first design
- **Font Awesome**: Professional icons
- **Gradient Backgrounds**: Modern visual appeal
- **Card-based Layout**: Clean, organized forms

### **Form Features**

- **Real-time Validation**: Instant feedback
- **Password Visibility Toggle**: Show/hide passwords
- **Form Persistence**: Restore form data on validation errors
- **Loading States**: Visual feedback during submission

### **Flash Messages**

- **Success Messages**: Green alerts for successful actions
- **Error Messages**: Red alerts for errors
- **Auto-dismiss**: Messages fade after 5 seconds
- **Contextual Icons**: Visual message type indicators

### **Security Indicators**

- **Password Strength**: Visual strength indicators
- **Security Tips**: Helpful password guidance
- **Account Lockout**: Clear lockout messaging
- **Token Expiry**: Reset link expiration notices

## 🛡️ Security Features

### **Password Security**

- **Argon2 Hashing**: Industry-standard password hashing
- **Salt Generation**: Unique salt per password
- **Complexity Requirements**: Minimum 6 characters (configurable)
- **Password History**: Prevents password reuse

### **Session Security**

- **Session Fixation Protection**: ID regeneration
- **Secure Cookies**: HttpOnly, Secure flags
- **Session Timeout**: Configurable session lifetime
- **CSRF Protection**: Built-in token validation

### **Account Protection**

- **Rate Limiting**: Failed login attempt limits
- **Account Lockout**: Temporary lockout after failures
- **Activity Logging**: All auth attempts logged
- **Email Enumeration Protection**: Consistent responses

## 🔧 Configuration

### **Session Settings**

```env
SESSION_DRIVER=database
SESSION_LIFETIME=120
SESSION_ENCRYPT=false
SESSION_COOKIE=rustaxum_session
SESSION_SECURE=false
SESSION_HTTP_ONLY=true
SESSION_SAME_SITE=lax
```

### **Authentication Settings**

```env
JWT_SECRET=your-jwt-secret-key
PASSWORD_RESET_EXPIRY_HOURS=24
MAX_FAILED_LOGIN_ATTEMPTS=5
LOCKOUT_DURATION_MINUTES=30
```

## 📱 Mobile Responsiveness

- **Mobile-First Design**: Optimized for all screen sizes
- **Touch-Friendly**: Large buttons and touch targets
- **Responsive Forms**: Adapts to screen orientation
- **Mobile Navigation**: Collapsible navigation menu

## 🧪 Testing the System

### **Manual Testing Flow**

1. **Registration Test**:

   1. Visit /auth/register
   2. Try invalid data (see validation)
   3. Register valid user
   4. Verify redirect to dashboard
   5. Check session data

2. **Login Test**:

   1. Logout if logged in
   2. Visit /auth/login
   3. Try wrong password (count failures)
   4. Login with correct credentials
   5. Verify session restoration

3. **Password Reset Test**:

   1. Visit /auth/forgot-password
   2. Enter registered email
   3. Check email for reset link
   4. Click link and reset password
   5. Login with new password

4. **Session Test**:
   1. Login and visit /dashboard
   2. Close browser (session persists)
   3. Reopen and verify still logged in
   4. Test session timeout

### **Automated Testing**

```bash
# Run all tests
cargo test

# Test specific authentication features
cargo test auth
cargo test session
cargo test password
```

## 🎯 Laravel Compatibility

The web authentication system provides Laravel-equivalent features:

### **Laravel Feature → RustAxum Implementation**

| Laravel          | RustAxum                | Description         |
| ---------------- | ----------------------- | ------------------- |
| `Auth::login()`  | Session-based login     | User authentication |
| `Auth::logout()` | POST `/auth/logout`     | Session termination |
| `Auth::user()`   | Session user data       | Current user access |
| `@auth` Blade    | `auth_guard` middleware | Route protection    |
| `@guest` Blade   | Template conditionals   | Guest-only content  |
| Flash messages   | Session flash data      | Temporary messages  |
| Form validation  | Server-side validation  | Input validation    |
| Password reset   | Email token flow        | Password recovery   |
| Remember me      | Extended sessions       | Persistent login    |

### **Middleware Protection**

```rust
// Protected routes (like Laravel's 'auth' middleware)
.route("/dashboard", get(WebAuthController::dashboard))
.route("/auth/change-password", get(WebAuthController::show_change_password))
.route_layer(middleware::from_fn(auth_guard));

// Public routes (like Laravel's 'guest' middleware)
.route("/auth/login", get(WebAuthController::show_login))
.route("/auth/register", get(WebAuthController::show_register))
```

## 🌟 Advanced Features

### **Multi-Authentication Support**

- **Web Sessions**: Cookie-based authentication for browsers
- **API Tokens**: JWT tokens for API access
- **Unified Middleware**: Single middleware supports both methods

### **Template System**

- **Layout Inheritance**: Base layouts with content blocks
- **Partial Templates**: Reusable template components
- **Data Binding**: Server-side template rendering
- **Flash Data**: Automatic message handling

### **Error Handling**

- **Graceful Degradation**: Fallback for failed operations
- **User-Friendly Messages**: Clear error communication
- **Debug Information**: Detailed logs for development
- **Recovery Options**: Clear paths to resolution

## 🚨 Troubleshooting

### **Common Issues**

1. **Session Not Persisting**:

   - Check database connection
   - Verify session middleware is applied
   - Check cookie settings

2. **Form Validation Errors**:

   - Verify CSRF token inclusion
   - Check form field names
   - Validate input data types

3. **Password Reset Not Working**:

   - Check email configuration
   - Verify token expiration settings
   - Check database token storage

4. **Template Not Rendering**:
   - Verify template file paths
   - Check template syntax
   - Validate data passed to templates

### **Debug Commands**

```bash
# Check session storage
docker-compose exec postgres psql -U rustaxum -d rustaxum -c "SELECT * FROM sessions LIMIT 5;"

# View recent activity logs
docker-compose exec postgres psql -U rustaxum -d rustaxum -c "SELECT * FROM activity_log WHERE log_name = 'authentication' ORDER BY created_at DESC LIMIT 10;"

# Check application logs
docker-compose logs -f app | grep -i auth
```

The web authentication system provides a complete, production-ready authentication solution with modern UI/UX, comprehensive security features, and Laravel-like developer experience.
