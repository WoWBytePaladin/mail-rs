use std::collections::HashMap;
use chrono::{DateTime, Utc};
use crate::address::Address;

/// Email header field
#[derive(Debug, Clone)]
pub struct Header {
    fields: HashMap<String, Vec<String>>,
}

impl Header {
    /// Create a new empty header
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    /// Set a header field with a single value
    pub fn set(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.fields.insert(name.into(), vec![value.into()]);
    }

    /// Set a header field with multiple values
    pub fn set_many(&mut self, name: impl Into<String>, values: Vec<String>) {
        self.fields.insert(name.into(), values);
    }

    /// Add a value to an existing header field
    pub fn add(&mut self, name: impl Into<String>, value: impl Into<String>) {
        let name = name.into();
        self.fields.entry(name).or_insert_with(Vec::new).push(value.into());
    }

    /// Get header field values
    pub fn get(&self, name: &str) -> Option<&Vec<String>> {
        self.fields.get(name)
    }

    /// Get the first value of a header field
    pub fn get_first(&self, name: &str) -> Option<&String> {
        self.fields.get(name).and_then(|v| v.first())
    }

    /// Remove a header field
    pub fn remove(&mut self, name: &str) -> Option<Vec<String>> {
        self.fields.remove(name)
    }

    /// Check if a header field exists
    pub fn contains(&self, name: &str) -> bool {
        self.fields.contains_key(name)
    }

    /// Set an address header field
    pub fn set_address(&mut self, name: impl Into<String>, address: Address) {
        self.set(name, address.format());
    }

    /// Set an address header field with multiple addresses
    pub fn set_addresses(&mut self, name: impl Into<String>, addresses: Vec<Address>) {
        let formatted: Vec<String> = addresses.iter().map(|a| a.format()).collect();
        self.set_many(name, formatted);
    }

    /// Set a date header field
    pub fn set_date(&mut self, name: impl Into<String>, date: DateTime<Utc>) {
        self.set(name, date.to_rfc2822());
    }

    /// Encode header value for MIME (RFC 2047)
    pub fn encode_value(value: &str, charset: &str) -> String {
        if needs_encoding(value) {
            encode_rfc2047(value, charset)
        } else {
            value.to_string()
        }
    }

    /// Get all header fields
    pub fn fields(&self) -> &HashMap<String, Vec<String>> {
        &self.fields
    }

    /// Get mutable reference to all header fields
    pub fn fields_mut(&mut self) -> &mut HashMap<String, Vec<String>> {
        &mut self.fields
    }

    /// Format header as string for email message
    pub fn format(&self, charset: &str) -> String {
        let mut result = String::new();
        
        for (name, values) in &self.fields {
            for value in values {
                let encoded_value = Self::encode_value(value, charset);
                result.push_str(name);
                result.push_str(": ");
                result.push_str(&encoded_value);
                result.push_str("\r\n");
            }
        }
        
        result
    }
}

impl Default for Header {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a string needs MIME encoding
fn needs_encoding(s: &str) -> bool {
    s.chars().any(|c| !c.is_ascii() || c.is_control())
}

/// Encode a string using RFC 2047 (MIME header encoding)
fn encode_rfc2047(s: &str, charset: &str) -> String {
    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(s.as_bytes());
    format!("=?{}?B?{}?=", charset, encoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_set_get() {
        let mut header = Header::new();
        header.set("Subject", "Test");
        assert_eq!(header.get_first("Subject"), Some(&"Test".to_string()));
    }

    #[test]
    fn test_header_add() {
        let mut header = Header::new();
        header.add("To", "test1@example.com");
        header.add("To", "test2@example.com");
        
        let values = header.get("To").unwrap();
        assert_eq!(values.len(), 2);
        assert_eq!(values[0], "test1@example.com");
        assert_eq!(values[1], "test2@example.com");
    }

    #[test]
    fn test_header_address() {
        let mut header = Header::new();
        let addr = Address::with_name("test@example.com", "Test User");
        header.set_address("From", addr);
        
        assert_eq!(header.get_first("From"), Some(&"Test User <test@example.com>".to_string()));
    }

    #[test]
    fn test_needs_encoding() {
        assert!(!needs_encoding("Hello"));
        assert!(needs_encoding("Hëllo"));
        assert!(needs_encoding("你好"));
    }
}
