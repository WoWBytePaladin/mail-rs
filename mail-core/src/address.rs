use std::fmt;

/// Email address representation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Address {
    /// Email address (e.g., "user@example.com")
    pub email: String,
    /// Display name (e.g., "John Doe")
    pub name: Option<String>,
}

impl Address {
    /// Create a new address with just an email
    pub fn new(email: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            name: None,
        }
    }

    /// Create a new address with email and display name
    pub fn with_name(email: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            name: Some(name.into()),
        }
    }

    /// Format address according to RFC 5322
    pub fn format(&self) -> String {
        match &self.name {
            Some(name) if needs_quoting(name) => {
                format!("\"{}\" <{}>", escape_quotes(name), self.email)
            }
            Some(name) => format!("{} <{}>", name, self.email),
            None => self.email.clone(),
        }
    }

    /// Validate email address format
    pub fn is_valid(&self) -> bool {
        self.email.contains('@') && !self.email.is_empty()
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format())
    }
}

impl From<String> for Address {
    fn from(email: String) -> Self {
        Self::new(email)
    }
}

impl From<&str> for Address {
    fn from(email: &str) -> Self {
        Self::new(email)
    }
}

fn needs_quoting(name: &str) -> bool {
    name.chars().any(|c| matches!(c, '(' | ')' | '<' | '>' | '[' | ']' | ':' | ';' | '@' | '\\' | ',' | '.' | '"'))
}

fn escape_quotes(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_new() {
        let addr = Address::new("test@example.com");
        assert_eq!(addr.email, "test@example.com");
        assert_eq!(addr.name, None);
    }

    #[test]
    fn test_address_with_name() {
        let addr = Address::with_name("test@example.com", "Test User");
        assert_eq!(addr.email, "test@example.com");
        assert_eq!(addr.name, Some("Test User".to_string()));
    }

    #[test]
    fn test_address_format() {
        let addr1 = Address::new("test@example.com");
        assert_eq!(addr1.format(), "test@example.com");

        let addr2 = Address::with_name("test@example.com", "Test User");
        assert_eq!(addr2.format(), "Test User <test@example.com>");

        let addr3 = Address::with_name("test@example.com", "User, Test");
        assert_eq!(addr3.format(), "\"User, Test\" <test@example.com>");
    }
}
