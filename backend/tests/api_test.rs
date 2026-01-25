use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::sleep;

const BASE_URL: &str = "http://localhost:3000";

/// Helper to wait for server to be ready
async fn wait_for_server() {
    let client = Client::new();
    for _ in 0..30 {
        if client
            .get(&format!("{}/health", BASE_URL))
            .send()
            .await
            .is_ok()
        {
            return;
        }
        sleep(Duration::from_millis(100)).await;
    }
    panic!("Server did not become ready in time");
}

#[tokio::test]
#[ignore] // Run only when server is running: cargo test --test api_test -- --ignored
async fn test_health_endpoint() {
    wait_for_server().await;

    let client = Client::new();
    let response = client
        .get(&format!("{}/health", BASE_URL))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::OK);

    let body: Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["status"], "healthy");
    assert!(body["service"].is_string());
    assert!(body["version"].is_string());
}

#[tokio::test]
#[ignore] // Run only when server is running: cargo test --test api_test -- --ignored
async fn test_readiness_endpoint() {
    wait_for_server().await;

    let client = Client::new();
    let response = client
        .get(&format!("{}/health/ready", BASE_URL))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::OK);

    let body: Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["status"], "ready");
    assert_eq!(body["checks"]["database"], "healthy");
}

#[tokio::test]
#[ignore] // Run only when server is running: cargo test --test api_test -- --ignored
async fn test_create_and_retrieve_item() {
    wait_for_server().await;

    let client = Client::new();

    // Create an item
    let create_payload = json!({
        "name": "Test Item",
        "description": "Created by API test"
    });

    let create_response = client
        .post(&format!("{}/api/items", BASE_URL))
        .json(&create_payload)
        .send()
        .await
        .expect("Failed to create item");

    assert_eq!(create_response.status(), StatusCode::CREATED);

    let created_item: Value = create_response
        .json()
        .await
        .expect("Failed to parse created item");

    assert_eq!(created_item["name"], "Test Item");
    assert_eq!(created_item["description"], "Created by API test");
    assert!(created_item["id"].is_string());
    assert!(created_item["created_at"].is_string());

    let item_id = created_item["id"].as_str().unwrap();

    // Retrieve the item
    let get_response = client
        .get(&format!("{}/api/items/{}", BASE_URL, item_id))
        .send()
        .await
        .expect("Failed to get item");

    assert_eq!(get_response.status(), StatusCode::OK);

    let retrieved_item: Value = get_response
        .json()
        .await
        .expect("Failed to parse retrieved item");

    assert_eq!(retrieved_item["id"], item_id);
    assert_eq!(retrieved_item["name"], "Test Item");
}

#[tokio::test]
#[ignore] // Run only when server is running: cargo test --test api_test -- --ignored
async fn test_list_items() {
    wait_for_server().await;

    let client = Client::new();

    // Create a few items
    for i in 1..=3 {
        let payload = json!({
            "name": format!("Test Item {}", i),
            "description": format!("Description {}", i)
        });

        client
            .post(&format!("{}/api/items", BASE_URL))
            .json(&payload)
            .send()
            .await
            .expect("Failed to create item");
    }

    // List items
    let response = client
        .get(&format!("{}/api/items", BASE_URL))
        .send()
        .await
        .expect("Failed to list items");

    assert_eq!(response.status(), StatusCode::OK);

    let items: Value = response.json().await.expect("Failed to parse items");
    assert!(items.is_array());
    assert!(items.as_array().unwrap().len() >= 3);
}

#[tokio::test]
#[ignore] // Run only when server is running: cargo test --test api_test -- --ignored
async fn test_get_nonexistent_item() {
    wait_for_server().await;

    let client = Client::new();
    let fake_id = "550e8400-e29b-41d4-a716-446655440000";

    let response = client
        .get(&format!("{}/api/items/{}", BASE_URL, fake_id))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore] // Run only when server is running: cargo test --test api_test -- --ignored
async fn test_create_item_with_invalid_data() {
    wait_for_server().await;

    let client = Client::new();

    // Empty name
    let payload = json!({
        "name": "",
        "description": "Test"
    });

    let response = client
        .post(&format!("{}/api/items", BASE_URL))
        .json(&payload)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Name too long
    let payload = json!({
        "name": "a".repeat(101),
        "description": "Test"
    });

    let response = client
        .post(&format!("{}/api/items", BASE_URL))
        .json(&payload)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
