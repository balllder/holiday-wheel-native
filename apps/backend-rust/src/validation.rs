//! Input validation module for request payloads.
//!
//! Provides validation helpers and error formatting for API requests.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use validator::ValidationErrors;

/// Standard validation error response format
#[derive(Debug, Serialize)]
pub struct ValidationErrorResponse {
    pub ok: bool,
    pub error: String,
    pub details: Vec<FieldError>,
}

/// Individual field validation error
#[derive(Debug, Serialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

impl ValidationErrorResponse {
    /// Create a new validation error response from validator errors
    pub fn from_validation_errors(errors: &ValidationErrors) -> Self {
        let details: Vec<FieldError> = errors
            .field_errors()
            .iter()
            .flat_map(|(field, errs)| {
                errs.iter().map(move |e| FieldError {
                    field: field.to_string(),
                    message: e
                        .message
                        .as_ref()
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| format!("Invalid value for {}", field)),
                })
            })
            .collect();

        Self {
            ok: false,
            error: "Validation failed".to_string(),
            details,
        }
    }
}

impl IntoResponse for ValidationErrorResponse {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, Json(self)).into_response()
    }
}

/// Validate email format using a standard regex pattern
pub fn validate_email_format(email: &str) -> bool {
    // Standard email regex pattern
    let email_regex =
        regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    email_regex.is_match(email)
}

/// Custom validator for email format
pub fn email_validator(email: &str) -> Result<(), validator::ValidationError> {
    if validate_email_format(email) {
        Ok(())
    } else {
        let mut err = validator::ValidationError::new("email_format");
        err.message = Some("Must be a valid email address".into());
        Err(err)
    }
}

/// Minimum password length requirement
pub const PASSWORD_MIN_LENGTH: usize = 8;

/// Custom validator for password strength
///
/// Enforces the following requirements:
/// - At least 8 characters long
/// - At least one lowercase letter
/// - At least one uppercase letter
/// - At least one digit
/// - At least one special character (!@#$%^&*(),.?":{}|<>-_+=)
pub fn password_validator(password: &str) -> Result<(), validator::ValidationError> {
    if password.len() < PASSWORD_MIN_LENGTH {
        let mut err = validator::ValidationError::new("password_length");
        err.message = Some(
            format!(
                "Password must be at least {} characters",
                PASSWORD_MIN_LENGTH
            )
            .into(),
        );
        return Err(err);
    }

    let has_lowercase = password.chars().any(|c| c.is_ascii_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password
        .chars()
        .any(|c| "!@#$%^&*(),.?\":{}|<>-_+=[]\\;'/~`".contains(c));

    let mut missing = Vec::new();

    if !has_lowercase {
        missing.push("lowercase letter");
    }
    if !has_uppercase {
        missing.push("uppercase letter");
    }
    if !has_digit {
        missing.push("number");
    }
    if !has_special {
        missing.push("special character");
    }

    if !missing.is_empty() {
        let mut err = validator::ValidationError::new("password_complexity");
        let missing_str = if missing.len() == 1 {
            format!("a {}", missing[0])
        } else {
            let last = missing.pop().unwrap();
            format!("a {}, and a {}", missing.join(", a "), last)
        };
        err.message = Some(format!("Password must contain {}", missing_str).into());
        return Err(err);
    }

    Ok(())
}

/// Custom validator for display name
pub fn display_name_validator(name: &str) -> Result<(), validator::ValidationError> {
    let trimmed = name.trim();

    if trimmed.len() < 2 {
        let mut err = validator::ValidationError::new("display_name_length");
        err.message = Some("Display name must be at least 2 characters".into());
        return Err(err);
    }

    if trimmed.len() > 24 {
        let mut err = validator::ValidationError::new("display_name_length");
        err.message = Some("Display name must be 24 characters or less".into());
        return Err(err);
    }

    // Check for valid characters (alphanumeric, spaces, common punctuation)
    if !trimmed
        .chars()
        .all(|c| c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' || c == '.')
    {
        let mut err = validator::ValidationError::new("display_name_chars");
        err.message = Some("Display name can only contain letters, numbers, spaces, hyphens, underscores, and periods".into());
        return Err(err);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validator_valid() {
        assert!(email_validator("test@example.com").is_ok());
        assert!(email_validator("user.name@domain.org").is_ok());
        assert!(email_validator("user+tag@sub.domain.com").is_ok());
    }

    #[test]
    fn test_email_validator_invalid() {
        assert!(email_validator("").is_err());
        assert!(email_validator("notanemail").is_err());
        assert!(email_validator("@nodomain.com").is_err());
        assert!(email_validator("noat.domain.com").is_err());
    }

    #[test]
    fn test_password_validator_valid() {
        // Meets all requirements: 8+ chars, lowercase, uppercase, number, special
        assert!(password_validator("Password1!").is_ok());
        assert!(password_validator("MySecure123@").is_ok());
        assert!(password_validator("Abcdefg1#").is_ok());
        assert!(password_validator("P@ssw0rd").is_ok());
        assert!(password_validator("Complex-Pass99").is_ok());
    }

    #[test]
    fn test_password_validator_too_short() {
        assert!(password_validator("Ab1!").is_err());
        assert!(password_validator("").is_err());
        assert!(password_validator("Short1!").is_err()); // 7 chars
    }

    #[test]
    fn test_password_validator_no_number() {
        assert!(password_validator("Password!").is_err());
    }

    #[test]
    fn test_password_validator_no_letter() {
        assert!(password_validator("12345678!").is_err());
    }

    #[test]
    fn test_password_validator_no_uppercase() {
        assert!(password_validator("password1!").is_err());
    }

    #[test]
    fn test_password_validator_no_lowercase() {
        assert!(password_validator("PASSWORD1!").is_err());
    }

    #[test]
    fn test_password_validator_no_special() {
        assert!(password_validator("Password1").is_err());
    }

    #[test]
    fn test_password_validator_error_messages() {
        // Test that error messages are descriptive
        let result = password_validator("abc");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.is_some());

        let result2 = password_validator("abcdefgh"); // Only lowercase
        assert!(result2.is_err());
        let err2 = result2.unwrap_err();
        assert!(err2.message.unwrap().to_string().contains("uppercase"));
    }

    #[test]
    fn test_display_name_validator_valid() {
        assert!(display_name_validator("John").is_ok());
        assert!(display_name_validator("User Name").is_ok());
        assert!(display_name_validator("Player-1").is_ok());
        assert!(display_name_validator("a_b").is_ok());
    }

    #[test]
    fn test_display_name_validator_too_short() {
        assert!(display_name_validator("a").is_err());
        assert!(display_name_validator("").is_err());
    }

    #[test]
    fn test_display_name_validator_too_long() {
        let long_name = "a".repeat(25);
        assert!(display_name_validator(&long_name).is_err());
    }

    #[test]
    fn test_display_name_validator_invalid_chars() {
        assert!(display_name_validator("user@name").is_err());
        assert!(display_name_validator("user<script>").is_err());
    }
}
