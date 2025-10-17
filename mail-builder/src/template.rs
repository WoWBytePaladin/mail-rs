use std::collections::HashMap;
use std::fmt;
use mail_core::{Message, Address};

/// Template engine for creating dynamic email content
#[derive(Debug, Clone)]
pub struct EmailTemplate {
    from_template: Option<String>,
    to_template: Vec<String>,
    cc_template: Vec<String>,
    bcc_template: Vec<String>,
    subject_template: String,
    text_body_template: Option<String>,
    html_body_template: Option<String>,
    variables: HashMap<String, String>,
}

/// Context for template rendering with variables
#[derive(Debug, Clone)]
pub struct TemplateContext {
    variables: HashMap<String, String>,
}

impl TemplateContext {
    /// Create a new template context
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    /// Add a variable to the context
    pub fn set<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.variables.insert(key.into(), value.into());
        self
    }

    /// Add multiple variables to the context
    pub fn set_many<K: Into<String>, V: Into<String>>(mut self, vars: Vec<(K, V)>) -> Self {
        for (key, value) in vars {
            self.variables.insert(key.into(), value.into());
        }
        self
    }

    /// Get a variable from the context
    pub fn get(&self, key: &str) -> Option<&String> {
        self.variables.get(key)
    }

    /// Check if a variable exists in the context
    pub fn contains(&self, key: &str) -> bool {
        self.variables.contains_key(key)
    }

    /// Get all variables
    pub fn variables(&self) -> &HashMap<String, String> {
        &self.variables
    }
}

impl Default for TemplateContext {
    fn default() -> Self {
        Self::new()
    }
}

impl EmailTemplate {
    /// Create a new email template
    pub fn new() -> Self {
        Self {
            from_template: None,
            to_template: Vec::new(),
            cc_template: Vec::new(),
            bcc_template: Vec::new(),
            subject_template: String::new(),
            text_body_template: None,
            html_body_template: None,
            variables: HashMap::new(),
        }
    }

    /// Set the from address template
    pub fn from<S: Into<String>>(mut self, from: S) -> Self {
        self.from_template = Some(from.into());
        self
    }

    /// Add a to address template
    pub fn to<S: Into<String>>(mut self, to: S) -> Self {
        self.to_template.push(to.into());
        self
    }

    /// Add multiple to address templates
    pub fn to_many<S: Into<String>>(mut self, addresses: Vec<S>) -> Self {
        for addr in addresses {
            self.to_template.push(addr.into());
        }
        self
    }

    /// Add a cc address template
    pub fn cc<S: Into<String>>(mut self, cc: S) -> Self {
        self.cc_template.push(cc.into());
        self
    }

    /// Add a bcc address template
    pub fn bcc<S: Into<String>>(mut self, bcc: S) -> Self {
        self.bcc_template.push(bcc.into());
        self
    }

    /// Set the subject template
    pub fn subject<S: Into<String>>(mut self, subject: S) -> Self {
        self.subject_template = subject.into();
        self
    }

    /// Set the text body template
    pub fn text_body<S: Into<String>>(mut self, body: S) -> Self {
        self.text_body_template = Some(body.into());
        self
    }

    /// Set the HTML body template
    pub fn html_body<S: Into<String>>(mut self, body: S) -> Self {
        self.html_body_template = Some(body.into());
        self
    }

    /// Add a default variable that will be used if not provided in context
    pub fn variable<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.variables.insert(key.into(), value.into());
        self
    }

    /// Render the template with the given context
    pub fn render(&self, context: &TemplateContext) -> Result<Message, TemplateError> {
        // Merge default variables with context variables (context takes precedence)
        let mut merged_vars = self.variables.clone();
        for (key, value) in &context.variables {
            merged_vars.insert(key.clone(), value.clone());
        }

        let mut message = Message::new();

        // Render from address
        if let Some(ref from_template) = self.from_template {
            let from_addr = render_template(from_template, &merged_vars)?;
            message = message.from(parse_address(&from_addr)?);
        }

        // Render to addresses
        let mut to_addresses = Vec::new();
        for to_template in &self.to_template {
            let to_addr = render_template(to_template, &merged_vars)?;
            to_addresses.push(parse_address(&to_addr)?);
        }
        if !to_addresses.is_empty() {
            message = message.to(to_addresses);
        }

        // Render cc addresses
        let mut cc_addresses = Vec::new();
        for cc_template in &self.cc_template {
            let cc_addr = render_template(cc_template, &merged_vars)?;
            cc_addresses.push(parse_address(&cc_addr)?);
        }
        if !cc_addresses.is_empty() {
            message = message.cc(cc_addresses);
        }

        // Render bcc addresses
        let mut bcc_addresses = Vec::new();
        for bcc_template in &self.bcc_template {
            let bcc_addr = render_template(bcc_template, &merged_vars)?;
            bcc_addresses.push(parse_address(&bcc_addr)?);
        }
        if !bcc_addresses.is_empty() {
            message = message.bcc(bcc_addresses);
        }

        // Render subject
        let subject = render_template(&self.subject_template, &merged_vars)?;
        message = message.subject(subject);

        // Render text body
        if let Some(ref text_template) = self.text_body_template {
            let text_body = render_template(text_template, &merged_vars)?;
            message = message.body("text/plain", text_body);
        }

        // Render HTML body
        if let Some(ref html_template) = self.html_body_template {
            let html_body = render_template(html_template, &merged_vars)?;
            if self.text_body_template.is_some() {
                message = message.alternative("text/html", html_body);
            } else {
                message = message.body("text/html", html_body);
            }
        }

        Ok(message)
    }

    /// Get a list of all variables used in this template
    pub fn get_variables(&self) -> Vec<String> {
        let mut variables = std::collections::HashSet::new();

        // Extract variables from all template fields
        if let Some(ref from) = self.from_template {
            extract_variables(from, &mut variables);
        }

        for to in &self.to_template {
            extract_variables(to, &mut variables);
        }

        for cc in &self.cc_template {
            extract_variables(cc, &mut variables);
        }

        for bcc in &self.bcc_template {
            extract_variables(bcc, &mut variables);
        }

        extract_variables(&self.subject_template, &mut variables);

        if let Some(ref text) = self.text_body_template {
            extract_variables(text, &mut variables);
        }

        if let Some(ref html) = self.html_body_template {
            extract_variables(html, &mut variables);
        }

        variables.into_iter().collect()
    }
}

impl Default for EmailTemplate {
    fn default() -> Self {
        Self::new()
    }
}

/// Template rendering errors
#[derive(Debug, Clone)]
pub enum TemplateError {
    MissingVariable(String),
    InvalidAddress(String),
    RenderError(String),
}

impl fmt::Display for TemplateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TemplateError::MissingVariable(var) => write!(f, "Missing template variable: {}", var),
            TemplateError::InvalidAddress(addr) => write!(f, "Invalid email address: {}", addr),
            TemplateError::RenderError(msg) => write!(f, "Template render error: {}", msg),
        }
    }
}

impl std::error::Error for TemplateError {}

/// Render a template string by replacing variables
fn render_template(template: &str, variables: &HashMap<String, String>) -> Result<String, TemplateError> {
    let mut result = template.to_string();
    
    // Find all {{variable}} patterns and replace them
    let mut start = 0;
    while let Some(open_pos) = result[start..].find("{{") {
        let absolute_open = start + open_pos;
        if let Some(close_pos) = result[absolute_open + 2..].find("}}") {
            let absolute_close = absolute_open + 2 + close_pos;
            let var_name = result[absolute_open + 2..absolute_close].trim();
            
            if let Some(value) = variables.get(var_name) {
                result.replace_range(absolute_open..absolute_close + 2, value);
                start = absolute_open + value.len();
            } else {
                return Err(TemplateError::MissingVariable(var_name.to_string()));
            }
        } else {
            return Err(TemplateError::RenderError("Unclosed template variable".to_string()));
        }
    }
    
    Ok(result)
}

/// Parse an email address string
fn parse_address(addr_str: &str) -> Result<Address, TemplateError> {
    // Simple email address parsing - could be enhanced
    if addr_str.contains('<') && addr_str.contains('>') {
        // Format: "Name <email@domain.com>"
        let parts: Vec<&str> = addr_str.splitn(2, '<').collect();
        if parts.len() == 2 {
            let name = parts[0].trim().trim_matches('"');
            let email = parts[1].trim_end_matches('>').trim();
            if is_valid_email(email) {
                Ok(Address::with_name(email, name))
            } else {
                Err(TemplateError::InvalidAddress(addr_str.to_string()))
            }
        } else {
            Err(TemplateError::InvalidAddress(addr_str.to_string()))
        }
    } else {
        // Simple email format
        let email = addr_str.trim();
        if is_valid_email(email) {
            Ok(Address::new(email))
        } else {
            Err(TemplateError::InvalidAddress(addr_str.to_string()))
        }
    }
}

/// Basic email validation
fn is_valid_email(email: &str) -> bool {
    email.contains('@') && email.contains('.') && email.len() > 5
}

/// Extract variable names from a template string
fn extract_variables(template: &str, variables: &mut std::collections::HashSet<String>) {
    let mut start = 0;
    while let Some(open_pos) = template[start..].find("{{") {
        let absolute_open = start + open_pos;
        if let Some(close_pos) = template[absolute_open + 2..].find("}}") {
            let absolute_close = absolute_open + 2 + close_pos;
            let var_name = template[absolute_open + 2..absolute_close].trim();
            variables.insert(var_name.to_string());
            start = absolute_close + 2;
        } else {
            break;
        }
    }
}

/// Predefined email templates for common use cases
pub struct CommonTemplates;

impl CommonTemplates {
    /// Welcome email template
    pub fn welcome() -> EmailTemplate {
        EmailTemplate::new()
            .subject("Welcome to {{company_name}}, {{user_name}}!")
            .text_body(
                "Hi {{user_name}},\n\n\
                Welcome to {{company_name}}! We're excited to have you on board.\n\n\
                Your account has been created with the email address: {{user_email}}\n\n\
                To get started, please visit: {{app_url}}\n\n\
                If you have any questions, feel free to contact us at {{support_email}}.\n\n\
                Best regards,\n\
                The {{company_name}} Team"
            )
            .html_body(
                "<h1>Welcome to {{company_name}}, {{user_name}}!</h1>\
                <p>We're excited to have you on board.</p>\
                <p>Your account has been created with the email address: <strong>{{user_email}}</strong></p>\
                <p><a href=\"{{app_url}}\">Get Started</a></p>\
                <p>If you have any questions, feel free to contact us at \
                <a href=\"mailto:{{support_email}}\">{{support_email}}</a>.</p>\
                <p>Best regards,<br>The {{company_name}} Team</p>"
            )
    }

    /// Password reset template
    pub fn password_reset() -> EmailTemplate {
        EmailTemplate::new()
            .subject("Reset your {{company_name}} password")
            .text_body(
                "Hi {{user_name}},\n\n\
                We received a request to reset your password for your {{company_name}} account.\n\n\
                To reset your password, click the following link:\n\
                {{reset_url}}\n\n\
                This link will expire in {{expiry_hours}} hours.\n\n\
                If you didn't request this password reset, please ignore this email.\n\n\
                Best regards,\n\
                The {{company_name}} Team"
            )
            .html_body(
                "<h1>Reset your {{company_name}} password</h1>\
                <p>Hi {{user_name}},</p>\
                <p>We received a request to reset your password for your {{company_name}} account.</p>\
                <p><a href=\"{{reset_url}}\" style=\"background-color: #007bff; color: white; padding: 10px 20px; text-decoration: none; border-radius: 5px;\">Reset Password</a></p>\
                <p>This link will expire in {{expiry_hours}} hours.</p>\
                <p>If you didn't request this password reset, please ignore this email.</p>\
                <p>Best regards,<br>The {{company_name}} Team</p>"
            )
    }

    /// Newsletter template
    pub fn newsletter() -> EmailTemplate {
        EmailTemplate::new()
            .subject("{{newsletter_title}} - {{company_name}}")
            .text_body(
                "{{newsletter_title}}\n\n\
                {{newsletter_content}}\n\n\
                ---\n\
                You're receiving this because you subscribed to {{company_name}} newsletter.\n\
                To unsubscribe, visit: {{unsubscribe_url}}"
            )
            .html_body(
                "<h1>{{newsletter_title}}</h1>\
                <div>{{newsletter_content}}</div>\
                <hr>\
                <p><small>You're receiving this because you subscribed to {{company_name}} newsletter.<br>\
                <a href=\"{{unsubscribe_url}}\">Unsubscribe</a></small></p>"
            )
    }

    /// Order confirmation template
    pub fn order_confirmation() -> EmailTemplate {
        EmailTemplate::new()
            .subject("Order Confirmation #{{order_number}} - {{company_name}}")
            .text_body(
                "Hi {{customer_name}},\n\n\
                Thank you for your order! Here are your order details:\n\n\
                Order Number: {{order_number}}\n\
                Order Date: {{order_date}}\n\
                Total Amount: {{order_total}}\n\n\
                Items:\n\
                {{order_items}}\n\n\
                Shipping Address:\n\
                {{shipping_address}}\n\n\
                Your order will be processed within {{processing_time}} business days.\n\n\
                Track your order: {{tracking_url}}\n\n\
                Thank you for choosing {{company_name}}!\n\n\
                Best regards,\n\
                The {{company_name}} Team"
            )
            .html_body(
                "<h1>Order Confirmation #{{order_number}}</h1>\
                <p>Hi {{customer_name}},</p>\
                <p>Thank you for your order! Here are your order details:</p>\
                <table style=\"border-collapse: collapse; width: 100%;\">\
                <tr><td style=\"border: 1px solid #ddd; padding: 8px;\"><strong>Order Number:</strong></td><td style=\"border: 1px solid #ddd; padding: 8px;\">{{order_number}}</td></tr>\
                <tr><td style=\"border: 1px solid #ddd; padding: 8px;\"><strong>Order Date:</strong></td><td style=\"border: 1px solid #ddd; padding: 8px;\">{{order_date}}</td></tr>\
                <tr><td style=\"border: 1px solid #ddd; padding: 8px;\"><strong>Total Amount:</strong></td><td style=\"border: 1px solid #ddd; padding: 8px;\">{{order_total}}</td></tr>\
                </table>\
                <h3>Items:</h3>\
                <div>{{order_items}}</div>\
                <h3>Shipping Address:</h3>\
                <div>{{shipping_address}}</div>\
                <p>Your order will be processed within {{processing_time}} business days.</p>\
                <p><a href=\"{{tracking_url}}\">Track your order</a></p>\
                <p>Thank you for choosing {{company_name}}!</p>\
                <p>Best regards,<br>The {{company_name}} Team</p>"
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_context() {
        let context = TemplateContext::new()
            .set("name", "John")
            .set("company", "Acme Corp");
        
        assert_eq!(context.get("name"), Some(&"John".to_string()));
        assert_eq!(context.get("company"), Some(&"Acme Corp".to_string()));
        assert!(context.contains("name"));
        assert!(!context.contains("missing"));
    }

    #[test]
    fn test_render_template() {
        let vars = vec![
            ("name".to_string(), "John".to_string()),
            ("company".to_string(), "Acme Corp".to_string()),
        ].into_iter().collect();

        let template = "Hello {{name}}, welcome to {{company}}!";
        let result = render_template(template, &vars).unwrap();
        assert_eq!(result, "Hello John, welcome to Acme Corp!");
    }

    #[test]
    fn test_render_template_missing_variable() {
        let vars = HashMap::new();
        let template = "Hello {{name}}!";
        let result = render_template(template, &vars);
        assert!(matches!(result, Err(TemplateError::MissingVariable(_))));
    }

    #[test]
    fn test_extract_variables() {
        let mut variables = std::collections::HashSet::new();
        extract_variables("Hello {{name}}, your order {{order_id}} is ready!", &mut variables);
        
        assert!(variables.contains("name"));
        assert!(variables.contains("order_id"));
        assert_eq!(variables.len(), 2);
    }

    #[test]
    fn test_parse_address() {
        let addr1 = parse_address("test@example.com").unwrap();
        assert_eq!(addr1.email, "test@example.com");

        let addr2 = parse_address("\"John Doe\" <john@example.com>").unwrap();
        assert_eq!(addr2.email, "john@example.com");
        assert_eq!(addr2.name, Some("John Doe".to_string()));
    }

    #[test]
    fn test_email_template_render() {
        let template = EmailTemplate::new()
            .from("noreply@{{company_domain}}")
            .to("{{user_email}}")
            .subject("Welcome {{user_name}}")
            .text_body("Hello {{user_name}}, welcome to {{company_name}}!");

        let context = TemplateContext::new()
            .set("company_domain", "example.com")
            .set("user_email", "john@test.com")
            .set("user_name", "John")
            .set("company_name", "Test Corp");

        let message = template.render(&context).unwrap();
        
        assert_eq!(message.from_address().unwrap(), "noreply@example.com");
        assert_eq!(message.get_header().get_first("Subject").unwrap(), "Welcome John");
    }

    #[test]
    fn test_common_templates() {
        let welcome_template = CommonTemplates::welcome();
        let variables = welcome_template.get_variables();
        
        assert!(variables.contains(&"company_name".to_string()));
        assert!(variables.contains(&"user_name".to_string()));
        assert!(variables.contains(&"user_email".to_string()));
    }
}