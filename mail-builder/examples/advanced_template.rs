use mail_core::{Message, Address};
use mail_builder::{
    AdvancedTemplateEngine, AdvancedTemplateContext, EmailTemplateDefinition,
    TemplateOptions, TemplateHelper
};
use serde_json::{json, Value};

/// Custom helper for formatting currency
#[derive(Debug)]
struct CurrencyHelper;

impl TemplateHelper for CurrencyHelper {
    fn execute(&self, args: &[Value], _context: &AdvancedTemplateContext) -> Result<String, mail_builder::advanced_template::TemplateError> {
        if let Some(amount_val) = args.first() {
            if let Some(amount) = amount_val.as_f64() {
                let currency = args.get(1)
                    .and_then(|v| v.as_str())
                    .unwrap_or("USD");
                Ok(format!("{} ${:.2}", currency, amount))
            } else {
                Ok("$0.00".to_string())
            }
        } else {
            Ok("$0.00".to_string())
        }
    }

    fn name(&self) -> &str {
        "currency"
    }

    fn description(&self) -> &str {
        "Format number as currency"
    }
}

/// Custom helper for generating URLs
#[derive(Debug)]
struct UrlHelper;

impl TemplateHelper for UrlHelper {
    fn execute(&self, args: &[Value], context: &AdvancedTemplateContext) -> Result<String, mail_builder::advanced_template::TemplateError> {
        if let Some(path_val) = args.first() {
            let path = path_val.as_str().unwrap_or("");
            let base_url = context.get("base_url")
                .and_then(|v| v.as_str())
                .unwrap_or("https://example.com");
            Ok(format!("{}{}", base_url, path))
        } else {
            Ok("#".to_string())
        }
    }

    fn name(&self) -> &str {
        "url"
    }

    fn description(&self) -> &str {
        "Generate full URL from path"
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Enhanced Template Engine Example");
    println!("===============================");

    // Create template engine with custom options
    let options = TemplateOptions {
        auto_escape: true,
        allow_missing_vars: true,
        missing_var_default: "[MISSING]".to_string(),
        trim_whitespace: true,
    };

    let mut engine = AdvancedTemplateEngine::with_options(options);

    // Register custom helpers
    engine.register_helper(Box::new(CurrencyHelper));
    engine.register_helper(Box::new(UrlHelper));

    // Register partial templates
    engine.register_partial("header", r#"
        <div style="background: #f4f4f4; padding: 20px; text-align: center;">
            <h1 style="color: #333;">{{company_name}}</h1>
            <p>Your trusted partner since {{founded_year}}</p>
        </div>
    "#);

    engine.register_partial("footer", r#"
        <div style="background: #333; color: white; padding: 20px; text-align: center;">
            <p>&copy; {{current_year}} {{company_name}}. All rights reserved.</p>
            <p>Contact us: {{support_email}} | {{support_phone}}</p>
        </div>
    "#);

    println!("✓ Template engine configured with custom helpers and partials");

    // Create rich template context
    let context = AdvancedTemplateContext::new()
        .set_string("recipient_name", "John Smith")
        .set_string("recipient_email", "john.smith@example.com")
        .set_string("sender_email", "noreply@acmecorp.com")
        .set_string("company_name", "ACME Corporation")
        .set_string("founded_year", "1985")
        .set_string("current_year", "2025")
        .set_string("support_email", "support@acmecorp.com")
        .set_string("support_phone", "+1-800-555-0123")
        .set_string("base_url", "https://acmecorp.com")
        .set_number("order_total", 1299.99)
        .set_array("order_items", vec![
            json!({
                "name": "Premium Widget Pro",
                "quantity": 2,
                "price": 599.99,
                "total": 1199.98
            }),
            json!({
                "name": "Shipping & Handling",
                "quantity": 1,
                "price": 100.01,
                "total": 100.01
            })
        ])
        .set_object("user_preferences", {
            let mut prefs = serde_json::Map::new();
            prefs.insert("newsletter".to_string(), json!(true));
            prefs.insert("promotions".to_string(), json!(false));
            prefs.insert("language".to_string(), json!("en"));
            prefs.insert("timezone".to_string(), json!("America/New_York"));
            prefs
        });

    println!("✓ Rich template context created with user data, orders, and preferences");

    // Test basic variable substitution
    println!("\n🔧 Testing Basic Variable Substitution:");
    let simple_template = "Hello {{recipient_name}}, welcome to {{company_name}}!";
    let simple_result = engine.render(simple_template, &context)?;
    println!("  Template: {}", simple_template);
    println!("  Result: {}", simple_result);

    // Test nested variable access
    println!("\n🔧 Testing Nested Variable Access:");
    let nested_template = "Language: {{user_preferences.language}}, Newsletter: {{user_preferences.newsletter}}";
    let nested_result = engine.render(nested_template, &context)?;
    println!("  Template: {}", nested_template);
    println!("  Result: {}", nested_result);

    // Test built-in helpers
    println!("\n🔧 Testing Built-in Helpers:");
    let helper_template = "Welcome {{uppercase(recipient_name)}}! Total items: {{length(order_items)}}";
    let helper_result = engine.render(helper_template, &context)?;
    println!("  Template: {}", helper_template);
    println!("  Result: {}", helper_result);

    // Test custom helpers
    println!("\n🔧 Testing Custom Helpers:");
    let custom_helper_template = "Order total: {{currency(order_total)}} | Support: {{url('/support')}}";
    let custom_helper_result = engine.render(custom_helper_template, &context)?;
    println!("  Template: {}", custom_helper_template);
    println!("  Result: {}", custom_helper_result);

    // Create comprehensive email template
    let email_template = EmailTemplateDefinition::new()
        .from("{{sender_email}}")
        .to("{{recipient_email}}")
        .subject("Your Order Confirmation - {{company_name}}")
        .html_body(r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Order Confirmation</title>
</head>
<body style="font-family: Arial, sans-serif; margin: 0; padding: 0;">
    {{> header}}
    
    <div style="padding: 30px;">
        <h2>Thank you for your order, {{capitalize(recipient_name)}}!</h2>
        
        <p>We're excited to confirm your recent purchase. Here are the details:</p>
        
        <div style="background: #f9f9f9; padding: 20px; border-radius: 5px; margin: 20px 0;">
            <h3>Order Summary</h3>
            <table style="width: 100%; border-collapse: collapse;">
                <thead>
                    <tr style="background: #e9e9e9;">
                        <th style="padding: 10px; text-align: left;">Item</th>
                        <th style="padding: 10px; text-align: center;">Qty</th>
                        <th style="padding: 10px; text-align: right;">Price</th>
                        <th style="padding: 10px; text-align: right;">Total</th>
                    </tr>
                </thead>
                <tbody>
                    <!-- In a full implementation, this would use #each loop -->
                    <tr>
                        <td style="padding: 10px; border-bottom: 1px solid #ddd;">Premium Widget Pro</td>
                        <td style="padding: 10px; text-align: center; border-bottom: 1px solid #ddd;">2</td>
                        <td style="padding: 10px; text-align: right; border-bottom: 1px solid #ddd;">$599.99</td>
                        <td style="padding: 10px; text-align: right; border-bottom: 1px solid #ddd;">$1,199.98</td>
                    </tr>
                    <tr>
                        <td style="padding: 10px; border-bottom: 1px solid #ddd;">Shipping & Handling</td>
                        <td style="padding: 10px; text-align: center; border-bottom: 1px solid #ddd;">1</td>
                        <td style="padding: 10px; text-align: right; border-bottom: 1px solid #ddd;">$100.01</td>
                        <td style="padding: 10px; text-align: right; border-bottom: 1px solid #ddd;">$100.01</td>
                    </tr>
                    <tr style="font-weight: bold; background: #f0f0f0;">
                        <td colspan="3" style="padding: 15px; text-align: right;">Grand Total:</td>
                        <td style="padding: 15px; text-align: right;">{{currency(order_total)}}</td>
                    </tr>
                </tbody>
            </table>
        </div>
        
        <div style="margin: 30px 0;">
            <h3>What's Next?</h3>
            <ul>
                <li>📦 Your order is being processed</li>
                <li>📧 You'll receive a shipping notification within 24 hours</li>
                <li>🚚 Estimated delivery: 3-5 business days</li>
                <li>📞 Questions? Contact us at {{support_email}}</li>
            </ul>
        </div>
        
        <div style="text-align: center; margin: 30px 0;">
            <a href="{{url('/orders/track')}}" 
               style="background: #007cba; color: white; padding: 12px 30px; text-decoration: none; border-radius: 5px; display: inline-block;">
                Track Your Order
            </a>
        </div>
        
        <div style="background: #e7f3ff; padding: 20px; border-radius: 5px; margin: 20px 0;">
            <h4>💡 Pro Tip</h4>
            <p>Create an account to track all your orders, save favorites, and get exclusive offers!</p>
            <a href="{{url('/account/register')}}" style="color: #007cba;">Create Account →</a>
        </div>
    </div>
    
    {{> footer}}
</body>
</html>
        "#)
        .text_body(r#"
{{company_name}} - Order Confirmation

Thank you for your order, {{capitalize(recipient_name)}}!

ORDER SUMMARY:
- Premium Widget Pro (2x) = $1,199.98
- Shipping & Handling (1x) = $100.01
---
Grand Total: {{currency(order_total)}}

WHAT'S NEXT:
* Your order is being processed
* You'll receive shipping notification within 24 hours
* Estimated delivery: 3-5 business days
* Questions? Contact {{support_email}}

Track your order: {{url('/orders/track')}}

{{company_name}}
{{support_email}} | {{support_phone}}
        "#);

    println!("\n📧 Rendering Complete Email Template:");

    // Render the email
    let message = engine.render_email(&email_template, &context)?;
    println!("✓ Email template rendered successfully");

    // Display email details
    println!("\n📋 Email Details:");
    let formatted_message = message.format()?;
    let email_str = String::from_utf8_lossy(&formatted_message);
    
    // Extract and display subject
    if let Some(subject_start) = email_str.find("Subject: ") {
        if let Some(subject_end) = email_str[subject_start..].find("\r\n") {
            let subject = &email_str[subject_start + 9..subject_start + subject_end];
            println!("  Subject: {}", subject);
        }
    }

    // Display size information
    println!("  Email size: {} bytes", formatted_message.len());
    println!("  Content includes: HTML body, text alternative, headers");

    // Demonstrate partial rendering
    println!("\n🧩 Testing Partial Templates:");
    let header_result = engine.render("{{> header}}", &context)?;
    println!("  Header partial rendered: {} characters", header_result.len());
    
    let footer_result = engine.render("{{> footer}}", &context)?;
    println!("  Footer partial rendered: {} characters", footer_result.len());

    // Demonstrate error handling
    println!("\n❌ Testing Error Handling:");
    
    // Missing variable (should show default)
    let missing_var_result = engine.render("Hello {{missing_variable}}!", &context)?;
    println!("  Missing variable result: '{}'", missing_var_result);
    
    // Missing partial (should error)
    match engine.render("{{> nonexistent_partial}}", &context) {
        Ok(_) => println!("  Unexpected success with missing partial"),
        Err(e) => println!("  Missing partial error: {}", e),
    }

    println!("\n🎯 Template Engine Features Demonstrated:");
    println!("  ✓ Variable substitution with {{variable}} syntax");
    println!("  ✓ Nested object access with dot notation");
    println!("  ✓ Built-in helpers (uppercase, lowercase, capitalize, length)");
    println!("  ✓ Custom helpers (currency, url)");
    println!("  ✓ Partial templates with {{> partial_name}}");
    println!("  ✓ Rich template context with JSON data structures");
    println!("  ✓ Email template rendering with headers and body");
    println!("  ✓ Error handling for missing variables and partials");
    println!("  ✓ Configurable options (auto-escape, missing vars, etc.)");

    println!("\n💡 Advanced Features Available:");
    println!("  • Conditional rendering with {{#if condition}}");
    println!("  • Loop rendering with {{#each array}}");
    println!("  • Complex data structures (arrays, objects)");
    println!("  • Custom helper registration");
    println!("  • Template inheritance and composition");
    println!("  • Auto-escaping for security");
    println!("  • Missing variable handling");

    Ok(())
}