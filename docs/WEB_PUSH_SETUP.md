# Web Push Notifications Setup Guide

This guide explains how to set up and use the web push notification system in your RustAxum application.

## Overview

The web push notification system allows you to send real-time notifications to users' browsers even when they're not actively using your website. It's built on standard web technologies and integrates seamlessly with the existing notification system.

## Features

- **Multi-channel notifications**: Send via email, database, and web push simultaneously
- **Service worker integration**: Handles notifications in the background
- **Subscription management**: Users can subscribe/unsubscribe easily
- **Offline support**: Notifications work even when the browser is closed
- **Action buttons**: Add interactive buttons to notifications
- **Laravel-like API**: Familiar notification patterns for Laravel developers

## Environment Setup

### 1. Generate VAPID Keys

VAPID keys are required for web push notifications. You can generate them using the web-push library:

```bash
# Install web-push CLI tool (Node.js required)
npm install -g web-push

# Generate VAPID keys
web-push generate-vapid-keys
```

### 2. Configure Environment Variables

Add these variables to your `.env` file:

```env
# Web Push Configuration
VAPID_PRIVATE_KEY=your_vapid_private_key_here
VAPID_PUBLIC_KEY=your_vapid_public_key_here
VAPID_SUBJECT=mailto:admin@yourapp.com
```

### 3. Run Database Migrations

```bash
# Run the migrations to create the push_subscriptions table
cargo run --bin artisan -- migrate
```

## Frontend Integration

### 1. Include JavaScript Files

Add these files to your HTML pages:

```html
<!-- Service Worker Registration -->
<script src="/push-notifications.js"></script>

<!-- Your service worker file should be accessible at /sw.js -->
```

### 2. Basic Usage

```javascript
// Initialize push notifications
const pushManager = window.pushNotificationManager;

// Check if supported
if (pushManager.isSupported) {
    // Subscribe to notifications
    document.getElementById('subscribe-btn').addEventListener('click', async () => {
        try {
            await pushManager.subscribe();
            console.log('Successfully subscribed to push notifications');
        } catch (error) {
            console.error('Failed to subscribe:', error);
        }
    });

    // Unsubscribe from notifications
    document.getElementById('unsubscribe-btn').addEventListener('click', async () => {
        try {
            await pushManager.unsubscribe();
            console.log('Successfully unsubscribed from push notifications');
        } catch (error) {
            console.error('Failed to unsubscribe:', error);
        }
    });

    // Send test notification
    document.getElementById('test-btn').addEventListener('click', async () => {
        try {
            await pushManager.sendTestNotification('Test', 'Hello from RustAxum!');
            console.log('Test notification sent');
        } catch (error) {
            console.error('Failed to send test notification:', error);
        }
    });
}
```

### 3. Check Subscription Status

```javascript
// Get current status
const status = pushManager.getSubscriptionStatus();
console.log('Supported:', status.supported);
console.log('Permission:', status.permission);
console.log('Subscribed:', status.subscribed);
```

## Backend Usage

### 1. Create Notifications

Generate a new notification class:

```bash
cargo run --bin artisan -- make notification OrderShipped
```

This creates a notification class with web push support:

```rust
use crate::app::notifications::order_shipped_notification::OrderShippedNotification;
use crate::app::services::notification_service::NotificationService;

// Create notification
let notification = OrderShippedNotification::new(
    "ORD-123456".to_string(),
    "1Z999AA1234567890".to_string(),
    "John Doe".to_string(),
    "UPS".to_string(),
    "December 25, 2025".to_string(),
);

// Send notification (will use all configured channels including web push)
let service = NotificationService::new().await;
service.send(&notification, &user).await?;
```

### 2. Customize Web Push Messages

Override the `to_web_push` method in your notification:

```rust
async fn to_web_push(&self, _notifiable: &dyn Notifiable) -> Result<WebPushMessage> {
    let notification = WebPushMessage::new(
        "📦 Order Shipped!".to_string(),
        format!("Your order #{} has been shipped!", self.order_id),
    )
    .icon("/static/images/package-icon.png".to_string())
    .badge("/static/images/shipping-badge.png".to_string())
    .tag(format!("order-shipped-{}", self.order_id))
    .require_interaction(true)
    .add_action(NotificationAction::new(
        "track".to_string(),
        "Track Package".to_string(),
    ))
    .add_action(NotificationAction::new(
        "view-order".to_string(),
        "View Order".to_string(),
    ));

    Ok(notification)
}
```

### 3. Send Web Push Only

If you want to send only web push notifications:

```rust
async fn via(&self, _notifiable: &dyn Notifiable) -> Vec<NotificationChannel> {
    vec![NotificationChannel::WebPush]
}
```

## API Endpoints

The system provides several API endpoints for managing web push notifications:

### Public Endpoints

- `GET /api/web-push/vapid-public-key` - Get VAPID public key for subscription
- `GET /api/web-push/status` - Check web push configuration status

### Authenticated Endpoints

- `POST /api/web-push/subscribe` - Subscribe to push notifications
- `DELETE /api/web-push/unsubscribe` - Unsubscribe from push notifications
- `GET /api/web-push/subscriptions` - Get user's push subscriptions
- `POST /api/web-push/test` - Send a test notification

### Admin Endpoints

- `POST /api/web-push/cleanup` - Clean up invalid subscriptions

## Service Worker Features

The service worker (`/sw.js`) provides:

- **Push event handling**: Receives and displays notifications
- **Notification click handling**: Opens appropriate pages when clicked
- **Offline caching**: Caches important assets for offline use
- **Background sync**: Handles offline actions when connection is restored
- **Subscription management**: Automatically handles subscription changes

## Customization

### Notification Icons and Images

Place notification assets in your `public` directory:

```
public/
├── static/
│   └── images/
│       ├── icon-192x192.png      # Default notification icon
│       ├── badge-72x72.png       # Notification badge
│       ├── package-icon.png      # Custom notification icons
│       └── track-icon.png        # Action button icons
└── sw.js                         # Service worker
```

### Notification Actions

Add interactive buttons to notifications:

```rust
let notification = WebPushMessage::new(title, body)
    .add_action(NotificationAction::new(
        "action_id".to_string(),
        "Button Text".to_string(),
    ).icon("/static/images/action-icon.png".to_string()));
```

Handle actions in your service worker or update the existing one in `/sw.js`.

## Browser Support

Web push notifications are supported in:

- Chrome 42+
- Firefox 44+
- Safari 16+ (macOS 13+, iOS 16.4+)
- Edge 17+

The system gracefully degrades for unsupported browsers.

## Security Considerations

1. **HTTPS Required**: Web push only works over HTTPS (except localhost for development)
2. **User Permission**: Users must explicitly grant permission for notifications
3. **VAPID Keys**: Keep your private VAPID key secure and never expose it to the client
4. **Rate Limiting**: Consider implementing rate limiting for notification endpoints

## Troubleshooting

### Common Issues

1. **Notifications not showing**: Check browser permissions and HTTPS setup
2. **Subscription failed**: Verify VAPID keys are correctly configured
3. **Service worker not registering**: Ensure `/sw.js` is accessible and valid
4. **Database errors**: Run migrations and check PostgreSQL connection

### Debug Mode

Enable debug logging in the client library:

```javascript
const pushManager = new PushNotificationManager({
    debug: true
});
```

### Testing

Use the provided test endpoints:

```bash
# Test web push configuration
curl http://localhost:3000/api/web-push/status

# Send test notification (requires authentication)
curl -X POST http://localhost:3000/api/web-push/test \
  -H "Authorization: Bearer YOUR_TOKEN"
```

## Production Deployment

1. Ensure HTTPS is properly configured
2. Set production VAPID keys
3. Configure proper CSP headers for service workers
4. Set up monitoring for notification delivery rates
5. Implement cleanup jobs for invalid subscriptions

## Examples

### Complete Integration Example

```html
<!DOCTYPE html>
<html>
<head>
    <title>Web Push Demo</title>
</head>
<body>
    <div id="push-controls">
        <button id="subscribe-btn">Subscribe to Notifications</button>
        <button id="unsubscribe-btn">Unsubscribe</button>
        <button id="test-btn">Send Test Notification</button>
        <div id="status"></div>
    </div>

    <script src="/push-notifications.js"></script>
    <script>
        document.addEventListener('DOMContentLoaded', function() {
            const pushManager = window.pushNotificationManager;
            const statusDiv = document.getElementById('status');

            function updateStatus() {
                const status = pushManager.getSubscriptionStatus();
                statusDiv.innerHTML = `
                    <p>Supported: ${status.supported}</p>
                    <p>Permission: ${status.permission}</p>
                    <p>Subscribed: ${status.subscribed}</p>
                `;
            }

            // Event listeners
            document.getElementById('subscribe-btn').addEventListener('click', async () => {
                try {
                    await pushManager.subscribe();
                    updateStatus();
                    alert('Subscribed successfully!');
                } catch (error) {
                    alert('Subscription failed: ' + error.message);
                }
            });

            document.getElementById('unsubscribe-btn').addEventListener('click', async () => {
                try {
                    await pushManager.unsubscribe();
                    updateStatus();
                    alert('Unsubscribed successfully!');
                } catch (error) {
                    alert('Unsubscription failed: ' + error.message);
                }
            });

            document.getElementById('test-btn').addEventListener('click', async () => {
                try {
                    await pushManager.sendTestNotification();
                    alert('Test notification sent!');
                } catch (error) {
                    alert('Test failed: ' + error.message);
                }
            });

            // Initial status update
            updateStatus();
        });
    </script>
</body>
</html>
```

This completes the web push notification system implementation for your RustAxum framework!