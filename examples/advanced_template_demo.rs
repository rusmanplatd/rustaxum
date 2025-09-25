use rustaxum::app::services::template_service::TemplateService;
use serde_json::json;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🎨 Advanced Handlebars Template Demo");
    println!("=====================================\n");

    // Initialize the template service
    let template_service = TemplateService::new()?;

    // Test advanced template features
    test_helpers(&template_service).await?;
    test_conditionals(&template_service).await?;
    test_loops(&template_service).await?;
    test_partials(&template_service).await?;
    test_complex_data(&template_service).await?;

    println!("\n✅ All template features working correctly!");

    // Show template locations
    println!("\n📁 Template Locations:");
    println!("   - Authentication: resources/views/auth/");
    println!("   - Dashboard: resources/views/dashboard/");
    println!("   - Layouts: resources/views/layouts/");
    println!("   - Components: resources/views/components/");
    println!("   - Emails: resources/views/emails/");

    println!("\n🔧 Advanced Features:");
    println!("   - Custom helpers: formatCurrency, truncate, json, add, subtract");
    println!("   - Layout system with content injection");
    println!("   - Component-based templates");
    println!("   - Form validation & error display");
    println!("   - Data tables with sorting & pagination");
    println!("   - Modal dialogs with AJAX forms");
    println!("   - Responsive email templates");

    Ok(())
}

async fn test_helpers(service: &TemplateService) -> Result<()> {
    println!("1. Testing Custom Helpers:");

    // Currency formatting
    let currency_template = r#"
Price: {{formatCurrency price "USD"}}
Euro Price: {{formatCurrency euro_price "EUR"}}
"#;
    let currency_data = json!({
        "price": 1299.99,
        "euro_price": 1150.50
    });
    let result = service.render_string(currency_template, &currency_data).await?;
    println!("   Currency: {}", result.trim());

    // Text truncation
    let truncate_template = r#"
Short: {{truncate long_text 20}}
Full: {{long_text}}
"#;
    let truncate_data = json!({
        "long_text": "This is a very long text that should be truncated for display purposes."
    });
    let result = service.render_string(truncate_template, &truncate_data).await?;
    println!("   Truncate: {}", result.trim());

    // Math operations
    let math_template = r#"
Sum: {{add num1 num2}}
Difference: {{subtract num1 num2}}
"#;
    let math_data = json!({
        "num1": 100,
        "num2": 25
    });
    let result = service.render_string(math_template, &math_data).await?;
    println!("   Math: {}", result.trim());

    // JSON helper
    let json_template = r#"
<script>
const data = {{{json user}}};
console.log(data);
</script>
"#;
    let json_data = json!({
        "user": {
            "id": 1,
            "name": "John Doe",
            "email": "john@example.com"
        }
    });
    let result = service.render_string(json_template, &json_data).await?;
    println!("   JSON: {}", result.trim());

    Ok(())
}

async fn test_conditionals(service: &TemplateService) -> Result<()> {
    println!("\n2. Testing Conditionals & Logic:");

    let template = r#"
{{#if user}}
  Welcome, {{user.name}}!
  {{#if user.is_admin}}
    <admin-panel>You have admin access</admin-panel>
  {{else}}
    <user-panel>Standard user access</user-panel>
  {{/if}}
{{else}}
  Please log in to continue.
{{/if}}

{{#unless user.verified}}
  ⚠️  Please verify your email address.
{{/unless}}

{{#with user.profile}}
  Profile: {{bio}}
  Location: {{location}}
{{/with}}
"#;

    let data_admin = json!({
        "user": {
            "name": "Admin User",
            "is_admin": true,
            "verified": true,
            "profile": {
                "bio": "System administrator",
                "location": "San Francisco, CA"
            }
        }
    });

    let data_user = json!({
        "user": {
            "name": "Regular User",
            "is_admin": false,
            "verified": false,
            "profile": {
                "bio": "Software developer",
                "location": "New York, NY"
            }
        }
    });

    let result_admin = service.render_string(template, &data_admin).await?;
    let result_user = service.render_string(template, &data_user).await?;

    println!("   Admin view: {}", result_admin.trim());
    println!("   User view: {}", result_user.trim());

    Ok(())
}

async fn test_loops(service: &TemplateService) -> Result<()> {
    println!("\n3. Testing Loops & Iteration:");

    let template = r#"
<h3>Team Members ({{team.members.length}} total):</h3>
<ul>
{{#each team.members}}
  <li>
    {{@index}}: {{name}} - {{role}}
    {{#if @first}}(Team Lead){{/if}}
    {{#if @last}}(Newest Member){{/if}}
  </li>
{{/each}}
</ul>

<h3>Project Stats:</h3>
{{#each stats}}
<div class="stat">
  <strong>{{@key}}:</strong> {{this}}
</div>
{{/each}}

{{#unless team.members}}
<p>No team members found.</p>
{{/unless}}
"#;

    let data = json!({
        "team": {
            "members": [
                {"name": "Alice Smith", "role": "Project Manager"},
                {"name": "Bob Jones", "role": "Senior Developer"},
                {"name": "Carol White", "role": "UX Designer"},
                {"name": "David Brown", "role": "DevOps Engineer"}
            ]
        },
        "stats": {
            "commits": 1247,
            "issues_closed": 89,
            "tests_passing": "98%",
            "code_coverage": "85%"
        }
    });

    let result = service.render_string(template, &data).await?;
    println!("   {}", result.trim());

    Ok(())
}

async fn test_partials(service: &TemplateService) -> Result<()> {
    println!("\n4. Testing Template Composition:");

    // Register a partial template
    let user_card_partial = r#"
<div class="user-card">
  <img src="{{avatar}}" alt="{{name}}">
  <h3>{{name}}</h3>
  <p>{{email}}</p>
  <span class="badge {{role_class}}">{{role}}</span>
</div>
"#;

    service.register_template("user-card", user_card_partial).await?;

    let main_template = r#"
<div class="users-grid">
{{#each users}}
  {{> user-card}}
{{/each}}
</div>
"#;

    let data = json!({
        "users": [
            {
                "name": "John Doe",
                "email": "john@example.com",
                "avatar": "https://ui-avatars.com/api/?name=John+Doe&background=667eea&color=fff",
                "role": "Admin",
                "role_class": "admin"
            },
            {
                "name": "Jane Smith",
                "email": "jane@example.com",
                "avatar": "https://ui-avatars.com/api/?name=Jane+Smith&background=28a745&color=fff",
                "role": "User",
                "role_class": "user"
            }
        ]
    });

    let result = service.render_string(main_template, &data).await?;
    println!("   Partial rendering: {}", result.trim());

    Ok(())
}

async fn test_complex_data(service: &TemplateService) -> Result<()> {
    println!("\n5. Testing Complex Data Structures:");

    let template = r#"
<div class="dashboard">
  <h1>{{company.name}} Dashboard</h1>

  <div class="metrics">
    {{#each metrics}}
    <div class="metric {{type}}">
      <span class="value">{{formatCurrency value currency}}</span>
      <span class="label">{{label}}</span>
      <span class="trend {{trend.direction}}">
        {{trend.percentage}}% {{trend.period}}
      </span>
    </div>
    {{/each}}
  </div>

  <div class="recent-orders">
    <h2>Recent Orders</h2>
    {{#each orders}}
    <div class="order">
      <span class="id">#{{id}}</span>
      <span class="customer">{{customer.name}}</span>
      <span class="total">{{formatCurrency total "USD"}}</span>
      <span class="status {{status_class}}">{{uppercase status}}</span>
      <div class="items">
        {{#each items}}
        - {{name}} ({{quantity}}x {{formatCurrency price "USD"}})
        {{/each}}
      </div>
    </div>
    {{/each}}
  </div>

  <div class="footer">
    <p>Generated: {{timestamp}}</p>
    <p>Data: {{{json summary}}}</p>
  </div>
</div>
"#;

    let data = json!({
        "company": {
            "name": "RustAxum Corp"
        },
        "metrics": [
            {
                "label": "Revenue",
                "value": 125847.50,
                "currency": "USD",
                "type": "revenue",
                "trend": {
                    "direction": "up",
                    "percentage": 15,
                    "period": "vs last month"
                }
            },
            {
                "label": "Orders",
                "value": 1247,
                "currency": "",
                "type": "orders",
                "trend": {
                    "direction": "up",
                    "percentage": 8,
                    "period": "vs last week"
                }
            }
        ],
        "orders": [
            {
                "id": "ORD-2024-001",
                "customer": {
                    "name": "Acme Inc"
                },
                "total": 2499.99,
                "status": "completed",
                "status_class": "success",
                "items": [
                    {"name": "Premium Package", "quantity": 1, "price": 1999.99},
                    {"name": "Support Contract", "quantity": 1, "price": 500.00}
                ]
            },
            {
                "id": "ORD-2024-002",
                "customer": {
                    "name": "Tech Startup LLC"
                },
                "total": 899.99,
                "status": "pending",
                "status_class": "warning",
                "items": [
                    {"name": "Basic Package", "quantity": 1, "price": 899.99}
                ]
            }
        ],
        "timestamp": "2024-01-15 14:30:00",
        "summary": {
            "total_customers": 156,
            "active_subscriptions": 89,
            "monthly_recurring_revenue": 15420.00
        }
    });

    let result = service.render_string(template, &data).await?;
    println!("   Complex data: {}", result.lines().take(10).collect::<Vec<_>>().join("\n"));
    println!("   ... (truncated for brevity)");

    Ok(())
}