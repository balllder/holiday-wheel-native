//! API Documentation module
//!
//! Serves OpenAPI specification and Swagger UI at /docs

use axum::{
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};

/// The OpenAPI specification YAML content
const OPENAPI_YAML: &str = include_str!("../openapi.yaml");

/// Swagger UI HTML template
const SWAGGER_UI_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Holiday Wheel API Documentation</title>
    <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5.11.0/swagger-ui.css" />
    <style>
        html { box-sizing: border-box; overflow-y: scroll; }
        *, *:before, *:after { box-sizing: inherit; }
        body { margin: 0; background: #fafafa; }
        .swagger-ui .topbar { display: none; }
        .swagger-ui .info { margin: 20px 0; }
        .swagger-ui .info .title { font-size: 2em; }
    </style>
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@5.11.0/swagger-ui-bundle.js"></script>
    <script>
        window.onload = function() {
            SwaggerUIBundle({
                url: "/docs/openapi.yaml",
                dom_id: '#swagger-ui',
                deepLinking: true,
                presets: [
                    SwaggerUIBundle.presets.apis,
                    SwaggerUIBundle.SwaggerUIStandalonePreset
                ],
                layout: "StandaloneLayout",
                defaultModelsExpandDepth: 1,
                defaultModelExpandDepth: 1,
                docExpansion: "list",
                filter: true,
                showExtensions: true,
                showCommonExtensions: true,
                tryItOutEnabled: true
            });
        };
    </script>
</body>
</html>"#;

/// Serve the Swagger UI HTML page
async fn swagger_ui() -> Html<&'static str> {
    Html(SWAGGER_UI_HTML)
}

/// Serve the OpenAPI YAML specification
async fn openapi_yaml() -> Response {
    ([("content-type", "application/yaml")], OPENAPI_YAML).into_response()
}

/// Serve the OpenAPI JSON specification (converted from YAML)
async fn openapi_json() -> Response {
    // Parse YAML and convert to JSON
    match serde_yaml_to_json(OPENAPI_YAML) {
        Ok(json) => ([("content-type", "application/json")], json).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to convert OpenAPI spec: {}", e),
        )
            .into_response(),
    }
}

/// Convert YAML to JSON string
fn serde_yaml_to_json(_yaml: &str) -> Result<String, String> {
    // Swagger UI natively supports YAML, so JSON conversion is optional
    // If needed, add serde_yaml dependency and implement conversion
    Err("JSON conversion not available - use YAML endpoint".to_string())
}

/// ReDoc HTML template (alternative to Swagger UI)
const REDOC_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Holiday Wheel API Documentation - ReDoc</title>
    <link href="https://fonts.googleapis.com/css?family=Montserrat:300,400,700|Roboto:300,400,700" rel="stylesheet">
    <style>
        body { margin: 0; padding: 0; }
    </style>
</head>
<body>
    <redoc spec-url='/docs/openapi.yaml'></redoc>
    <script src="https://cdn.redoc.ly/redoc/latest/bundles/redoc.standalone.js"></script>
</body>
</html>"#;

/// Serve ReDoc UI (alternative documentation viewer)
async fn redoc_ui() -> Html<&'static str> {
    Html(REDOC_HTML)
}

/// Create the documentation router
pub fn routes() -> Router {
    Router::new()
        .route("/", get(swagger_ui))
        .route("/openapi.yaml", get(openapi_yaml))
        .route("/openapi.json", get(openapi_json))
        .route("/redoc", get(redoc_ui))
}
