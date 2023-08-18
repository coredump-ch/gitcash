use inquire::{
    validator::{ErrorMessage, StringValidator, Validation},
    CustomUserError,
};

#[derive(Debug, Clone)]
pub struct UsernameValidator {
    usernames: Vec<String>,
}

impl UsernameValidator {
    pub fn new(usernames: Vec<String>) -> Self {
        Self { usernames }
    }
}

impl StringValidator for UsernameValidator {
    fn validate(&self, input: &str) -> Result<Validation, CustomUserError> {
        Ok(if self.usernames.iter().any(|name| name == input) {
            Validation::Valid
        } else {
            Validation::Invalid(ErrorMessage::Custom(format!(
                "Not a known username: {}",
                input
            )))
        })
    }
}

#[derive(Debug, Clone)]
pub struct NewUsernameValidator {
    usernames: Vec<String>,
}

impl NewUsernameValidator {
    pub fn new(usernames: Vec<String>) -> Self {
        Self { usernames }
    }
}

impl StringValidator for NewUsernameValidator {
    fn validate(&self, input: &str) -> Result<Validation, CustomUserError> {
        let input = input.trim();
        Ok(if input.is_empty() {
            Validation::Invalid(ErrorMessage::Custom("Username may not be empty".into()))
        } else if input.contains(' ') {
            Validation::Invalid(ErrorMessage::Custom(
                "Username may not contain a space".into(),
            ))
        } else if input.contains(':') {
            Validation::Invalid(ErrorMessage::Custom(
                "Username may not contain a colon".into(),
            ))
        } else if self.usernames.iter().any(|name| name == input) {
            Validation::Invalid(ErrorMessage::Custom(format!(
                "Username already exists: {}",
                input
            )))
        } else {
            Validation::Valid
        })
    }
}
