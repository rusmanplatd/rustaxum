# 🎨 RustAxum Handlebars Templates

This document provides an overview of the comprehensive Handlebars template system integrated into your RustAxum application.

## 📁 Template Structure

```
resources/views/
├── layouts/           # Layout templates
│   ├── main.hbs      # Standard layout with Bootstrap
│   ├── auth.hbs      # Authentication pages layout
│   └── dashboard.hbs # Admin dashboard layout
├── auth/             # Authentication templates
│   ├── login.hbs     # Login form with social auth
│   ├── register.hbs  # Registration with validation
│   └── forgot-password.hbs
├── dashboard/        # Dashboard & admin templates
│   └── index.hbs     # Dashboard with charts & stats
├── pages/            # General content pages
│   └── home.hbs      # Homepage template
├── components/       # Reusable components
│   ├── forms/
│   │   └── input.hbs # Universal form input component
│   ├── tables/
│   │   └── data-table.hbs # Advanced data table with sorting
│   └── modals/
│       └── base-modal.hbs # Modal dialog component
├── emails/           # Email templates
│   ├── welcome.hbs   # Welcome email
│   └── order-confirmation.hbs # E-commerce confirmation
└── partials/         # Template partials
```

## 🔧 Core Features

### Template Service
- **Thread-safe**: Uses `Arc<RwLock<Handlebars>>` for concurrent access
- **Auto-loading**: Recursively loads all `.hbs` files from templates directory
- **Global instance**: `TemplateService::global()` for easy access
- **Template reloading**: Hot reload support for development

### Template Response
- **Layout system**: Automatic content injection into layouts
- **Global variables**: Auto-injected `app_name`, `app_url`, `year`
- **Status codes**: Support for custom HTTP status codes
- **Error handling**: Graceful fallback with error pages

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
{{formatCurrency 1299.99 "USD"}}  → $1,299.99
{{formatCurrency 1150.50 "EUR"}}  → €1,150.50
```

### Text Manipulation
```handlebars
{{truncate long_text 50}}         → "This is a very long text that..."
{{uppercase "hello world"}}       → "HELLO WORLD"
{{lowercase "HELLO WORLD"}}       → "hello world"
```

### Math Operations
```handlebars
{{add 100 25}}                    → 125
{{subtract 100 25}}               → 75
```

### JSON Helper
```handlebars
<script>
const data = {{{json user}}};
</script>
```

## 🚀 Usage in Controllers

### Basic Template Rendering
```rust
use crate::app::http::responses::template_response::TemplateResponse;

pub async fn index() -> impl IntoResponse {
    let data = json!({
        "title": "Dashboard",
        "user": current_user(),
        "stats": get_dashboard_stats()
    });

    TemplateResponse::new("dashboard/index", &data)
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
```

### Template Service Initialization
```rust
// Global instance (automatic)
let service = TemplateService::global();

// Manual instance
let service = TemplateService::new()?;
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
- Errors show template name and line number
- Missing variables render as empty strings (strict mode disabled)
- Invalid helpers cause compilation errors

### Development Tools
- `cargo run --example template_demo` - Basic template testing
- `cargo run --example advanced_template_demo` - Advanced features
- Template reloading in development mode
- Error pages with detailed information

## 📱 Responsive Design

All templates include:
- **Mobile-first**: Responsive breakpoints
- **Touch-friendly**: Large tap targets
- **Fast loading**: Optimized CSS and JavaScript
- **Accessibility**: ARIA labels and semantic HTML
- **Cross-browser**: Modern browser compatibility

## 🎨 Styling

### CSS Framework
- **Bootstrap 5**: Latest responsive framework
- **Font Awesome**: Icon library
- **Custom CSS**: Additional styling for components

### Theme Customization
- CSS custom properties for easy theming
- Gradient backgrounds and modern animations
- Dark mode support (planned)
- Component-specific styling

This template system provides enterprise-grade functionality with developer-friendly APIs, making it easy to create complex, responsive web applications with RustAxum.