//! OpenAPI Schema Generator
//!
//! This binary generates the OpenAPI 3.0 schema from the annotated API code.
//! The generated schema is used for:
//! - TypeScript client generation
//! - API documentation
//! - Contract validation
//!
//! Usage:
//! ```bash
//! cargo run --bin generate-openapi > openapi.json
//! ```

use project_starter_api::ApiDoc;
use utoipa::OpenApi;

fn main() {
    // Generate OpenAPI schema
    let openapi = ApiDoc::openapi();

    // Output as pretty-printed JSON
    let json = openapi
        .to_pretty_json()
        .expect("Failed to serialize OpenAPI schema to JSON");

    println!("{}", json);
}
