use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;
use uuid::Uuid;

/// An item in the system
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
#[schema(example = json!({
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "Example Item",
    "description": "This is an example item",
    "created_at": "2024-01-01T12:00:00Z"
}))]
pub struct Item {
    /// Unique identifier for the item
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: Uuid,

    /// Name of the item (max 100 characters)
    #[schema(example = "Example Item")]
    pub name: String,

    /// Optional description of the item (max 500 characters)
    #[schema(example = "This is an example item")]
    pub description: Option<String>,

    /// Timestamp when the item was created
    #[schema(example = "2024-01-01T12:00:00Z")]
    pub created_at: DateTime<Utc>,
}

/// Request payload for creating a new item
#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({
    "name": "New Item",
    "description": "Optional description"
}))]
pub struct CreateItemRequest {
    /// Name of the item (required, max 100 characters)
    #[schema(example = "New Item", min_length = 1, max_length = 100)]
    pub name: String,

    /// Optional description (max 500 characters)
    #[schema(example = "Optional description", max_length = 500)]
    pub description: Option<String>,
}

impl CreateItemRequest {
    /// Validate the create item request
    ///
    /// # Errors
    /// Returns an error if validation fails
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Item name cannot be empty".to_string());
        }

        if self.name.len() > 100 {
            return Err("Item name cannot exceed 100 characters".to_string());
        }

        if let Some(ref desc) = self.description {
            if desc.len() > 500 {
                return Err("Item description cannot exceed 500 characters".to_string());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_empty_name() {
        let req = CreateItemRequest {
            name: "".to_string(),
            description: None,
        };

        assert!(req.validate().is_err());
    }

    #[test]
    fn test_validate_name_too_long() {
        let req = CreateItemRequest {
            name: "a".repeat(101),
            description: None,
        };

        assert!(req.validate().is_err());
    }

    #[test]
    fn test_validate_description_too_long() {
        let req = CreateItemRequest {
            name: "Valid Name".to_string(),
            description: Some("a".repeat(501)),
        };

        assert!(req.validate().is_err());
    }

    #[test]
    fn test_validate_valid_request() {
        let req = CreateItemRequest {
            name: "Valid Item".to_string(),
            description: Some("Valid description".to_string()),
        };

        assert!(req.validate().is_ok());
    }
}
