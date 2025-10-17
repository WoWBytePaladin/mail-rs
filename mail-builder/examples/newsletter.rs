//! Newsletter sending with error handling and retry logic

use mail_builder::prelude::*;
use std::time::Duration;
use tokio::time::sleep;

struct Subscriber {
    email: String,
    name: String,
    preferences: Vec<String>,
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Subscriber list
    let subscribers = vec![
        Subscriber {
            email: "alice@example.com".to_string(),
            name: "Alice".to_string(),
            preferences: vec!["tech".to_string(), "news".to_string()],
        },
        Subscriber {
            email: "bob@example.com".to_string(),
            name: "Bob".to_string(),
            preferences: vec!["sports".to_string()],
        },
        Subscriber {
            email: "charlie@example.com".to_string(),
            name: "Charlie".to_string(),
            preferences: vec!["tech".to_string()],
        },
    ];

    // Newsletter content
    let newsletter_content = create_newsletter_content();

    // SMTP configuration
    let transport = SmtpTransport::new("smtp.newsletter.com", 587)
        .with_starttls(TlsConfig::new());

    let client = SmtpClient::new(transport)
        .credentials(Credentials::new("newsletter@company.com", "password"))
        .timeout(Duration::from_secs(30));

    let mut success_count = 0;
    let mut failed_count = 0;

    // Send newsletter to each subscriber
    for subscriber in subscribers {
        let personalized_content = personalize_content(&newsletter_content, &subscriber);
        
        let message = MessageBuilder::new()
            .from_with_name("newsletter@company.com", "Tech Newsletter")
            .to_with_name(&subscriber.email, &subscriber.name)
            .subject("Tech Weekly - Latest Updates")
            .html_body(&personalized_content.html)
            .text_body(&personalized_content.text)
            .header("List-Unsubscribe", "<mailto:unsubscribe@company.com>")
            .header("X-Campaign", "tech-weekly-2024")
            .build();

        // Send with retry logic
        match send_with_retry(&client, &message, 3).await {
            Ok(_) => {
                println!("✓ Newsletter sent to {} ({})", subscriber.name, subscriber.email);
                success_count += 1;
            }
            Err(e) => {
                eprintln!("✗ Failed to send to {} ({}): {}", 
                    subscriber.name, subscriber.email, e);
                failed_count += 1;
            }
        }

        // Rate limiting - small delay between sends
        sleep(Duration::from_millis(500)).await;
    }

    // Summary
    println!("\n=== Newsletter Sending Summary ===");
    println!("Successfully sent: {}", success_count);
    println!("Failed to send: {}", failed_count);
    println!("Total subscribers: {}", success_count + failed_count);

    Ok(())
}

struct NewsletterContent {
    html: String,
    text: String,
}

fn create_newsletter_content() -> NewsletterContent {
    let html = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <style>
            body { font-family: Arial, sans-serif; margin: 0; padding: 20px; }
            .header { background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 20px; border-radius: 8px; }
            .content { background: white; padding: 20px; margin: 20px 0; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
            .article { margin-bottom: 30px; }
            .article h2 { color: #333; }
            .footer { background: #f5f5f5; padding: 15px; border-radius: 8px; font-size: 12px; color: #666; }
        </style>
    </head>
    <body>
        <div class="header">
            <h1>🚀 Tech Weekly Newsletter</h1>
            <p>Your weekly dose of technology news and insights</p>
        </div>
        
        <div class="content">
            <div class="article">
                <h2>🔥 This Week's Top Stories</h2>
                <ul>
                    <li><strong>AI Breakthrough:</strong> New language model achieves 98% accuracy</li>
                    <li><strong>Web Development:</strong> New JavaScript framework released</li>
                    <li><strong>Cloud Computing:</strong> Major provider announces price cuts</li>
                </ul>
            </div>
            
            <div class="article">
                <h2>📚 Featured Tutorial</h2>
                <p><strong>Building Scalable APIs with Rust</strong></p>
                <p>Learn how to create high-performance web APIs using Rust and popular frameworks...</p>
            </div>
        </div>
        
        <div class="footer">
            <p>Hello {{name}}! This newsletter was personalized based on your interests: {{preferences}}</p>
            <p>You received this email because you subscribed to our newsletter.</p>
            <p><a href="mailto:unsubscribe@company.com">Unsubscribe</a> | <a href="https://company.com/preferences">Manage Preferences</a></p>
        </div>
    </body>
    </html>
    "#.to_string();

    let text = r#"
TECH WEEKLY NEWSLETTER
======================

🚀 Your weekly dose of technology news and insights

THIS WEEK'S TOP STORIES
------------------------
• AI Breakthrough: New language model achieves 98% accuracy
• Web Development: New JavaScript framework released  
• Cloud Computing: Major provider announces price cuts

FEATURED TUTORIAL
-----------------
Building Scalable APIs with Rust
Learn how to create high-performance web APIs using Rust and popular frameworks...

---
Hello {{name}}! This newsletter was personalized based on your interests: {{preferences}}

You received this email because you subscribed to our newsletter.
To unsubscribe, reply with "UNSUBSCRIBE" or visit: https://company.com/unsubscribe
    "#.to_string();

    NewsletterContent { html, text }
}

fn personalize_content(content: &NewsletterContent, subscriber: &Subscriber) -> NewsletterContent {
    let preferences_str = subscriber.preferences.join(", ");
    
    let html = content.html
        .replace("{{name}}", &subscriber.name)
        .replace("{{preferences}}", &preferences_str);
    
    let text = content.text
        .replace("{{name}}", &subscriber.name)
        .replace("{{preferences}}", &preferences_str);

    NewsletterContent { html, text }
}

async fn send_with_retry(
    client: &SmtpClient,
    message: &Message,
    max_retries: u32,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut last_error = None;
    
    for attempt in 1..=max_retries {
        match client.send(message).await {
            Ok(response) => {
                return Ok(response);
            }
            Err(e) => {
                last_error = Some(e);
                if attempt < max_retries {
                    let delay = Duration::from_secs(2u64.pow(attempt - 1)); // Exponential backoff
                    eprintln!("Attempt {} failed, retrying in {:?}...", attempt, delay);
                    sleep(delay).await;
                }
            }
        }
    }
    
    Err(Box::new(last_error.unwrap()))
}