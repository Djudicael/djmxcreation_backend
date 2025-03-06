use app_config::database_configuration::DatabaseConfiguration;
use app_core::contact::contact_repository::IContactRepository;
use app_core::dto::contact_dto::ContactDto;
use repository::config::db::DatabasePool;
use repository::contact_repository::ContactRepository;
use serde_json::json;
use serde_json::Value;
use std::sync::Arc;
use test_util::postgresql::{init_postgresql, PostgresContainer};
use uuid::Uuid;

struct TestContext {
    repo: ContactRepository,
    id: Uuid,
    _container: PostgresContainer,
}

impl TestContext {
    async fn new() -> Self {
        let test_db_config = DatabaseConfiguration {
            pg_user: "postgres".to_string(),
            pg_password: "postgres".to_string(),
            pg_host: "localhost".to_string(),
            pg_db: "portfolio".to_string(),
            pg_app_max_con: 5,
            pg_port: 5432,
        };

        let (podman, image) = init_postgresql(&test_db_config).expect("Failed to init PostgreSQL");
        let container = podman.start(image).await.expect("Failed to run PostgreSQL");
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let url = container.url().await.expect("Failed to get container URL");
        println!("Container started: {:?}", url);

        // Create database pool with the test configuration and URL
        let pool = DatabasePool::new(&test_db_config, Some(&url))
            .await
            .expect("Failed to create database pool");

        let repo = ContactRepository::new(Arc::new(pool));
        let id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").expect("uuid parse error");

        // Initialize test data
        repo.update_contact(
            id,
            &ContactDto {
                id: None,
                description: Some(json!({
                    "email": "test@example.com",
                    "phone": "+1234567890"
                })),
            },
        )
        .await
        .expect("Failed to insert test data");

        Self {
            repo,
            id,
            _container: container,
        }
    }
}

#[tokio::test]
async fn test_contact_crud_operations() {
    let ctx = TestContext::new().await;

    // Test 1: Get initial contact
    let contact = ctx
        .repo
        .get_contact()
        .await
        .expect("Failed to get initial contact");
    assert!(contact.description.is_some());
    let desc = contact.description.as_ref().unwrap();
    let desc_value = match desc {
        Value::String(s) => serde_json::from_str::<Value>(s).expect("Failed to parse JSON string"),
        _ => desc.clone(),
    };
    assert_eq!(desc_value["email"].as_str().unwrap(), "test@example.com");
    assert_eq!(desc_value["phone"].as_str().unwrap(), "+1234567890");

    // Test 2: Update contact
    let updated = ctx
        .repo
        .update_contact(
            ctx.id,
            &ContactDto {
                id: None,
                description: Some(json!({
                    "email": "updated@example.com",
                    "phone": "+9876543210",
                    "linkedin": "https://linkedin.com/in/test"
                })),
            },
        )
        .await
        .expect("Failed to update contact");

    assert!(updated.description.is_some());
    let updated_desc = updated.description.as_ref().unwrap();
    let updated_desc_value = match updated_desc {
        Value::String(s) => serde_json::from_str::<Value>(s).expect("Failed to parse JSON string"),
        _ => desc.clone(),
    };

    assert_eq!(
        updated_desc_value["email"].as_str().unwrap(),
        "updated@example.com"
    );
    assert_eq!(updated_desc_value["phone"].as_str().unwrap(), "+9876543210");
    assert_eq!(
        updated_desc_value["linkedin"].as_str().unwrap(),
        "https://linkedin.com/in/test"
    );

    // Test 3: Get updated contact
    let final_contact = ctx
        .repo
        .get_contact()
        .await
        .expect("Failed to get updated contact");
    assert!(final_contact.description.is_some());
    let final_desc = final_contact.description.as_ref().unwrap();
    let final_desc_value = match final_desc {
        Value::String(s) => serde_json::from_str::<Value>(s).expect("Failed to parse JSON string"),
        _ => desc.clone(),
    };
    assert_eq!(
        final_desc_value["email"].as_str().unwrap(),
        "updated@example.com"
    );
    assert_eq!(final_desc_value["phone"].as_str().unwrap(), "+9876543210");
    assert_eq!(
        final_desc_value["linkedin"].as_str().unwrap(),
        "https://linkedin.com/in/test"
    );
}
