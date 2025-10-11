use axum::{routing::{get, post}, Router, response::Html, middleware};
use crate::database::DbPool;
use crate::app::http::controllers::home_controller;
use crate::app::http::controllers::csrf_controller;
use crate::app::http::controllers::web_auth_controller;
use crate::app::http::controllers::mfa_controller;
use crate::app::http::middleware::auth_guard::{auth_guard, mfa_guard};

pub fn routes() -> Router<DbPool> {
    tracing::debug!("Creating web routes...");

    // Public web authentication routes
    let auth_routes = Router::new()
        .route("/auth/login", get(web_auth_controller::show_login))
        .route("/auth/login", post(web_auth_controller::login))
        .route("/auth/register", get(web_auth_controller::show_register))
        .route("/auth/register", post(web_auth_controller::register))
        .route("/auth/forgot-password", get(web_auth_controller::show_forgot_password))
        .route("/auth/forgot-password", post(web_auth_controller::forgot_password))
        .route("/auth/reset-password/{token}", get(web_auth_controller::show_reset_password))
        .route("/auth/reset-password", get(web_auth_controller::show_reset_password_query))
        .route("/auth/reset-password", post(web_auth_controller::reset_password));

    // Protected web authentication routes
    let protected_auth_routes = Router::new()
        .route("/auth/logout", post(web_auth_controller::logout))
        .route("/auth/change-password", get(web_auth_controller::show_change_password))
        .route("/auth/change-password", post(web_auth_controller::change_password))
        .route("/dashboard", get(web_auth_controller::dashboard))
        // Non-setup MFA management routes require full auth
        .route("/mfa/disable", post(mfa_controller::disable_mfa))
        .route("/mfa/backup-codes", post(mfa_controller::regenerate_backup_codes))
        .route("/mfa/methods", get(mfa_controller::get_mfa_methods))
        .route_layer(middleware::from_fn(auth_guard));

    // MFA setup and verification routes use relaxed MFA guard
    let mfa_routes = Router::new()
        .route("/mfa", get(mfa_controller::show_setup_page))
        .route("/mfa/setup", post(mfa_controller::setup_mfa))
        .route("/mfa/verify-page", get(mfa_controller::show_verify_page)) // Login MFA verification page
        .route("/mfa/verify", post(mfa_controller::verify_mfa))
        .route_layer(middleware::from_fn(mfa_guard));

    // Public routes
    let public_routes = Router::new()
        .route("/", get(home_controller::index))
        .route("/health", get(health_check))
        .route("/web-push-demo", get(web_push_demo))
        // CSRF test routes
        .route("/csrf/token", get(csrf_controller::token))
        .route("/csrf/form", get(csrf_controller::form))
        .route("/csrf/test", post(csrf_controller::test_form))
        .route("/csrf/api-test", post(csrf_controller::test_api))
        .route("/csrf/regenerate", post(csrf_controller::regenerate))
        // Documentation UIs - custom HTML that references the OpenAPI endpoint
        .route("/docs/swagger", get(swagger_ui))
        .route("/docs/rapidoc", get(rapidoc_ui))
        .route("/docs/redoc", get(redoc_ui));

    // Combine all routes
    let router = Router::new()
        .merge(auth_routes)
        .merge(protected_auth_routes)
        .merge(mfa_routes)
        .merge(public_routes);

    tracing::info!("Web routes created successfully with complete authentication system");
    router
}


async fn health_check() -> &'static str {
    "OK"
}

async fn web_push_demo() -> Html<&'static str> {
    Html(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Web Push Notifications Demo - RustAxum</title>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; background: #f5f5f5; }
        .container { max-width: 800px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
        h1 { color: #2c3e50; margin-bottom: 30px; }
        .section { margin: 20px 0; padding: 20px; border: 1px solid #e1e1e1; border-radius: 5px; background: #fafafa; }
        button { background: #3498db; color: white; border: none; padding: 12px 24px; border-radius: 5px; cursor: pointer; margin: 5px; }
        button:hover { background: #2980b9; }
        button:disabled { background: #bdc3c7; cursor: not-allowed; }
        .status { margin: 10px 0; padding: 10px; border-radius: 5px; }
        .success { background: #d4edda; color: #155724; border: 1px solid #c3e6cb; }
        .error { background: #f8d7da; color: #721c24; border: 1px solid #f5c6cb; }
        .info { background: #d1ecf1; color: #0c5460; border: 1px solid #bee5eb; }
        .warning { background: #fff3cd; color: #856404; border: 1px solid #ffeaa7; }
        pre { background: #f8f9fa; padding: 15px; border-radius: 5px; overflow-x: auto; border: 1px solid #e9ecef; }
        .endpoint-info { font-family: monospace; background: #e9ecef; padding: 5px; border-radius: 3px; }
    </style>
</head>
<body>
    <div class="container">
        <h1>🔔 Web Push Notifications Demo</h1>

        <div class="section">
            <h3>Web Push Status</h3>
            <div id="support-status" class="status info">Checking browser support...</div>
            <div id="permission-status" class="status info">Checking permission status...</div>
            <div id="service-worker-status" class="status info">Checking service worker...</div>
        </div>

        <div class="section">
            <h3>Subscription Management</h3>
            <button id="subscribe-btn" onclick="subscribe()" disabled>Subscribe to Notifications</button>
            <button id="unsubscribe-btn" onclick="unsubscribe()" disabled>Unsubscribe</button>
            <button id="get-subscriptions-btn" onclick="getSubscriptions()" disabled>Get My Subscriptions</button>
            <div id="subscription-status" class="status info">Not subscribed</div>
        </div>

        <div class="section">
            <h3>Test Notifications</h3>
            <button id="test-notification-btn" onclick="sendTestNotification()" disabled>Send Test Notification</button>
            <div id="test-status" class="status info">Subscribe first to test notifications</div>
        </div>

        <div class="section">
            <h3>API Endpoints</h3>
            <p><strong>VAPID Public Key:</strong> <span class="endpoint-info">GET /api/web-push/vapid-public-key</span></p>
            <p><strong>Subscribe:</strong> <span class="endpoint-info">POST /api/web-push/subscribe</span></p>
            <p><strong>Unsubscribe:</strong> <span class="endpoint-info">DELETE /api/web-push/unsubscribe</span></p>
            <p><strong>Test:</strong> <span class="endpoint-info">POST /api/web-push/test</span></p>
            <p><strong>Status:</strong> <span class="endpoint-info">GET /api/web-push/status</span></p>
        </div>

        <div class="section">
            <h3>Configuration Status</h3>
            <button onclick="checkWebPushStatus()">Check Server Configuration</button>
            <div id="config-status" class="status info">Click to check server configuration</div>
        </div>

        <div class="section">
            <h3>Debug Info</h3>
            <button onclick="showDebugInfo()">Show Debug Information</button>
            <pre id="debug-info" style="display: none;"></pre>
        </div>
    </div>

    <script>
        let currentSubscription = null;
        let vapidPublicKey = null;

        // Initialize the demo
        document.addEventListener('DOMContentLoaded', function() {
            initializeWebPush();
        });

        async function initializeWebPush() {
            // Check browser support
            if ('serviceWorker' in navigator && 'PushManager' in window) {
                document.getElementById('support-status').innerHTML = '<span class="success">✅ Web Push supported</span>';

                // Register service worker
                try {
                    const registration = await navigator.serviceWorker.register('/sw.js');
                    document.getElementById('service-worker-status').innerHTML = '<span class="success">✅ Service Worker registered</span>';

                    // Check permission
                    checkPermissionStatus();

                    // Get VAPID public key
                    await getVapidPublicKey();

                    // Enable buttons
                    document.getElementById('subscribe-btn').disabled = false;
                    document.getElementById('get-subscriptions-btn').disabled = false;

                } catch (error) {
                    document.getElementById('service-worker-status').innerHTML = '<span class="error">❌ Service Worker registration failed: ' + error.message + '</span>';
                }
            } else {
                document.getElementById('support-status').innerHTML = '<span class="error">❌ Web Push not supported</span>';
            }
        }

        function checkPermissionStatus() {
            const permission = Notification.permission;
            let statusClass = 'info';
            let icon = 'ℹ️';

            if (permission === 'granted') {
                statusClass = 'success';
                icon = '✅';
            } else if (permission === 'denied') {
                statusClass = 'error';
                icon = '❌';
            } else {
                statusClass = 'warning';
                icon = '⚠️';
            }

            document.getElementById('permission-status').innerHTML =
                '<span class="' + statusClass + '">' + icon + ' Permission: ' + permission + '</span>';
        }

        async function getVapidPublicKey() {
            try {
                const response = await fetch('/api/web-push/vapid-public-key');
                if (response.ok) {
                    const data = await response.json();
                    vapidPublicKey = data.public_key;
                    console.log('VAPID public key received:', vapidPublicKey);
                } else {
                    throw new Error('Failed to get VAPID public key');
                }
            } catch (error) {
                console.error('Error getting VAPID public key:', error);
                document.getElementById('subscription-status').innerHTML =
                    '<span class="error">❌ Failed to get VAPID public key: ' + error.message + '</span>';
            }
        }

        async function subscribe() {
            try {
                const permission = await Notification.requestPermission();
                checkPermissionStatus();

                if (permission !== 'granted') {
                    throw new Error('Notification permission denied');
                }

                const registration = await navigator.serviceWorker.ready;
                const subscription = await registration.pushManager.subscribe({
                    userVisibleOnly: true,
                    applicationServerKey: urlBase64ToUint8Array(vapidPublicKey)
                });

                currentSubscription = subscription;

                // Send subscription to server
                const response = await fetch('/api/web-push/subscribe', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                        // Note: In a real app, you'd include authentication headers here
                    },
                    body: JSON.stringify({
                        endpoint: subscription.endpoint,
                        keys: {
                            p256dh: arrayBufferToBase64(subscription.getKey('p256dh')),
                            auth: arrayBufferToBase64(subscription.getKey('auth'))
                        }
                    })
                });

                if (response.ok) {
                    const data = await response.json();
                    document.getElementById('subscription-status').innerHTML =
                        '<span class="success">✅ Subscribed successfully! ID: ' + (data.subscription_id || 'N/A') + '</span>';
                    document.getElementById('unsubscribe-btn').disabled = false;
                    document.getElementById('test-notification-btn').disabled = false;
                    document.getElementById('test-status').innerHTML =
                        '<span class="info">Ready to send test notifications</span>';
                } else {
                    throw new Error('Server subscription failed');
                }

            } catch (error) {
                console.error('Subscription failed:', error);
                document.getElementById('subscription-status').innerHTML =
                    '<span class="error">❌ Subscription failed: ' + error.message + '</span>';
            }
        }

        async function unsubscribe() {
            try {
                if (currentSubscription) {
                    await currentSubscription.unsubscribe();

                    // Notify server
                    const response = await fetch('/api/web-push/unsubscribe', {
                        method: 'DELETE',
                        headers: {
                            'Content-Type': 'application/json',
                        },
                        body: JSON.stringify({
                            endpoint: currentSubscription.endpoint
                        })
                    });

                    currentSubscription = null;
                    document.getElementById('subscription-status').innerHTML =
                        '<span class="info">Unsubscribed successfully</span>';
                    document.getElementById('unsubscribe-btn').disabled = true;
                    document.getElementById('test-notification-btn').disabled = true;
                    document.getElementById('test-status').innerHTML =
                        '<span class="info">Subscribe first to test notifications</span>';
                }
            } catch (error) {
                console.error('Unsubscribe failed:', error);
                document.getElementById('subscription-status').innerHTML =
                    '<span class="error">❌ Unsubscribe failed: ' + error.message + '</span>';
            }
        }

        async function sendTestNotification() {
            try {
                const response = await fetch('/api/web-push/test?title=Test&message=Hello from RustAxum!', {
                    method: 'POST',
                    headers: {
                        // Note: In a real app, you'd include authentication headers here
                    }
                });

                if (response.ok) {
                    const data = await response.json();
                    document.getElementById('test-status').innerHTML =
                        '<span class="success">✅ Test notification sent: ' + data.message + '</span>';
                } else {
                    throw new Error('Test notification failed');
                }
            } catch (error) {
                console.error('Test notification failed:', error);
                document.getElementById('test-status').innerHTML =
                    '<span class="error">❌ Test notification failed: ' + error.message + '</span>';
            }
        }

        async function getSubscriptions() {
            try {
                const response = await fetch('/api/web-push/subscriptions');
                if (response.ok) {
                    const data = await response.json();
                    document.getElementById('subscription-status').innerHTML =
                        '<span class="info">Subscriptions: ' + data.subscriptions.length + '</span>';
                } else {
                    throw new Error('Failed to get subscriptions');
                }
            } catch (error) {
                console.error('Get subscriptions failed:', error);
                document.getElementById('subscription-status').innerHTML =
                    '<span class="error">❌ Failed to get subscriptions: ' + error.message + '</span>';
            }
        }

        async function checkWebPushStatus() {
            try {
                const response = await fetch('/api/web-push/status');
                if (response.ok) {
                    const data = await response.json();
                    const statusClass = data.configured ? 'success' : 'warning';
                    const icon = data.configured ? '✅' : '⚠️';
                    document.getElementById('config-status').innerHTML =
                        '<span class="' + statusClass + '">' + icon + ' ' + data.message + '</span>';
                } else {
                    throw new Error('Failed to check status');
                }
            } catch (error) {
                document.getElementById('config-status').innerHTML =
                    '<span class="error">❌ Status check failed: ' + error.message + '</span>';
            }
        }

        function showDebugInfo() {
            const debugInfo = {
                userAgent: navigator.userAgent,
                serviceWorkerSupport: 'serviceWorker' in navigator,
                pushManagerSupport: 'PushManager' in window,
                notificationPermission: Notification.permission,
                vapidPublicKey: vapidPublicKey,
                currentSubscription: currentSubscription ? {
                    endpoint: currentSubscription.endpoint,
                    keys: {
                        p256dh: currentSubscription.getKey('p256dh') ? 'present' : 'missing',
                        auth: currentSubscription.getKey('auth') ? 'present' : 'missing'
                    }
                } : null
            };

            document.getElementById('debug-info').textContent = JSON.stringify(debugInfo, null, 2);
            document.getElementById('debug-info').style.display = 'block';
        }

        // Utility functions
        function urlBase64ToUint8Array(base64String) {
            const padding = '='.repeat((4 - base64String.length % 4) % 4);
            const base64 = (base64String + padding)
                .replace(/\-/g, '+')
                .replace(/_/g, '/');

            const rawData = window.atob(base64);
            const outputArray = new Uint8Array(rawData.length);

            for (let i = 0; i < rawData.length; ++i) {
                outputArray[i] = rawData.charCodeAt(i);
            }
            return outputArray;
        }

        function arrayBufferToBase64(buffer) {
            const bytes = new Uint8Array(buffer);
            let binary = '';
            for (let i = 0; i < bytes.byteLength; i++) {
                binary += String.fromCharCode(bytes[i]);
            }
            return window.btoa(binary);
        }
    </script>
</body>
</html>
    "#)
}

async fn swagger_ui() -> Html<&'static str> {
    Html(r#"
<!DOCTYPE html>
<html>
<head>
    <title>RustAxum API Documentation - Swagger UI</title>
    <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@5.29.0/swagger-ui.css" />
    <style>
        html { box-sizing: border-box; overflow: -moz-scrollbars-vertical; overflow-y: scroll; }
        *, *:before, *:after { box-sizing: inherit; }
        body { margin:0; background: #fafafa; }
    </style>
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@5.29.0/swagger-ui-bundle.js"></script>
    <script src="https://unpkg.com/swagger-ui-dist@5.29.0/swagger-ui-standalone-preset.js"></script>
    <script>
        window.onload = function() {
            const ui = SwaggerUIBundle({
                url: '/api/docs/openapi.json',
                dom_id: '#swagger-ui',
                deepLinking: true,
                presets: [
                    SwaggerUIBundle.presets.apis,
                    SwaggerUIStandalonePreset
                ],
                plugins: [
                    SwaggerUIBundle.plugins.DownloadUrl
                ],
                layout: "StandaloneLayout"
            });
        };
    </script>
</body>
</html>
    "#)
}

async fn rapidoc_ui() -> Html<&'static str> {
    Html(r##"
<!DOCTYPE html>
<html>
<head>
    <title>RustAxum API Documentation - RapiDoc</title>
    <script type="module" src="https://unpkg.com/rapidoc@9.3.8/dist/rapidoc-min.js"></script>
</head>
<body>
    <rapi-doc
        spec-url="/api/docs/openapi.json"
        theme="light"
        render-style="read"
        nav-bg-color="#1f2937"
        primary-color="#3b82f6"
        show-header="true"
        show-info="true"
        allow-try="true"
        allow-server-selection="true"
        allow-authentication="true">
    </rapi-doc>
</body>
</html>
    "##)
}

async fn redoc_ui() -> Html<&'static str> {
    Html(r#"
<!DOCTYPE html>
<html>
<head>
    <title>RustAxum API Documentation - Redoc</title>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <link href="https://fonts.googleapis.com/css?family=Montserrat:300,400,700|Roboto:300,400,700" rel="stylesheet">
    <style>
        body { margin: 0; padding: 0; }
    </style>
</head>
<body>
    <redoc spec-url='/api/docs/openapi.json'></redoc>
    <script src="https://cdn.redoc.ly/redoc/latest/bundles/redoc.standalone.js"></script>
</body>
</html>
    "#)
}