# 🎨 RustAxum Handlebars Templates

This document provides an overview of the comprehensive Handlebars template system integrated into your RustAxum application.

## 📁 Template Structure

```txt
resources/views/
├── layouts/           # Layout templates
│   ├── main.hbs      # Standard layout with Bootstrap 5 & navigation
│   ├── auth.hbs      # Authentication pages layout
│   └── dashboard.hbs # Admin dashboard layout
├── auth/             # Authentication templates
│   ├── login.hbs     # Login form
│   ├── register.hbs  # Registration with validation
│   ├── forgot-password.hbs # Password reset
│   └── change-password.hbs # Password change
├── dashboard/        # Dashboard & admin templates
│   └── index.hbs     # Dashboard with stats
├── pages/            # General content pages
│   └── home.hbs      # Homepage template
├── components/       # Reusable components
│   ├── alert.hbs     # Flash message alerts
│   ├── forms/
│   │   └── input.hbs # Universal form input component
│   ├── tables/
│   │   └── data-table.hbs # Data table with sorting
│   └── modals/
│       └── base-modal.hbs # Modal dialog component
├── emails/           # Email templates
│   ├── welcome.hbs   # Welcome email
│   └── order-confirmation.hbs # E-commerce confirmation
└── partials/         # Template partials (for future use)
```

## 🔧 Core Features

### Template Service

- **Thread-safe**: Uses `Arc<RwLock<Handlebars>>` for concurrent access
- **Auto-loading**: Recursively loads all `.hbs` files from templates directory
- **Global instance**: `TemplateService::global()` using `OnceLock` for singleton pattern
- **Template reloading**: Hot reload support with `reload_templates()` method
- **Template management**: Register, unregister, and clear templates dynamically
- **String templates**: Support for rendering template strings without files
- **Production-ready**: Async template rendering with comprehensive error handling

### Template Response

- **Layout system**: Automatic content injection into layouts with flexible layout selection
- **Global variables**: Auto-injected `app_name`, `app_url`, `year` from configuration
- **Status codes**: Support for custom HTTP status codes via `with_status()`
- **Error handling**: Graceful fallback with detailed error pages and logging
- **Async rendering**: Production-ready async template rendering with `render_async()`
- **Helper functions**: Convenience functions for common rendering scenarios
- **Streaming responses**: Efficient streaming response implementation

## 🎯 Layout System

### Main Layout (`layouts/main.hbs`)

- Bootstrap 5 integration
- Responsive navigation
- Flash message support
- User dropdown with avatar
- Automatic footer with copyright

### Auth Layout (`layouts/auth.hbs`)

- Gradient backgrounds with floating animations
- Form validation styling
- Social login buttons
- Password strength indicators
- Mobile-responsive design

### Dashboard Layout (`layouts/dashboard.hbs`)

- Collapsible sidebar navigation
- Permission-based menu items
- Real-time notifications
- Chart.js integration
- DataTables support

## 📝 Form Components

### Universal Input Component (`components/forms/input.hbs`)

Supports all input types:

- Text, email, password, number, date
- Select dropdowns with options
- Textareas with auto-resize
- File uploads with preview
- Checkboxes and radio buttons
- Date ranges
- Password toggles

**Usage Example:**

```rust
let input_data = json!({
    "name": "email",
    "type": "email",
    "label": "Email Address",
    "placeholder": "Enter your email",
    "required": true,
    "icon": "fas fa-envelope",
    "value": "user@example.com",
    "errors": ["Email is required"]
});
```

## 📊 Data Tables

### Advanced Data Table (`components/tables/data-table.hbs`)

Features:

- **Sorting**: Click column headers to sort
- **Filtering**: Multiple filter types (text, select, date-range)
- **Pagination**: Bootstrap pagination with customizable page sizes
- **Bulk actions**: Select multiple rows for bulk operations
- **Row actions**: Dropdown menus or button groups per row
- **Responsive**: Mobile-friendly with horizontal scrolling
- **Search**: Global search across all columns
- **Export**: Built-in CSV/Excel export buttons

**Column Types:**

- `avatar`: User profile with image and name
- `badge`: Colored status badges
- `boolean`: Checkmarks/X icons
- `currency`: Formatted monetary values
- `date`: Formatted dates with tooltips
- `image`: Thumbnail images
- `link`: Clickable links
- `progress`: Progress bars
- `tags`: Multiple badge tags
- `truncate`: Text truncation with tooltips

## 💬 Modal Dialogs

### Base Modal (`components/modals/base-modal.hbs`)

- **AJAX forms**: Automatic form submission with loading states
- **Validation**: Client-side and server-side error display
- **Auto-focus**: Automatic field focusing on show
- **Custom buttons**: Flexible footer button configuration
- **Loading states**: Spinner animations and disabled states
- **Responsive**: Mobile-optimized sizing

## 📧 Email Templates

### Welcome Email (`emails/welcome.hbs`)

- Responsive HTML design
- Social media links
- Unsubscribe handling
- Brand customization

### Order Confirmation (`emails/order-confirmation.hbs`)

- Product images and details
- Order tracking integration
- Billing and shipping addresses
- Progress indicators
- Customer support links

## 🎨 Custom Helpers

### Currency Formatting

```handlebars
{{formatCurrency 1299.99 "USD"}}
→ $1,299.99
{{formatCurrency 1150.5 "EUR"}}
→ €1,150.50
{{formatCurrency 999.99 "GBP"}}
→ £999.99
{{formatCurrency 500 "CAD"}}
→ 500.00 CAD
```

### Text Manipulation

```handlebars
{{truncate long_text 50}}
→ "This is a very long text that..."
{{uppercase "hello world"}}
→ "HELLO WORLD"
{{lowercase "HELLO WORLD"}}
→ "hello world"
```

### Math Operations

```handlebars
{{add 100 25}}
→ 125
{{subtract 100 25}}
→ 75
```

### JSON Helper

```handlebars
<script>
  const userData =
  {{{json user}}}; const config =
  {{{json app_config}}};
</script>
```

**Note**: All helpers are implemented with production-ready error handling and type safety.

## 🚀 Advanced Usage

### Helper Functions

The framework provides several helper functions for different scenarios:

```rust
use crate::app::http::responses::template_response::{
    view, view_with_layout, view_without_layout,
    render_template, render_template_with_layout,
    render_template_without_layout, render_template_with_status
};

// Async helper functions that return TemplateResponse
pub async fn dashboard() -> TemplateResponse {
    let data = json!({"title": "Dashboard"});
    view("dashboard/index", &data).await
}

// Direct Response rendering (production-ready)
pub async fn api_error() -> Response {
    let data = json!({"error": "Not found"});
    render_template_with_status("errors/404", &data, StatusCode::NOT_FOUND).await
}
```

### String Template Rendering

```rust
let template_service = TemplateService::global();
let result = template_service.render_string(
    "Hello {{name}}!",
    &json!({"name": "World"})
).await?;
```

## 🚀 Usage in Controllers

### Basic Template Rendering

```rust
use crate::app::http::responses::template_response::{
    TemplateResponse,
    render_template,
    render_template_with_layout
};

// Using TemplateResponse (recommended)
pub async fn index() -> impl IntoResponse {
    let data = json!({
        "title": "Dashboard",
        "user": current_user(),
        "stats": get_dashboard_stats()
    });

    TemplateResponse::new("dashboard/index", &data)
}

// Using helper functions (direct Response)
pub async fn page() -> Response {
    let data = json!({"title": "Welcome"});
    render_template("pages/home", &data).await
}
```

### With Custom Layout

```rust
TemplateResponse::new("auth/login", &data)
    .with_layout("layouts/auth")
```

### Without Layout

```rust
TemplateResponse::new("emails/welcome", &data)
    .without_layout()
```

### With Custom Status

```rust
TemplateResponse::new("errors/404", &data)
    .with_status(StatusCode::NOT_FOUND)
```

## 🔧 Configuration

### Environment Variables

```env
TEMPLATES_PATH=resources/views  # Template directory (default: resources/views)
APP_NAME=RustAxum             # Application name (injected globally)
APP_URL=http://localhost:3000  # Application URL (injected globally)
```

### Template Service Initialization

```rust
// Global instance (recommended - singleton pattern)
let service = TemplateService::global();

// Manual instance
let service = TemplateService::new()?;

// Template management
service.register_template("custom", "<h1>{{title}}</h1>").await?;
service.register_template_file("page", Path::new("template.hbs")).await?;
service.reload_templates().await?; // Hot reload in development
```

## 🎯 Best Practices

### Template Organization

1. **Layouts**: Keep layouts in `layouts/` directory
2. **Components**: Reusable components in `components/`
3. **Partials**: Small reusable pieces in `partials/`
4. **Feature-based**: Group templates by feature (auth, dashboard, etc.)

### Data Structure

1. **Consistent naming**: Use snake_case for template variables
2. **Nested objects**: Organize related data in objects
3. **Type safety**: Ensure data types match template expectations
4. **Global variables**: Leverage auto-injected globals (`app_name`, etc.)

### Performance

1. **Template caching**: Templates are cached after first load
2. **Partial reuse**: Use partials to avoid duplication
3. **Minimal data**: Only pass necessary data to templates
4. **Async rendering**: All template operations are async

## 🔍 Debugging

### Template Errors

- Comprehensive error messages with template name and context
- Missing variables render as empty strings (strict mode disabled)
- Invalid helpers cause compilation errors with detailed stack traces
- Production-ready error handling with logging via `tracing`

### Development Tools

- `cargo run --example template_demo` - Basic template testing and helper demonstrations
- Template reloading in development mode via `reload_templates()`
- Error pages with detailed information and proper HTTP status codes
- Template listing with `get_templates()` for debugging
- Template existence checking with `has_template()`

## 📱 Responsive Design

All templates include:

- **Mobile-first**: Bootstrap 5 responsive breakpoints
- **Touch-friendly**: Large tap targets and mobile navigation
- **Fast loading**: CDN-delivered CSS and JavaScript
- **Accessibility**: ARIA labels, semantic HTML, and keyboard navigation
- **Cross-browser**: Modern browser compatibility
- **Progressive enhancement**: Works without JavaScript

## 🎨 Styling

### CSS Framework

- **Bootstrap 5.3.0**: Latest responsive framework with utilities
- **CDN Delivery**: Fast loading via jsdelivr CDN
- **Custom CSS**: Additional styling via `css` template variable
- **Component styling**: Consistent styling across all components

### Theme Customization

- CSS custom properties for easy theming
- Gradient backgrounds and modern animations
- Dynamic CSS injection via template data
- Component-specific styling patterns
- Responsive navigation with user authentication states

## 🔧 Production Features

### Performance

- **Template caching**: Automatic caching after first load
- **Async rendering**: Non-blocking template rendering
- **Streaming responses**: Efficient content delivery
- **Singleton pattern**: Global template service instance
- **Memory efficient**: Arc<RwLock> for thread-safe sharing

### Error Handling

- **Graceful degradation**: Fallback error pages
- **Detailed logging**: Tracing integration for debugging
- **Production-ready**: Comprehensive error handling
- **Status code support**: Custom HTTP status codes
- **Context preservation**: Error messages with template context

### Security

- **XSS protection**: Safe HTML rendering by default
- **CSRF integration**: Built-in CSRF token support
- **Content-Type headers**: Proper HTTP headers
- **Input sanitization**: Safe template variable handling

This template system provides enterprise-grade functionality with developer-friendly APIs, making it easy to create complex, responsive web applications with RustAxum.
