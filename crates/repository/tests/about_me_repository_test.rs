use app_config::database_configuration::DatabaseConfiguration;
use app_core::about_me::about_me_repository::IAboutMeRepository;
use app_core::dto::about_me_dto::AboutMeDto;
use app_core::dto::content_dto::ContentDto;
use repository::about_me_repository::AboutMeRepository;
use repository::config::db::DatabasePool;
use test_util::postgresql::PostgresContainer;

use serde_json::json;
use std::sync::Arc;
use test_util::postgresql::init_postgresql;
use uuid::Uuid;

struct TestContext {
    repo: AboutMeRepository,
    id: Uuid,
    _container: PostgresContainer, // Keep container alive
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

        let repo = AboutMeRepository::new(Arc::new(pool));
        let id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").expect("uuid parse error");

        // Initialize test data
        repo.update_about_me(
            id,
            &AboutMeDto {
                id: None,
                first_name: "Test".to_string(),
                last_name: "User".to_string(),
                description: Some(json!({"bio": "Tester"})),
                photo: None,
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
async fn test_about_me_crud_operations() {
    let ctx = TestContext::new().await;

    // Test 1: Get initial about me
    let about_me = ctx
        .repo
        .get_about_me_by_id(ctx.id)
        .await
        .expect("Failed to get initial about_me");
    assert_eq!(about_me.first_name, "Test");
    assert_eq!(about_me.last_name, "User");

    // Test 2: Update about me
    let updated = ctx
        .repo
        .update_about_me(
            ctx.id,
            &AboutMeDto {
                id: None,
                first_name: "Alice".to_string(),
                last_name: "Doe".to_string(),
                description: Some(json!({"bio": "Artist"})),
                photo: None,
            },
        )
        .await
        .expect("Failed to update about_me");
    assert_eq!(updated.first_name, "Alice");
    assert_eq!(updated.last_name, "Doe");

    // Test 3: Update photo
    let content = ContentDto {
        id: None,
        bucket_name: "test_bucket".to_string(),
        file_name: "test_file".to_string(),
        mime_type: None,
    };
    ctx.repo
        .update_photo(ctx.id, &content)
        .await
        .expect("Failed to update photo");

    let with_photo = ctx
        .repo
        .get_about_me_by_id(ctx.id)
        .await
        .expect("Failed to get about_me with photo");
    assert!(with_photo.photo.is_some());

    // Test 4: Delete photo
    ctx.repo
        .delete_about_me_photo(ctx.id)
        .await
        .expect("Failed to delete photo");

    let without_photo = ctx
        .repo
        .get_about_me_by_id(ctx.id)
        .await
        .expect("Failed to get about_me without photo");
    assert!(without_photo.photo.is_none());

    // Test 5: Get about me (general)
    let general = ctx
        .repo
        .get_about_me()
        .await
        .expect("Failed to get about_me");
    assert!(!general.first_name.is_empty());
    assert!(!general.last_name.is_empty());
}
