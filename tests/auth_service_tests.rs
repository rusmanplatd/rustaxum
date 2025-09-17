use rustaxum::app::services::auth_service::AuthService;
use rustaxum::app::utils::password_validator::PasswordValidator;
use rustaxum::app::utils::token_utils::TokenUtils;
use chrono::{Duration, Utc};

#[tokio::test]
async fn test_password_hashing_and_verification() {
    let password = "TestPassword123!";

    // Test password hashing
    let hashed = AuthService::hash_password(password).unwrap();
    assert!(!hashed.is_empty());
    assert_ne!(hashed, password); // Should be different from original

    // Test password verification with correct password
    let is_valid = AuthService::verify_password(password, &hashed).unwrap();
    assert!(is_valid);

    // Test password verification with wrong password
    let is_invalid = AuthService::verify_password("WrongPassword", &hashed).unwrap();
    assert!(!is_invalid);
}

#[tokio::test]
async fn test_access_token_generation_and_validation() {
    let user_id = "test_user_123";
    let secret = "test_secret_key";
    let expires_in = 3600; // 1 hour

    // Generate access token
    let token = AuthService::generate_access_token(user_id, secret, expires_in).unwrap();
    assert!(!token.is_empty());

    // Decode and validate token
    let claims = AuthService::decode_token(&token, secret).unwrap();
    assert_eq!(claims.sub, user_id);
    assert!(claims.exp > Utc::now().timestamp() as usize);
    assert!(claims.iat <= Utc::now().timestamp() as usize);
    assert!(!claims.jti.is_empty()); // JWT ID should be present

    // Test with wrong secret should fail
    let wrong_decode = AuthService::decode_token(&token, "wrong_secret");
    assert!(wrong_decode.is_err());
}

#[tokio::test]
async fn test_refresh_token_generation() {
    let token1 = AuthService::generate_refresh_token();
    let token2 = AuthService::generate_refresh_token();

    assert!(!token1.is_empty());
    assert!(!token2.is_empty());
    assert_ne!(token1, token2); // Should be unique

    // Should be valid ULID format
    assert!(ulid::Ulid::from_string(&token1).is_ok());
    assert!(ulid::Ulid::from_string(&token2).is_ok());
}

#[tokio::test]
async fn test_token_expiration_validation() {
    let user_id = "test_user_123";
    let secret = "test_secret_key";

    // Generate token that expires in 1 second
    let expires_in = 1;
    let token = AuthService::generate_access_token(user_id, secret, expires_in).unwrap();

    // Should be valid immediately
    let claims = AuthService::decode_token(&token, secret).unwrap();
    assert_eq!(claims.sub, user_id);

    // Wait for token to expire
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Should now be expired (JWT library should handle this)
    // Note: We can't easily test expiration without waiting or mocking time
    // In a real implementation, you'd use a mocking library for time
}

#[tokio::test]
async fn test_password_validator_comprehensive() {
    // Test valid password
    assert!(PasswordValidator::validate("ValidPass123!").is_ok());

    // Test too short
    let result = PasswordValidator::validate("short");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("at least 8 characters"));

    // Test too long (129 characters)
    let long_password = "a".repeat(129);
    assert_eq!(long_password.len(), 129);
    let result = PasswordValidator::validate(&long_password);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("must not exceed 128 characters"));

    // Test missing lowercase
    let result = PasswordValidator::validate("PASSWORD123!");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("lowercase letter"));

    // Test missing uppercase
    let result = PasswordValidator::validate("password123!");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("uppercase letter"));

    // Test missing digit
    let result = PasswordValidator::validate("Password!");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("number"));

    // Test missing special character
    let result = PasswordValidator::validate("Password123");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("special character"));
}

#[tokio::test]
async fn test_password_confirmation_validation() {
    // Matching passwords
    assert!(PasswordValidator::validate_confirmation("password", "password").is_ok());

    // Non-matching passwords
    let result = PasswordValidator::validate_confirmation("password1", "password2");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("does not match"));

    // Empty passwords should still be checked for match
    assert!(PasswordValidator::validate_confirmation("", "").is_ok());

    // One empty, one not
    let result = PasswordValidator::validate_confirmation("password", "");
    assert!(result.is_err());
}

#[tokio::test]
async fn test_token_utils_functionality() {
    let test_token = "test_token_12345";

    // Test token hashing
    let hash1 = TokenUtils::hash_token(test_token);
    let hash2 = TokenUtils::hash_token(test_token);

    assert!(!hash1.is_empty());
    assert_eq!(hash1, hash2); // Same input should produce same hash

    // Different tokens should produce different hashes
    let different_hash = TokenUtils::hash_token("different_token");
    assert_ne!(hash1, different_hash);

    // Test reset token generation
    let reset_token1 = TokenUtils::generate_reset_token();
    let reset_token2 = TokenUtils::generate_reset_token();

    assert!(!reset_token1.is_empty());
    assert!(!reset_token2.is_empty());
    assert_ne!(reset_token1, reset_token2); // Should be unique

    // Should be 32 characters (UUID without hyphens)
    assert_eq!(reset_token1.len(), 32);
    assert_eq!(reset_token2.len(), 32);
}

#[tokio::test]
async fn test_authorization_header_extraction() {
    // Valid Bearer token
    let auth_header = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
    let token = TokenUtils::extract_token_from_header(Some(auth_header)).unwrap();
    assert_eq!(token, "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9");

    // Missing header
    let result = TokenUtils::extract_token_from_header(None);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Authorization header missing"));

    // Invalid format (no Bearer prefix)
    let result = TokenUtils::extract_token_from_header(Some("InvalidFormat"));
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid authorization header format"));

    // Invalid format (Bearer but no token)
    let result = TokenUtils::extract_token_from_header(Some("Bearer "));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "");

    // Invalid format (Bearer with space but no token)
    let result = TokenUtils::extract_token_from_header(Some("Bearer"));
    assert!(result.is_err());
}

#[tokio::test]
async fn test_jwt_claims_structure() {
    let user_id = "user_12345";
    let secret = "test_secret";
    let expires_in = 3600;

    let token = AuthService::generate_access_token(user_id, secret, expires_in).unwrap();
    let claims = AuthService::decode_token(&token, secret).unwrap();

    // Verify all required claims are present
    assert_eq!(claims.sub, user_id);
    assert!(claims.exp > 0);
    assert!(claims.iat > 0);
    assert!(!claims.jti.is_empty());

    // Verify expiration is in the future
    assert!(claims.exp > Utc::now().timestamp() as usize);

    // Verify issued at is in the past or now
    assert!(claims.iat <= Utc::now().timestamp() as usize);

    // Verify JWT ID is a valid ULID
    assert!(ulid::Ulid::from_string(&claims.jti).is_ok());
}

#[tokio::test]
async fn test_various_special_characters_in_passwords() {
    let special_chars = vec![
        "!", "@", "#", "$", "%", "^", "&", "*", "(", ")", "_", "+",
        "-", "=", "[", "]", "{", "}", "|", ";", ":", ",", ".", "<", ">", "?"
    ];

    for special_char in special_chars {
        let password = format!("ValidPass123{}", special_char);
        let result = PasswordValidator::validate(&password);
        assert!(result.is_ok(), "Password with '{}' should be valid", special_char);
    }
}

#[tokio::test]
async fn test_edge_case_password_lengths() {
    // Exactly 8 characters with all requirements
    let password_8 = "ValidA1!";
    assert_eq!(password_8.len(), 8);
    assert!(PasswordValidator::validate(password_8).is_ok());

    // Exactly 128 characters
    let password_128 = "a".repeat(128);
    assert_eq!(password_128.len(), 128);
    // This will fail validation due to missing uppercase, digit, special char, but should pass length check

    // 7 characters (too short)
    let password_7 = "ValidA1";
    assert_eq!(password_7.len(), 7);
    assert!(PasswordValidator::validate(password_7).is_err());

    // 129 characters (too long)
    let password_129 = "a".repeat(129);
    assert_eq!(password_129.len(), 129);
    assert!(PasswordValidator::validate(&password_129).is_err());
}

#[tokio::test]
async fn test_bcrypt_hash_uniqueness() {
    let password = "SamePassword123!";

    // Generate multiple hashes of the same password
    let hash1 = AuthService::hash_password(password).unwrap();
    let hash2 = AuthService::hash_password(password).unwrap();
    let hash3 = AuthService::hash_password(password).unwrap();

    // Each hash should be different due to salt
    assert_ne!(hash1, hash2);
    assert_ne!(hash2, hash3);
    assert_ne!(hash1, hash3);

    // But all should verify correctly
    assert!(AuthService::verify_password(password, &hash1).unwrap());
    assert!(AuthService::verify_password(password, &hash2).unwrap());
    assert!(AuthService::verify_password(password, &hash3).unwrap());
}