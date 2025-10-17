//! Email template example with dynamic content using the template engine

use mail_builder::{EmailTemplate, TemplateContext, CommonTemplates};

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("📝 Email Template Engine Example");
    
    // Demonstrate basic template usage
    println!("\n🎯 Basic Template Example");
    demo_basic_template().await?;
    
    // Demonstrate welcome email template
    println!("\n👋 Welcome Email Template");
    demo_welcome_template().await?;
    
    // Demonstrate newsletter template
    println!("\n📰 Newsletter Template");
    demo_newsletter_template().await?;
    
    // Demonstrate order confirmation template
    println!("\n🛒 Order Confirmation Template");
    demo_order_confirmation_template().await?;
    
    // Demonstrate variable extraction
    println!("\n🔍 Template Variable Analysis");
    demo_variable_extraction();
    
    println!("\n🎨 Template Engine Benefits:");
    println!("   ✅ Dynamic content generation with variable substitution");
    println!("   ✅ Reusable templates for consistent branding");
    println!("   ✅ Support for both text and HTML content");
    println!("   ✅ Built-in common templates for typical use cases");
    println!("   ✅ Type-safe template rendering with error handling");
    
    Ok(())
}

async fn demo_basic_template() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Create a simple template
    let template = EmailTemplate::new()
        .from("noreply@{{company_domain}}")
        .to("{{user_email}}")
        .subject("Hello {{user_name}}!")
        .text_body("Dear {{user_name}},\n\nThank you for joining {{company_name}}!\n\nBest regards,\nThe Team")
        .html_body("<h1>Hello {{user_name}}!</h1><p>Thank you for joining <strong>{{company_name}}</strong>!</p><p>Best regards,<br>The Team</p>");
    
    // Create context with variables
    let context = TemplateContext::new()
        .set("company_domain", "example.com")
        .set("user_email", "john.doe@test.com")
        .set("user_name", "John Doe")
        .set("company_name", "Tech Corp");
    
    // Render the template
    let message = template.render(&context)?;
    
    println!("   📧 Rendered Email:");
    println!("      From: {}", message.from_address().unwrap_or("N/A".to_string()));
    println!("      To: {:?}", message.recipients());
    println!("      Subject: {}", message.get_header().get_first("Subject").unwrap_or(&"N/A".to_string()));
    
    Ok(())
}

async fn demo_welcome_template() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let template = CommonTemplates::welcome()
        .from("welcome@{{company_domain}}")
        .to("{{user_email}}");
    
    let context = TemplateContext::new()
        .set("company_name", "TechStart Inc.")
        .set("company_domain", "techstart.com")
        .set("user_name", "Alice Johnson")
        .set("user_email", "alice.johnson@example.com")
        .set("app_url", "https://app.techstart.com")
        .set("support_email", "support@techstart.com");
    
    let message = template.render(&context)?;
    
    println!("   📧 Welcome Email Generated:");
    println!("      Subject: {}", message.get_header().get_first("Subject").unwrap_or(&"N/A".to_string()));
    println!("      Preview: Welcome to TechStart Inc., Alice Johnson!");
    
    Ok(())
}

async fn demo_newsletter_template() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let template = CommonTemplates::newsletter()
        .from("newsletter@{{company_domain}}")
        .to("{{subscriber_email}}");
    
    let context = TemplateContext::new()
        .set("company_name", "Tech Weekly")
        .set("company_domain", "techweekly.com")
        .set("newsletter_title", "🚀 This Week in Technology")
        .set("newsletter_content", 
            "• AI breakthroughs in machine learning\n\
            • New Rust 1.75 features released\n\
            • Cloud computing trends for 2025\n\
            • Open source spotlight: mail-rs")
        .set("subscriber_email", "subscriber@example.com")
        .set("unsubscribe_url", "https://techweekly.com/unsubscribe?token=abc123");
    
    let message = template.render(&context)?;
    
    println!("   📧 Newsletter Generated:");
    println!("      Subject: {}", message.get_header().get_first("Subject").unwrap_or(&"N/A".to_string()));
    
    Ok(())
}

async fn demo_order_confirmation_template() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let template = CommonTemplates::order_confirmation()
        .from("orders@{{company_domain}}")
        .to("{{customer_email}}");
    
    let context = TemplateContext::new()
        .set("company_name", "Online Store")
        .set("company_domain", "store.com")
        .set("customer_name", "Bob Smith")
        .set("customer_email", "bob.smith@example.com")
        .set("order_number", "ORD-2025-001")
        .set("order_date", "January 15, 2025")
        .set("order_total", "$89.99")
        .set("order_items", 
            "• Wireless Headphones - $59.99\n\
            • Phone Case - $19.99\n\
            • Shipping - $10.01")
        .set("shipping_address", 
            "123 Main St\n\
            Anytown, ST 12345\n\
            United States")
        .set("processing_time", "2-3")
        .set("tracking_url", "https://store.com/track/ORD-2025-001");
    
    let message = template.render(&context)?;
    
    println!("   📧 Order Confirmation Generated:");
    println!("      Subject: {}", message.get_header().get_first("Subject").unwrap_or(&"N/A".to_string()));
    
    Ok(())
}

fn demo_variable_extraction() {
    println!("   🔍 Analyzing template variables:");
    
    // Analyze welcome template
    let welcome_template = CommonTemplates::welcome();
    let variables = welcome_template.get_variables();
    println!("      Welcome template variables: {:?}", variables);
    
    // Analyze custom template
    let custom_template = EmailTemplate::new()
        .subject("{{event_name}} reminder for {{attendee_name}}")
        .text_body("Hi {{attendee_name}}, don't forget about {{event_name}} on {{event_date}} at {{event_location}}!");
    
    let custom_vars = custom_template.get_variables();
    println!("      Custom template variables: {:?}", custom_vars);
}