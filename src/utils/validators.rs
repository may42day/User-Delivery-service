use validator::ValidationError;

// Custom validator of field role for model CreateUser
pub fn validate_role(role: &str) -> Result<(), ValidationError> {
    if role == "USER" || role == "COURIER" {
        Ok(())
    } else {
        Err(ValidationError::new("Role Validation Failed"))
    }
}
