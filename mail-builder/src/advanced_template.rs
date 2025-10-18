use std::collections::HashMap;
use std::fmt;
use mail_core::{Message, Address};
use serde::{Serialize, Deserialize};
use serde_json::Value;

/// Enhanced template engine with advanced features
#[derive(Debug)]
pub struct AdvancedTemplateEngine {
    /// Template cache for partials
    partials: HashMap<String, String>,
    /// Custom helper functions
    helpers: HashMap<String, Box<dyn TemplateHelper>>,
    /// Template options
    options: TemplateOptions,
}

/// Template rendering options
#[derive(Debug, Clone)]
pub struct TemplateOptions {
    /// Whether to escape HTML by default
    pub auto_escape: bool,
    /// Whether to allow missing variables
    pub allow_missing_vars: bool,
    /// Default value for missing variables
    pub missing_var_default: String,
    /// Whether to trim whitespace
    pub trim_whitespace: bool,
}

impl Default for TemplateOptions {
    fn default() -> Self {
        Self {
            auto_escape: true,
            allow_missing_vars: false,
            missing_var_default: "".to_string(),
            trim_whitespace: true,
        }
    }
}

/// Enhanced template context with support for complex data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedTemplateContext {
    /// Template variables as JSON values
    pub data: Value,
}

impl AdvancedTemplateContext {
    /// Create a new empty context
    pub fn new() -> Self {
        Self {
            data: Value::Object(serde_json::Map::new()),
        }
    }

    /// Create context from JSON value
    pub fn from_json(data: Value) -> Self {
        Self { data }
    }

    /// Set a variable in the context
    pub fn set<K: Into<String>>(mut self, key: K, value: Value) -> Self {
        if let Value::Object(ref mut map) = self.data {
            map.insert(key.into(), value);
        }
        self
    }

    /// Set a string variable
    pub fn set_string<K: Into<String>, V: Into<String>>(self, key: K, value: V) -> Self {
        self.set(key, Value::String(value.into()))
    }

    /// Set a number variable
    pub fn set_number<K: Into<String>>(self, key: K, value: f64) -> Self {
        let number = serde_json::Number::from_f64(value).unwrap_or_else(|| serde_json::Number::from(0));
        self.set(key, Value::Number(number))
    }

    /// Set a boolean variable
    pub fn set_bool<K: Into<String>>(self, key: K, value: bool) -> Self {
        self.set(key, Value::Bool(value))
    }

    /// Set an array variable
    pub fn set_array<K: Into<String>>(self, key: K, value: Vec<Value>) -> Self {
        self.set(key, Value::Array(value))
    }

    /// Set an object variable
    pub fn set_object<K: Into<String>>(self, key: K, value: serde_json::Map<String, Value>) -> Self {
        self.set(key, Value::Object(value))
    }

    /// Get a variable from the context
    pub fn get(&self, path: &str) -> Option<&Value> {
        get_nested_value(&self.data, path)
    }

    /// Check if a variable exists
    pub fn contains(&self, path: &str) -> bool {
        self.get(path).is_some()
    }
}

impl Default for AdvancedTemplateContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for custom template helpers
pub trait TemplateHelper: fmt::Debug + Send + Sync {
    /// Execute the helper with given arguments
    fn execute(&self, args: &[Value], context: &AdvancedTemplateContext) -> Result<String, TemplateError>;

    /// Get helper name
    fn name(&self) -> &str;

    /// Get helper description
    fn description(&self) -> &str {
        "Custom helper"
    }
}

/// Template error types
#[derive(Debug, Clone)]
pub enum TemplateError {
    /// Variable not found
    VariableNotFound(String),
    /// Invalid syntax
    InvalidSyntax(String),
    /// Helper error
    HelperError(String),
    /// Partial not found
    PartialNotFound(String),
    /// Invalid condition
    InvalidCondition(String),
    /// Loop error
    LoopError(String),
    /// General error
    General(String),
}

impl fmt::Display for TemplateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TemplateError::VariableNotFound(var) => write!(f, "Variable not found: {}", var),
            TemplateError::InvalidSyntax(msg) => write!(f, "Invalid syntax: {}", msg),
            TemplateError::HelperError(msg) => write!(f, "Helper error: {}", msg),
            TemplateError::PartialNotFound(name) => write!(f, "Partial not found: {}", name),
            TemplateError::InvalidCondition(msg) => write!(f, "Invalid condition: {}", msg),
            TemplateError::LoopError(msg) => write!(f, "Loop error: {}", msg),
            TemplateError::General(msg) => write!(f, "Template error: {}", msg),
        }
    }
}

impl std::error::Error for TemplateError {}

impl AdvancedTemplateEngine {
    /// Create a new template engine
    pub fn new() -> Self {
        let mut engine = Self {
            partials: HashMap::new(),
            helpers: HashMap::new(),
            options: TemplateOptions::default(),
        };

        // Register built-in helpers
        engine.register_builtin_helpers();
        engine
    }

    /// Create template engine with custom options
    pub fn with_options(options: TemplateOptions) -> Self {
        let mut engine = Self {
            partials: HashMap::new(),
            helpers: HashMap::new(),
            options,
        };

        engine.register_builtin_helpers();
        engine
    }

    /// Register a partial template
    pub fn register_partial<K: Into<String>, V: Into<String>>(&mut self, name: K, template: V) {
        self.partials.insert(name.into(), template.into());
    }

    /// Register a custom helper
    pub fn register_helper(&mut self, helper: Box<dyn TemplateHelper>) {
        let name = helper.name().to_string();
        self.helpers.insert(name, helper);
    }

    /// Render a template with context
    pub fn render(&self, template: &str, context: &AdvancedTemplateContext) -> Result<String, TemplateError> {
        let mut renderer = TemplateRenderer::new(self, context);
        renderer.render(template)
    }

    /// Render a template to an email message
    pub fn render_email(
        &self,
        email_template: &EmailTemplateDefinition,
        context: &AdvancedTemplateContext,
    ) -> Result<Message, TemplateError> {
        let mut message = Message::new();

        // Render from address
        if let Some(ref from_template) = email_template.from {
            let from_rendered = self.render(from_template, context)?;
            message = message.from(Address::new(from_rendered));
        }

        // Render to addresses
        let mut to_addresses = Vec::new();
        for to_template in &email_template.to {
            let to_rendered = self.render(to_template, context)?;
            to_addresses.push(Address::new(to_rendered));
        }
        if !to_addresses.is_empty() {
            message = message.to(to_addresses);
        }

        // Render CC addresses
        for cc_template in &email_template.cc {
            let cc_rendered = self.render(cc_template, context)?;
            // Note: Current Message API doesn't have cc method, would need to add
            message = message.header("Cc", cc_rendered);
        }

        // Render BCC addresses
        for bcc_template in &email_template.bcc {
            let bcc_rendered = self.render(bcc_template, context)?;
            message = message.header("Bcc", bcc_rendered);
        }

        // Render subject
        let subject_rendered = self.render(&email_template.subject, context)?;
        message = message.subject(subject_rendered);

        // Render text body
        if let Some(ref text_template) = email_template.text_body {
            let text_rendered = self.render(text_template, context)?;
            message = message.body("text/plain", text_rendered);
        }

        // Render HTML body
        if let Some(ref html_template) = email_template.html_body {
            let html_rendered = self.render(html_template, context)?;
            if email_template.text_body.is_some() {
                message = message.alternative("text/html", html_rendered);
            } else {
                message = message.body("text/html", html_rendered);
            }
        }

        Ok(message)
    }

    fn register_builtin_helpers(&mut self) {
        // Register built-in helpers
        self.register_helper(Box::new(UppercaseHelper));
        self.register_helper(Box::new(LowercaseHelper));
        self.register_helper(Box::new(CapitalizeHelper));
        self.register_helper(Box::new(FormatDateHelper));
        self.register_helper(Box::new(JoinHelper));
        self.register_helper(Box::new(DefaultHelper));
        self.register_helper(Box::new(LengthHelper));
        self.register_helper(Box::new(TruncateHelper));
    }
}

/// Email template definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailTemplateDefinition {
    pub from: Option<String>,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub subject: String,
    pub text_body: Option<String>,
    pub html_body: Option<String>,
}

impl EmailTemplateDefinition {
    pub fn new() -> Self {
        Self {
            from: None,
            to: Vec::new(),
            cc: Vec::new(),
            bcc: Vec::new(),
            subject: String::new(),
            text_body: None,
            html_body: None,
        }
    }

    pub fn from<S: Into<String>>(mut self, from: S) -> Self {
        self.from = Some(from.into());
        self
    }

    pub fn to<S: Into<String>>(mut self, to: S) -> Self {
        self.to.push(to.into());
        self
    }

    pub fn subject<S: Into<String>>(mut self, subject: S) -> Self {
        self.subject = subject.into();
        self
    }

    pub fn text_body<S: Into<String>>(mut self, body: S) -> Self {
        self.text_body = Some(body.into());
        self
    }

    pub fn html_body<S: Into<String>>(mut self, body: S) -> Self {
        self.html_body = Some(body.into());
        self
    }
}

/// Template renderer for processing template syntax
struct TemplateRenderer<'a> {
    engine: &'a AdvancedTemplateEngine,
    context: &'a AdvancedTemplateContext,
}

impl<'a> TemplateRenderer<'a> {
    fn new(engine: &'a AdvancedTemplateEngine, context: &'a AdvancedTemplateContext) -> Self {
        Self { engine, context }
    }

    fn render(&mut self, template: &str) -> Result<String, TemplateError> {
        let mut result = String::new();
        let mut chars = template.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' && chars.peek() == Some(&'{') {
                chars.next(); // consume second '{'
                
                // Look for closing }}
                let mut expression = String::new();
                let mut brace_count = 0;
                
                while let Some(ch) = chars.next() {
                    if ch == '}' && chars.peek() == Some(&'}') && brace_count == 0 {
                        chars.next(); // consume second '}'
                        break;
                    } else if ch == '{' && chars.peek() == Some(&'{') {
                        brace_count += 1;
                        expression.push(ch);
                    } else if ch == '}' && chars.peek() == Some(&'}') {
                        brace_count -= 1;
                        expression.push(ch);
                    } else {
                        expression.push(ch);
                    }
                }

                let rendered = self.process_expression(&expression.trim())?;
                result.push_str(&rendered);
            } else {
                result.push(ch);
            }
        }

        Ok(result)
    }

    fn process_expression(&mut self, expr: &str) -> Result<String, TemplateError> {
        let expr = expr.trim();

        // Handle different expression types
        if expr.starts_with("#if ") {
            self.process_conditional(expr)
        } else if expr.starts_with("#each ") {
            self.process_loop(expr)
        } else if expr.starts_with("> ") {
            self.process_partial(expr)
        } else if expr.contains('(') && expr.contains(')') {
            self.process_helper_call(expr)
        } else {
            self.process_variable(expr)
        }
    }

    fn process_variable(&self, var_path: &str) -> Result<String, TemplateError> {
        match self.context.get(var_path) {
            Some(value) => Ok(value_to_string(value)),
            None => {
                if self.engine.options.allow_missing_vars {
                    Ok(self.engine.options.missing_var_default.clone())
                } else {
                    Err(TemplateError::VariableNotFound(var_path.to_string()))
                }
            }
        }
    }

    fn process_conditional(&self, _expr: &str) -> Result<String, TemplateError> {
        // Simplified conditional processing (would need full parser in real implementation)
        Ok("<!-- Conditional rendering not fully implemented in this demo -->".to_string())
    }

    fn process_loop(&self, _expr: &str) -> Result<String, TemplateError> {
        // Simplified loop processing (would need full parser in real implementation)
        Ok("<!-- Loop rendering not fully implemented in this demo -->".to_string())
    }

    fn process_partial(&mut self, expr: &str) -> Result<String, TemplateError> {
        let partial_name = expr.strip_prefix("> ").unwrap_or("").trim();
        match self.engine.partials.get(partial_name) {
            Some(partial_template) => self.render(partial_template),
            None => Err(TemplateError::PartialNotFound(partial_name.to_string())),
        }
    }

    fn process_helper_call(&self, expr: &str) -> Result<String, TemplateError> {
        // Simple helper call parsing (would need proper parser in real implementation)
        if let Some(paren_pos) = expr.find('(') {
            let helper_name = expr[..paren_pos].trim();
            if let Some(helper) = self.engine.helpers.get(helper_name) {
                // For demo, just call with empty args
                helper.execute(&[], self.context)
            } else {
                Err(TemplateError::HelperError(format!("Helper not found: {}", helper_name)))
            }
        } else {
            Err(TemplateError::InvalidSyntax("Invalid helper call".to_string()))
        }
    }
}

// Helper utility functions
fn get_nested_value<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = value;

    for part in parts {
        match current {
            Value::Object(map) => {
                current = map.get(part)?;
            }
            Value::Array(arr) => {
                if let Ok(index) = part.parse::<usize>() {
                    current = arr.get(index)?;
                } else {
                    return None;
                }
            }
            _ => return None,
        }
    }

    Some(current)
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Array(arr) => format!("[{}]", arr.len()),
        Value::Object(obj) => format!("{{{}}}", obj.len()),
        Value::Null => "null".to_string(),
    }
}

// Built-in helper implementations
#[derive(Debug)]
struct UppercaseHelper;

impl TemplateHelper for UppercaseHelper {
    fn execute(&self, args: &[Value], _context: &AdvancedTemplateContext) -> Result<String, TemplateError> {
        if let Some(arg) = args.first() {
            Ok(value_to_string(arg).to_uppercase())
        } else {
            Ok(String::new())
        }
    }

    fn name(&self) -> &str {
        "uppercase"
    }

    fn description(&self) -> &str {
        "Convert text to uppercase"
    }
}

#[derive(Debug)]
struct LowercaseHelper;

impl TemplateHelper for LowercaseHelper {
    fn execute(&self, args: &[Value], _context: &AdvancedTemplateContext) -> Result<String, TemplateError> {
        if let Some(arg) = args.first() {
            Ok(value_to_string(arg).to_lowercase())
        } else {
            Ok(String::new())
        }
    }

    fn name(&self) -> &str {
        "lowercase"
    }

    fn description(&self) -> &str {
        "Convert text to lowercase"
    }
}

#[derive(Debug)]
struct CapitalizeHelper;

impl TemplateHelper for CapitalizeHelper {
    fn execute(&self, args: &[Value], _context: &AdvancedTemplateContext) -> Result<String, TemplateError> {
        if let Some(arg) = args.first() {
            let text = value_to_string(arg);
            let mut chars = text.chars();
            match chars.next() {
                None => Ok(String::new()),
                Some(first) => Ok(first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()),
            }
        } else {
            Ok(String::new())
        }
    }

    fn name(&self) -> &str {
        "capitalize"
    }

    fn description(&self) -> &str {
        "Capitalize first letter of text"
    }
}

#[derive(Debug)]
struct FormatDateHelper;

impl TemplateHelper for FormatDateHelper {
    fn execute(&self, args: &[Value], _context: &AdvancedTemplateContext) -> Result<String, TemplateError> {
        // Simplified date formatting
        if let Some(arg) = args.first() {
            Ok(format!("Formatted: {}", value_to_string(arg)))
        } else {
            Ok(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string())
        }
    }

    fn name(&self) -> &str {
        "format_date"
    }

    fn description(&self) -> &str {
        "Format date with optional format string"
    }
}

#[derive(Debug)]
struct JoinHelper;

impl TemplateHelper for JoinHelper {
    fn execute(&self, args: &[Value], _context: &AdvancedTemplateContext) -> Result<String, TemplateError> {
        if let Some(Value::Array(arr)) = args.first() {
            let separator = args.get(1)
                .map(value_to_string)
                .unwrap_or_else(|| ", ".to_string());
            
            let joined = arr.iter()
                .map(value_to_string)
                .collect::<Vec<_>>()
                .join(&separator);
            
            Ok(joined)
        } else {
            Ok(String::new())
        }
    }

    fn name(&self) -> &str {
        "join"
    }

    fn description(&self) -> &str {
        "Join array elements with separator"
    }
}

#[derive(Debug)]
struct DefaultHelper;

impl TemplateHelper for DefaultHelper {
    fn execute(&self, args: &[Value], _context: &AdvancedTemplateContext) -> Result<String, TemplateError> {
        if let Some(first) = args.first() {
            let value_str = value_to_string(first);
            if value_str.is_empty() || value_str == "null" {
                if let Some(default) = args.get(1) {
                    Ok(value_to_string(default))
                } else {
                    Ok(String::new())
                }
            } else {
                Ok(value_str)
            }
        } else {
            Ok(String::new())
        }
    }

    fn name(&self) -> &str {
        "default"
    }

    fn description(&self) -> &str {
        "Provide default value if variable is empty"
    }
}

#[derive(Debug)]
struct LengthHelper;

impl TemplateHelper for LengthHelper {
    fn execute(&self, args: &[Value], _context: &AdvancedTemplateContext) -> Result<String, TemplateError> {
        if let Some(arg) = args.first() {
            let length = match arg {
                Value::String(s) => s.len(),
                Value::Array(arr) => arr.len(),
                Value::Object(obj) => obj.len(),
                _ => value_to_string(arg).len(),
            };
            Ok(length.to_string())
        } else {
            Ok("0".to_string())
        }
    }

    fn name(&self) -> &str {
        "length"
    }

    fn description(&self) -> &str {
        "Get length of string, array, or object"
    }
}

#[derive(Debug)]
struct TruncateHelper;

impl TemplateHelper for TruncateHelper {
    fn execute(&self, args: &[Value], _context: &AdvancedTemplateContext) -> Result<String, TemplateError> {
        if let Some(text_arg) = args.first() {
            let text = value_to_string(text_arg);
            let max_length = args.get(1)
                .and_then(|v| v.as_u64())
                .unwrap_or(100) as usize;
            
            if text.len() <= max_length {
                Ok(text)
            } else {
                Ok(format!("{}...", &text[..max_length.saturating_sub(3)]))
            }
        } else {
            Ok(String::new())
        }
    }

    fn name(&self) -> &str {
        "truncate"
    }

    fn description(&self) -> &str {
        "Truncate text to specified length"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_advanced_template_context() {
        let context = AdvancedTemplateContext::new()
            .set_string("name", "John Doe")
            .set_number("age", 30.0)
            .set_bool("active", true);

        assert_eq!(context.get("name").unwrap(), &Value::String("John Doe".to_string()));
        assert_eq!(context.get("age").unwrap(), &json!(30.0));
        assert_eq!(context.get("active").unwrap(), &Value::Bool(true));
    }

    #[test]
    fn test_template_engine_creation() {
        let engine = AdvancedTemplateEngine::new();
        assert!(engine.helpers.contains_key("uppercase"));
        assert!(engine.helpers.contains_key("lowercase"));
        assert!(engine.helpers.contains_key("capitalize"));
    }

    #[test]
    fn test_basic_variable_rendering() {
        let engine = AdvancedTemplateEngine::new();
        let context = AdvancedTemplateContext::new()
            .set_string("name", "Alice");

        let template = "Hello {{name}}!";
        let result = engine.render(template, &context).unwrap();
        assert_eq!(result, "Hello Alice!");
    }

    #[test]
    fn test_partial_registration() {
        let mut engine = AdvancedTemplateEngine::new();
        engine.register_partial("greeting", "Hello {{name}}!");

        let context = AdvancedTemplateContext::new()
            .set_string("name", "Bob");

        let template = "{{> greeting}} How are you?";
        let result = engine.render(template, &context).unwrap();
        assert_eq!(result, "Hello Bob! How are you?");
    }

    #[test]
    fn test_email_template_rendering() {
        let engine = AdvancedTemplateEngine::new();
        let context = AdvancedTemplateContext::new()
            .set_string("recipient_name", "John")
            .set_string("sender_email", "sender@example.com")
            .set_string("recipient_email", "john@example.com");

        let email_template = EmailTemplateDefinition::new()
            .from("{{sender_email}}")
            .to("{{recipient_email}}")
            .subject("Welcome {{recipient_name}}!")
            .text_body("Hello {{recipient_name}}, welcome to our service!")
            .html_body("<h1>Hello {{recipient_name}}</h1><p>Welcome to our service!</p>");

        let message = engine.render_email(&email_template, &context).unwrap();
        
        // Note: This test would need proper Message API inspection methods
        // For now, just verify it doesn't error
        assert!(!message.format().unwrap().is_empty());
    }
}