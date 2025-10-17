use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use rustls::pki_types::ServerName;
use std::time::Duration;
use mail_core::Message;
use crate::{
    auth::{AuthMechanism, Credentials},
    transport::{SmtpTransport, TlsConfig},
    error::{Error, Result},
};

/// SMTP client for sending emails
pub struct SmtpClient {
    transport: SmtpTransport,
    credentials: Option<Credentials>,
    timeout: Option<Duration>,
}

impl SmtpClient {
    /// Create a new SMTP client
    pub fn new(transport: SmtpTransport) -> Self {
        Self {
            transport,
            credentials: None,
            timeout: Some(Duration::from_secs(30)),
        }
    }

    /// Set authentication credentials
    pub fn credentials(mut self, credentials: Credentials) -> Self {
        self.credentials = Some(credentials);
        self
    }

    /// Set connection timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Send an email message
    pub async fn send(&self, message: &Message) -> Result<()> {
        let mut connection = self.connect().await?;
        
        // Get sender and recipients
        let from_email = message.from_address()
            .ok_or_else(|| Error::Custom("Missing From header".to_string()))?;
        
        let recipients = message.recipients();
        if recipients.is_empty() {
            return Err(Error::Custom("No recipients specified".to_string()));
        }

        // Send MAIL FROM
        connection.command(&format!("MAIL FROM:<{}>", from_email)).await?;

        // Send RCPT TO for each recipient
        for recipient in &recipients {
            connection.command(&format!("RCPT TO:<{}>", recipient)).await?;
        }

        // Send DATA
        connection.command("DATA").await?;

        // Send message content
        let content = message.format()?;
        connection.write_data(&content).await?;

        Ok(())
    }

    async fn connect(&self) -> Result<SmtpConnection> {
        let addr = format!("{}:{}", self.transport.host(), self.transport.port());
        
        let stream = if let Some(timeout) = self.timeout {
            tokio::time::timeout(timeout, TcpStream::connect(&addr))
                .await
                .map_err(|_| Error::Timeout)?
                .map_err(|e| Error::Connection(e.to_string()))?
        } else {
            TcpStream::connect(&addr)
                .await
                .map_err(|e| Error::Connection(e.to_string()))?
        };

        let mut connection = if self.transport.is_tls() {
            // Direct TLS connection
            let tls_config = self.transport.tls_config()
                .ok_or_else(|| Error::Tls("TLS config not set".to_string()))?;
            
            let connector = TlsConnector::from(tls_config.rustls_config());
            let server_name = ServerName::try_from(self.transport.host())
                .map_err(|e| Error::Tls(format!("Invalid server name: {}", e)))?;
            
            let tls_stream = connector.connect(server_name.to_owned(), stream)
                .await
                .map_err(|e| Error::Tls(e.to_string()))?;
            
            SmtpConnection::Tls(BufReader::new(tls_stream))
        } else {
            SmtpConnection::Plain(BufReader::new(stream))
        };

        // Read greeting
        connection.read_response(220).await?;

        // Send EHLO
        let hostname = "localhost";
        let ehlo_response = connection.command(&format!("EHLO {}", hostname)).await?;

        // Handle STARTTLS if needed
        if self.transport.is_starttls() && !self.transport.is_tls() {
            if ehlo_response.contains("STARTTLS") {
                connection.command("STARTTLS").await?;
                
                let tls_config = self.transport.tls_config()
                    .ok_or_else(|| Error::Tls("TLS config not set".to_string()))?;
                
                connection = connection.upgrade_to_tls(
                    tls_config,
                    self.transport.host()
                ).await?;

                // Send EHLO again after STARTTLS
                let _ = connection.command(&format!("EHLO {}", hostname)).await?;
            }
        }

        // Authenticate if credentials are provided
        if let Some(ref creds) = self.credentials {
            let mechanisms = AuthMechanism::from_ehlo_response(&ehlo_response);
            let mechanism = mechanisms.first()
                .ok_or_else(|| Error::Authentication("No supported auth mechanism".to_string()))?;
            
            connection.authenticate(creds, *mechanism).await?;
        }

        Ok(connection)
    }
}

/// SMTP connection (either plain or TLS)
enum SmtpConnection {
    Plain(BufReader<TcpStream>),
    Tls(BufReader<tokio_rustls::client::TlsStream<TcpStream>>),
}

impl SmtpConnection {
    async fn read_response(&mut self, expected_code: u16) -> Result<String> {
        let mut response = String::new();
        
        loop {
            let mut line = String::new();
            let bytes_read = match self {
                Self::Plain(reader) => reader.read_line(&mut line).await?,
                Self::Tls(reader) => reader.read_line(&mut line).await?,
            };

            if bytes_read == 0 {
                return Err(Error::Connection("Connection closed".to_string()));
            }

            response.push_str(&line);

            // Check if this is the last line (starts with code and space)
            if line.len() >= 4 && &line[3..4] == " " {
                break;
            }
        }

        // Parse response code
        let code: u16 = response[0..3]
            .parse()
            .map_err(|_| Error::InvalidResponse(response.clone()))?;

        if code != expected_code && expected_code / 100 != code / 100 {
            return Err(Error::Command(response));
        }

        Ok(response)
    }

    async fn write_line(&mut self, data: &str) -> Result<()> {
        let line = format!("{}\r\n", data);
        match self {
            Self::Plain(reader) => {
                reader.get_mut().write_all(line.as_bytes()).await?;
                reader.get_mut().flush().await?;
            }
            Self::Tls(reader) => {
                reader.get_mut().write_all(line.as_bytes()).await?;
                reader.get_mut().flush().await?;
            }
        }
        Ok(())
    }

    async fn command(&mut self, cmd: &str) -> Result<String> {
        self.write_line(cmd).await?;
        self.read_response(250).await
    }

    async fn write_data(&mut self, data: &[u8]) -> Result<()> {
        // Write message data
        match self {
            Self::Plain(reader) => {
                reader.get_mut().write_all(data).await?;
                reader.get_mut().write_all(b"\r\n.\r\n").await?;
                reader.get_mut().flush().await?;
            }
            Self::Tls(reader) => {
                reader.get_mut().write_all(data).await?;
                reader.get_mut().write_all(b"\r\n.\r\n").await?;
                reader.get_mut().flush().await?;
            }
        }
        
        self.read_response(250).await?;
        Ok(())
    }

    async fn authenticate(&mut self, creds: &Credentials, mechanism: AuthMechanism) -> Result<()> {
        match mechanism {
            AuthMechanism::Plain => {
                let encoded = mechanism.encode(creds);
                self.write_line(&format!("AUTH PLAIN {}", encoded[0])).await?;
                self.read_response(235).await?;
            }
            AuthMechanism::Login => {
                let encoded = mechanism.encode(creds);
                self.write_line("AUTH LOGIN").await?;
                self.read_response(334).await?;
                
                self.write_line(&encoded[0]).await?;
                self.read_response(334).await?;
                
                self.write_line(&encoded[1]).await?;
                self.read_response(235).await?;
            }
            AuthMechanism::None => {}
        }
        Ok(())
    }

    async fn upgrade_to_tls(self, tls_config: &TlsConfig, host: &str) -> Result<Self> {
        let stream = match self {
            Self::Plain(reader) => reader.into_inner(),
            Self::Tls(_) => return Err(Error::Tls("Already using TLS".to_string())),
        };

        let connector = TlsConnector::from(tls_config.rustls_config());
        let server_name = ServerName::try_from(host)
            .map_err(|e| Error::Tls(format!("Invalid server name: {}", e)))?;
        
        let tls_stream = connector.connect(server_name.to_owned(), stream)
            .await
            .map_err(|e| Error::Tls(e.to_string()))?;

        Ok(Self::Tls(BufReader::new(tls_stream)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::SmtpTransport;

    #[test]
    fn test_client_creation() {
        let transport = SmtpTransport::new("smtp.test.com", 587);
        let _client = SmtpClient::new(transport);
        // Just test that we can create a client
    }
}
