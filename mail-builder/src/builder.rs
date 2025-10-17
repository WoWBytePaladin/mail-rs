//! Message builder for convenient email construction

use mail_core::{Address, Message, Encoding};
use chrono::{DateTime, Utc};

/// Convenient builder for constructing emails
pub struct MessageBuilder {
    message: Message,
}

impl MessageBuilder {
    /// Create a new message builder
    pub fn new() -> Self {
        Self {
            message: Message::new(),
        }
    }

    /// Set the From address
    pub fn from(self, address: impl Into<String>) -> Self {
        let addr = Address::new(address.into());
        Self {
            message: self.message.from(addr),
        }
    }

    /// Set the From address with name
    pub fn from_with_name(self, email: impl Into<String>, name: impl Into<String>) -> Self {
        let addr = Address::with_name(email.into(), name.into());
        Self {
            message: self.message.from(addr),
        }
    }

    /// Add a To address
    pub fn to(self, address: impl Into<String>) -> Self {
        let addr = Address::new(address.into());
        let to_addrs = vec![addr];
        Self {
            message: self.message.to(to_addrs),
        }
    }

    /// Add a To address with name
    pub fn to_with_name(self, email: impl Into<String>, name: impl Into<String>) -> Self {
        let addr = Address::with_name(email.into(), name.into());
        let to_addrs = vec![addr];
        Self {
            message: self.message.to(to_addrs),
        }
    }

    /// Add a CC address
    pub fn cc(self, address: impl Into<String>) -> Self {
        let addr = Address::new(address.into());
        let cc_addrs = vec![addr];
        Self {
            message: self.message.cc(cc_addrs),
        }
    }

    /// Add a BCC address
    pub fn bcc(self, address: impl Into<String>) -> Self {
        let addr = Address::new(address.into());
        let bcc_addrs = vec![addr];
        Self {
            message: self.message.bcc(bcc_addrs),
        }
    }

    /// Set the Reply-To address
    pub fn reply_to(self, address: impl Into<String>) -> Self {
        let addr = Address::new(address.into());
        Self {
            message: self.message.reply_to(addr),
        }
    }

    /// Set the subject
    pub fn subject(self, subject: impl Into<String>) -> Self {
        Self {
            message: self.message.subject(subject.into()),
        }
    }

    /// Set the date to now
    pub fn date_now(self) -> Self {
        Self {
            message: self.message.date_now(),
        }
    }

    /// Set a custom date
    pub fn date(self, date: DateTime<Utc>) -> Self {
        Self {
            message: self.message.header("Date", date.to_rfc2822()),
        }
    }

    /// Set plain text body
    pub fn text_body(self, content: impl Into<String>) -> Self {
        Self {
            message: self.message.body("text/plain", content.into()),
        }
    }

    /// Set HTML body
    pub fn html_body(self, content: impl Into<String>) -> Self {
        Self {
            message: self.message.body("text/html", content.into()),
        }
    }

    /// Add an attachment
    pub fn attachment(
        self,
        filename: impl Into<String>,
        content: &[u8],
        _content_type: impl Into<String>,
    ) -> Self {
        Self {
            message: self.message.attach(filename.into(), content.to_vec()),
        }
    }

    /// Add an attachment from Vec<u8>
    pub fn attachment_from_data(
        self,
        filename: impl Into<String>,
        content: Vec<u8>,
        _content_type: impl Into<String>,
    ) -> Self {
        Self {
            message: self.message.attach(filename.into(), content),
        }
    }

    /// Add an embedded image
    pub fn embedded_image(
        self,
        cid: impl Into<String>,
        content: &[u8],
        _content_type: impl Into<String>,
    ) -> Self {
        Self {
            message: self.message.embed(cid.into(), content.to_vec()),
        }
    }

    /// Add a custom header
    pub fn header(self, name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            message: self.message.header(name.into(), value.into()),
        }
    }

    /// Set charset
    pub fn charset(self, charset: impl Into<String>) -> Self {
        Self {
            message: self.message.charset(charset.into()),
        }
    }

    /// Set encoding
    pub fn encoding(self, encoding: Encoding) -> Self {
        Self {
            message: self.message.encoding(encoding),
        }
    }

    /// Build the final message
    pub fn build(self) -> Message {
        self.message
    }
}

impl Default for MessageBuilder {
    fn default() -> Self {
        Self::new()
    }
}