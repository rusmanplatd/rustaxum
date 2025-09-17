use rustaxum::app::utils::password_validator::PasswordValidator;

#[tokio::test]
async fn test_password_length_validation() {
    // Too short
    let result = PasswordValidator::validate("short");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("at least 8 characters"));

    // Too long
    let long_password = "a".repeat(129);
    let result = PasswordValidator::validate(&long_password);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("must not exceed 128 characters"));

    // Just right
    let result = PasswordValidator::validate("ValidPass123!");
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_password_complexity_validation() {
    // Missing lowercase
    let result = PasswordValidator::validate("PASSWORD123!");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("lowercase letter"));

    // Missing uppercase
    let result = PasswordValidator::validate("password123!");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("uppercase letter"));

    // Missing digit
    let result = PasswordValidator::validate("Password!");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("number"));

    // Missing special character
    let result = PasswordValidator::validate("Password123");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("special character"));

    // Valid password
    let result = PasswordValidator::validate("ValidPass123!");
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_password_confirmation_validation() {
    // Matching passwords
    let result = PasswordValidator::validate_confirmation("password", "password");
    assert!(result.is_ok());

    // Non-matching passwords
    let result = PasswordValidator::validate_confirmation("password1", "password2");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("does not match"));
}

#[tokio::test]
async fn test_various_special_characters() {
    let special_chars = ["!", "@", "#", "$", "%", "^", "&", "*", "(", ")", "_", "+", "-", "=", "[", "]", "{", "}", "|", ";", ":", ",", ".", "<", ">", "?"];

    for &special_char in &special_chars {
        let password = format!("ValidPass123{}", special_char);
        let result = PasswordValidator::validate(&password);
        assert!(result.is_ok(), "Password with {} should be valid", special_char);
    }
}

#[tokio::test]
async fn test_edge_case_passwords() {
    // Exactly 8 characters with all requirements
    let result = PasswordValidator::validate("Aa1!");
    assert!(result.is_err()); // Should fail because too short

    let result = PasswordValidator::validate("ValidA1!");
    assert!(result.is_ok()); // Should pass - exactly 8 chars with all requirements

    // Exactly 128 characters
    let password_128 = format!("{}A1!", "a".repeat(125));
    assert_eq!(password_128.len(), 128);
    let result = PasswordValidator::validate(&password_128);
    assert!(result.is_ok());
}