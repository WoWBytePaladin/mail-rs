//! Email template example with dynamic content

use mail_builder::prelude::*;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Template data
    let mut template_data = HashMap::new();
    template_data.insert("user_name", "Alice Johnson");
    template_data.insert("product_name", "Premium Subscription");
    template_data.insert("price", "$19.99");
    template_data.insert("renewal_date", "2024-01-15");

    // HTML template
    let html_template = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <style>
            body { font-family: Arial, sans-serif; }
            .header { background-color: #f0f0f0; padding: 20px; }
            .content { padding: 20px; }
            .footer { background-color: #e0e0e0; padding: 10px; font-size: 12px; }
        </style>
    </head>
    <body>
        <div class="header">
            <h1>Subscription Renewal Notice</h1>
        </div>
        <div class="content">
            <p>Dear {{user_name}},</p>
            <p>Your subscription for <strong>{{product_name}}</strong> will renew on {{renewal_date}}.</p>
            <p>Amount: <strong>{{price}}</strong></p>
            <p>Thank you for your continued subscription!</p>
        </div>
        <div class="footer">
            <p>This is an automated message. Please do not reply.</p>
        </div>
    </body>
    </html>
    "#;

    // Plain text template
    let text_template = r#"
Subscription Renewal Notice

Dear {{user_name}},

Your subscription for {{product_name}} will renew on {{renewal_date}}.
Amount: {{price}}

Thank you for your continued subscription!

---
This is an automated message. Please do not reply.
    "#;

    // Replace template variables
    let html_content = replace_template_vars(html_template, &template_data);
    let text_content = replace_template_vars(text_template, &template_data);

    // Build the email
    let message = MessageBuilder::new()
        .from_with_name("noreply@company.com", "Your Company")
        .to_with_name("alice@example.com", "Alice Johnson")
        .subject("Subscription Renewal Notice")
        .text_body(&text_content)
        .html_body(&html_content)
        .header("X-Mailer", "mail-rs")
        .header("X-Priority", "3")
        .build();

    // Configure SMTP client
    let transport = SmtpTransport::new("smtp.company.com", 587)
        .with_starttls(TlsConfig::new());

    let client = SmtpClient::new(transport)
        .credentials(Credentials::new("username", "password"));

    // Send the email
    match client.send(&message).await {
        Ok(_) => {
            println!("✓ Template email sent successfully to {}", 
                template_data["user_name"]);
        }
        Err(e) => {
            eprintln!("Failed to send email: {}", e);
        }
    }

    Ok(())
}

/// Simple template variable replacement
fn replace_template_vars(template: &str, data: &HashMap<&str, &str>) -> String {
    let mut result = template.to_string();
    
    for (key, value) in data {
        let placeholder = format!("{{{{{}}}}}", key);
        result = result.replace(&placeholder, value);
    }
    
    result
}