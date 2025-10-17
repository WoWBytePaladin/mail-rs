use std::io::Write;
use std::path::Path;
use chrono::Utc;
use crate::{Address, Header, Encoding, Result, Error};

/// Content type for message parts
#[derive(Debug, Clone)]
pub struct ContentType {
    pub mime_type: String,
    pub charset: Option<String>,
    pub boundary: Option<String>,
}

impl ContentType {
    pub fn new(mime_type: impl Into<String>) -> Self {
        Self {
            mime_type: mime_type.into(),
            charset: None,
            boundary: None,
        }
    }

    pub fn with_charset(mut self, charset: impl Into<String>) -> Self {
        self.charset = Some(charset.into());
        self
    }

    pub fn format(&self) -> String {
        let mut result = self.mime_type.clone();
        if let Some(charset) = &self.charset {
            result.push_str("; charset=");
            result.push_str(charset);
        }
        if let Some(boundary) = &self.boundary {
            result.push_str(";\r\n boundary=");
            result.push_str(boundary);
        }
        result
    }
}

/// Message body part
#[derive(Debug, Clone)]
pub struct Part {
    pub content_type: ContentType,
    pub encoding: Encoding,
    pub content: Vec<u8>,
}

impl Part {
    pub fn new(content_type: ContentType, content: Vec<u8>) -> Self {
        Self {
            content_type,
            encoding: Encoding::default(),
            content,
        }
    }

    pub fn with_encoding(mut self, encoding: Encoding) -> Self {
        self.encoding = encoding;
        self
    }
}

/// File attachment
#[derive(Debug, Clone)]
pub struct Attachment {
    pub filename: String,
    pub content_type: String,
    pub content: Vec<u8>,
    pub inline: bool,
    pub content_id: Option<String>,
}

impl Attachment {
    pub fn new(filename: impl Into<String>, content: Vec<u8>) -> Self {
        let filename = filename.into();
        let content_type = mime_guess::from_path(&filename)
            .first_or_octet_stream()
            .to_string();

        Self {
            filename,
            content_type,
            content,
            inline: false,
            content_id: None,
        }
    }

    pub fn inline(mut self) -> Self {
        self.inline = true;
        self
    }

    pub fn with_content_id(mut self, content_id: impl Into<String>) -> Self {
        self.content_id = Some(content_id.into());
        self
    }
}

/// Email message builder
#[derive(Debug)]
pub struct Message {
    header: Header,
    charset: String,
    encoding: Encoding,
    parts: Vec<Part>,
    attachments: Vec<Attachment>,
    embedded: Vec<Attachment>,
}

impl Message {
    /// Create a new message with UTF-8 charset and quoted-printable encoding
    pub fn new() -> Self {
        Self {
            header: Header::new(),
            charset: "UTF-8".to_string(),
            encoding: Encoding::QuotedPrintable,
            parts: Vec::new(),
            attachments: Vec::new(),
            embedded: Vec::new(),
        }
    }

    /// Set the charset for the message
    pub fn charset(mut self, charset: impl Into<String>) -> Self {
        self.charset = charset.into();
        self
    }

    /// Set the default encoding for the message
    pub fn encoding(mut self, encoding: Encoding) -> Self {
        self.encoding = encoding;
        self
    }

    /// Set a header field
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.header.set(name, value);
        self
    }

    /// Set multiple values for a header field
    pub fn header_many(mut self, name: impl Into<String>, values: Vec<String>) -> Self {
        self.header.set_many(name, values);
        self
    }

    /// Set the From header
    pub fn from(mut self, address: Address) -> Self {
        self.header.set_address("From", address);
        self
    }

    /// Set the To header
    pub fn to(mut self, addresses: Vec<Address>) -> Self {
        self.header.set_addresses("To", addresses);
        self
    }

    /// Set the Cc header
    pub fn cc(mut self, addresses: Vec<Address>) -> Self {
        self.header.set_addresses("Cc", addresses);
        self
    }

    /// Set the Bcc header
    pub fn bcc(mut self, addresses: Vec<Address>) -> Self {
        self.header.set_addresses("Bcc", addresses);
        self
    }

    /// Set the Reply-To header
    pub fn reply_to(mut self, address: Address) -> Self {
        self.header.set_address("Reply-To", address);
        self
    }

    /// Set the subject
    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.header.set("Subject", subject);
        self
    }

    /// Set the Date header to current time
    pub fn date_now(mut self) -> Self {
        self.header.set_date("Date", Utc::now());
        self
    }

    /// Set the message body
    pub fn body(mut self, content_type: impl Into<String>, content: impl Into<Vec<u8>>) -> Self {
        let ct = ContentType::new(content_type).with_charset(self.charset.clone());
        let part = Part::new(ct, content.into()).with_encoding(self.encoding);
        self.parts = vec![part];
        self
    }

    /// Add an alternative body (for multipart/alternative)
    pub fn alternative(mut self, content_type: impl Into<String>, content: impl Into<Vec<u8>>) -> Self {
        let ct = ContentType::new(content_type).with_charset(self.charset.clone());
        let part = Part::new(ct, content.into()).with_encoding(self.encoding);
        self.parts.push(part);
        self
    }

    /// Attach a file
    pub fn attach(mut self, filename: impl Into<String>, content: Vec<u8>) -> Self {
        self.attachments.push(Attachment::new(filename, content));
        self
    }

    /// Attach a file from path
    pub fn attach_file(self, path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read(path)?;
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| Error::Custom("Invalid filename".to_string()))?;
        
        Ok(self.attach(filename, content))
    }

    /// Embed a file (for inline images)
    pub fn embed(mut self, filename: impl Into<String>, content: Vec<u8>) -> Self {
        self.embedded.push(Attachment::new(filename, content).inline());
        self
    }

    /// Embed a file from path
    pub fn embed_file(self, path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read(path)?;
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| Error::Custom("Invalid filename".to_string()))?;
        
        Ok(self.embed(filename, content))
    }

    /// Reset the message to reuse it
    pub fn reset(&mut self) {
        self.header = Header::new();
        self.parts.clear();
        self.attachments.clear();
        self.embedded.clear();
    }

    /// Get the message header
    pub fn get_header(&self) -> &Header {
        &self.header
    }

    /// Get mutable reference to the message header
    pub fn get_header_mut(&mut self) -> &mut Header {
        &mut self.header
    }

    /// Check if message has multiple parts
    pub fn is_multipart(&self) -> bool {
        self.parts.len() > 1 || !self.attachments.is_empty() || !self.embedded.is_empty()
    }

    /// Get the From address
    pub fn from_address(&self) -> Option<String> {
        self.header.get_first("From").map(|s| extract_email(s))
    }

    /// Get all recipients (To, Cc, Bcc)
    pub fn recipients(&self) -> Vec<String> {
        let mut recipients = Vec::new();
        
        for field in &["To", "Cc", "Bcc"] {
            if let Some(values) = self.header.get(field) {
                for value in values {
                    recipients.push(extract_email(value));
                }
            }
        }
        
        recipients
    }

    /// Format the message as bytes
    pub fn format(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        self.write_to(&mut buffer)?;
        Ok(buffer)
    }

    /// Write the message to a writer
    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        // Add MIME-Version if not present
        if !self.header.contains("Mime-Version") {
            writer.write_all(b"Mime-Version: 1.0\r\n")?;
        }

        // Add Date if not present
        if !self.header.contains("Date") {
            let date = Utc::now().to_rfc2822();
            writer.write_all(b"Date: ")?;
            writer.write_all(date.as_bytes())?;
            writer.write_all(b"\r\n")?;
        }

        // Write headers (except Bcc)
        for (name, values) in self.header.fields() {
            if name == "Bcc" {
                continue;
            }
            for value in values {
                let encoded = Header::encode_value(value, &self.charset);
                writer.write_all(name.as_bytes())?;
                writer.write_all(b": ")?;
                writer.write_all(encoded.as_bytes())?;
                writer.write_all(b"\r\n")?;
            }
        }

        // Handle message body
        if self.is_multipart() {
            self.write_multipart(writer)?;
        } else {
            self.write_single_part(writer)?;
        }

        Ok(())
    }

    fn write_single_part<W: Write>(&self, writer: &mut W) -> Result<()> {
        if let Some(part) = self.parts.first() {
            writer.write_all(b"Content-Type: ")?;
            writer.write_all(part.content_type.format().as_bytes())?;
            writer.write_all(b"\r\n")?;
            
            writer.write_all(b"Content-Transfer-Encoding: ")?;
            writer.write_all(part.encoding.as_str().as_bytes())?;
            writer.write_all(b"\r\n\r\n")?;
            
            let encoded = part.encoding.encode(&part.content);
            writer.write_all(&encoded)?;
        } else if self.attachments.len() == 1 {
            // Single attachment without body
            let attachment = &self.attachments[0];
            self.write_attachment(writer, attachment)?;
        }
        
        Ok(())
    }

    fn write_multipart<W: Write>(&self, writer: &mut W) -> Result<()> {
        let boundary = generate_boundary();
        
        // Determine multipart type
        let multipart_type = if !self.attachments.is_empty() {
            "mixed"
        } else if !self.embedded.is_empty() {
            "related"
        } else {
            "alternative"
        };

        writer.write_all(b"Content-Type: multipart/")?;
        writer.write_all(multipart_type.as_bytes())?;
        writer.write_all(b";\r\n boundary=")?;
        writer.write_all(boundary.as_bytes())?;
        writer.write_all(b"\r\n\r\n")?;

        // Write body parts
        for part in &self.parts {
            writer.write_all(b"--")?;
            writer.write_all(boundary.as_bytes())?;
            writer.write_all(b"\r\n")?;
            
            writer.write_all(b"Content-Type: ")?;
            writer.write_all(part.content_type.format().as_bytes())?;
            writer.write_all(b"\r\n")?;
            
            writer.write_all(b"Content-Transfer-Encoding: ")?;
            writer.write_all(part.encoding.as_str().as_bytes())?;
            writer.write_all(b"\r\n\r\n")?;
            
            let encoded = part.encoding.encode(&part.content);
            writer.write_all(&encoded)?;
            writer.write_all(b"\r\n")?;
        }

        // Write embedded files
        for embedded in &self.embedded {
            writer.write_all(b"--")?;
            writer.write_all(boundary.as_bytes())?;
            writer.write_all(b"\r\n")?;
            self.write_attachment(writer, embedded)?;
        }

        // Write attachments
        for attachment in &self.attachments {
            writer.write_all(b"--")?;
            writer.write_all(boundary.as_bytes())?;
            writer.write_all(b"\r\n")?;
            self.write_attachment(writer, attachment)?;
        }

        // End boundary
        writer.write_all(b"--")?;
        writer.write_all(boundary.as_bytes())?;
        writer.write_all(b"--\r\n")?;

        Ok(())
    }

    fn write_attachment<W: Write>(&self, writer: &mut W, attachment: &Attachment) -> Result<()> {
        writer.write_all(b"Content-Type: ")?;
        writer.write_all(attachment.content_type.as_bytes())?;
        writer.write_all(b"; name=\"")?;
        writer.write_all(attachment.filename.as_bytes())?;
        writer.write_all(b"\"\r\n")?;

        let disposition = if attachment.inline { "inline" } else { "attachment" };
        writer.write_all(b"Content-Disposition: ")?;
        writer.write_all(disposition.as_bytes())?;
        writer.write_all(b"; filename=\"")?;
        writer.write_all(attachment.filename.as_bytes())?;
        writer.write_all(b"\"\r\n")?;

        if let Some(content_id) = &attachment.content_id {
            writer.write_all(b"Content-ID: <")?;
            writer.write_all(content_id.as_bytes())?;
            writer.write_all(b">\r\n")?;
        } else if attachment.inline {
            writer.write_all(b"Content-ID: <")?;
            writer.write_all(attachment.filename.as_bytes())?;
            writer.write_all(b">\r\n")?;
        }

        writer.write_all(b"Content-Transfer-Encoding: base64\r\n\r\n")?;

        let encoded = Encoding::Base64.encode_with_line_wrap(&attachment.content, 76);
        writer.write_all(&encoded)?;
        writer.write_all(b"\r\n")?;

        Ok(())
    }
}

impl Default for Message {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a random boundary string
fn generate_boundary() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("_BOUNDARY_{}_", nanos)
}

/// Extract email address from formatted string
/// Extract email address from a header value like "Name <email@domain.com>"
fn extract_email(header_value: &str) -> String {
    let trimmed = header_value.trim();
    
    // Look for email in angle brackets
    if let Some(start) = trimmed.rfind('<') {
        if let Some(end) = trimmed.rfind('>') {
            if start < end {
                return trimmed[(start + 1)..end].to_string();
            }
        }
    }
    
    // No angle brackets, assume the whole string is an email
    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_simple() {
        let msg = Message::new()
            .from(Address::new("sender@example.com"))
            .to(vec![Address::new("recipient@example.com")])
            .subject("Test Subject")
            .body("text/plain", "Hello, World!");

        let formatted = msg.format().unwrap();
        let text = String::from_utf8(formatted).unwrap();
        
        assert!(text.contains("From: sender@example.com"));
        assert!(text.contains("To: recipient@example.com"));
        assert!(text.contains("Subject: Test Subject"));
        assert!(text.contains("Hello, World!"));
    }

    #[test]
    fn test_message_with_attachment() {
        let msg = Message::new()
            .from(Address::new("sender@example.com"))
            .to(vec![Address::new("recipient@example.com")])
            .subject("Test with Attachment")
            .body("text/plain", "See attachment")
            .attach("test.txt", b"File content".to_vec());

        let formatted = msg.format().unwrap();
        let text = String::from_utf8(formatted).unwrap();
        
        assert!(text.contains("multipart/mixed"));
        assert!(text.contains("test.txt"));
    }

    #[test]
    fn test_extract_email() {
        assert_eq!(extract_email("test@example.com"), "test@example.com");
        assert_eq!(extract_email("User <test@example.com>"), "test@example.com");
        assert_eq!(extract_email("\"User Name\" <test@example.com>"), "test@example.com");
    }
}
