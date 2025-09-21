use axum::{routing::get, Router, response::Html};
use crate::database::DbPool;

pub fn routes() -> Router<DbPool> {
    tracing::debug!("Creating web routes...");
    let router = Router::new()
        .route("/", get(home))
        .route("/health", get(health_check))
        // Documentation UIs - custom HTML that references the OpenAPI endpoint
        .route("/docs/swagger", get(swagger_ui))
        .route("/docs/rapidoc", get(rapidoc_ui))
        .route("/docs/redoc", get(redoc_ui));

    tracing::info!("Web routes created successfully with working documentation UIs (Swagger, RapiDoc, Redoc)");
    router
}

async fn home() -> &'static str {
    "Welcome to RustAxum - A Laravel-inspired Rust web framework"
}

async fn health_check() -> &'static str {
    "OK"
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