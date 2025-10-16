use std::fmt;

/// Email encoding schemes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    /// Quoted-Printable encoding (RFC 2045)
    QuotedPrintable,
    /// Base64 encoding (RFC 2045)
    Base64,
    /// 8-bit encoding (no encoding)
    EightBit,
    /// 7-bit encoding (ASCII only)
    SevenBit,
}

impl Encoding {
    /// Get the MIME encoding name
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::QuotedPrintable => "quoted-printable",
            Self::Base64 => "base64",
            Self::EightBit => "8bit",
            Self::SevenBit => "7bit",
        }
    }

    /// Encode data according to the encoding scheme
    pub fn encode(&self, data: &[u8]) -> Vec<u8> {
        match self {
            Self::QuotedPrintable => {
                quoted_printable::encode(data).into_bytes()
            }
            Self::Base64 => {
                use base64::Engine;
                base64::engine::general_purpose::STANDARD.encode(data).into_bytes()
            }
            Self::EightBit | Self::SevenBit => data.to_vec(),
        }
    }

    /// Encode data and wrap lines at specified length
    pub fn encode_with_line_wrap(&self, data: &[u8], line_length: usize) -> Vec<u8> {
        match self {
            Self::Base64 => {
                use base64::Engine;
                let encoded = base64::engine::general_purpose::STANDARD.encode(data);
                wrap_lines(&encoded, line_length).into_bytes()
            }
            _ => self.encode(data),
        }
    }
}

impl Default for Encoding {
    fn default() -> Self {
        Self::QuotedPrintable
    }
}

impl fmt::Display for Encoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Wrap text at specified line length
fn wrap_lines(text: &str, max_len: usize) -> String {
    let mut result = String::new();
    let mut pos = 0;
    
    while pos < text.len() {
        let end = (pos + max_len).min(text.len());
        result.push_str(&text[pos..end]);
        if end < text.len() {
            result.push_str("\r\n");
        }
        pos = end;
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoding_as_str() {
        assert_eq!(Encoding::QuotedPrintable.as_str(), "quoted-printable");
        assert_eq!(Encoding::Base64.as_str(), "base64");
        assert_eq!(Encoding::EightBit.as_str(), "8bit");
        assert_eq!(Encoding::SevenBit.as_str(), "7bit");
    }

    #[test]
    fn test_quoted_printable_encode() {
        let data = b"Hello, World!";
        let encoded = Encoding::QuotedPrintable.encode(data);
        assert_eq!(String::from_utf8(encoded).unwrap(), "Hello, World!");
    }

    #[test]
    fn test_base64_encode() {
        let data = b"Hello, World!";
        let encoded = Encoding::Base64.encode(data);
        assert_eq!(String::from_utf8(encoded).unwrap(), "SGVsbG8sIFdvcmxkIQ==");
    }
}
