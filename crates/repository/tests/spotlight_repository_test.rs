use app_config::database_configuration::DatabaseConfiguration;
use app_core::dto::project_dto::ProjectDto;
use app_core::dto::spotlight_dto::SpotlightDto;
use app_core::spotlight::spotlight_repository::ISpotlightRepository;
use repository::config::db::DatabasePool;
use repository::project_repository::ProjectRepository;
use repository::spotlight_repository::SpotlightRepository;
use serde_json::json;
use std::sync::Arc;
use test_util::postgresql::{init_postgresql, PostgresContainer};
use uuid::Uuid;

struct TestContext {
    repo: SpotlightRepository,
    project_repo: ProjectRepository,
    id: Uuid,
    project_id: Uuid,
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

        let pool = Arc::new(
            DatabasePool::new(&test_db_config, Some(&url))
                .await
                .expect("Failed to create database pool"),
        );

        let repo = SpotlightRepository::new(pool.clone());
        let project_repo = ProjectRepository::new(pool);
        // Create a project first
        let project = ProjectDto::new()
            .metadata(Some(json!({
                "title": "Test Project",
                "sub_title": "Test Subtitle",
                "client": "Test Client"
            })))
            .description(Some(json!({"details": "Test description"})))
            .visible(true)
            .adult(false)
            .build();

        let created_project = project_repo
            .create_project(&project)
            .await
            .expect("Failed to create project");

        let project_id = created_project.id.expect("Project should have an ID");

        // Then create the spotlight
        let spotlight = repo
            .add_spotlight(project_id)
            .await
            .expect("Failed to create initial spotlight");

        Self {
            repo,
            project_repo,
            id: spotlight.id.expect("Spotlight should have an ID"),
            project_id,
            _container: container,
        }
    }
}

#[tokio::test]
async fn test_spotlight_crud_operations() {
    let ctx = TestContext::new().await;

    // Test 1: Get initial spotlight
    let spotlight = ctx
        .repo
        .get_spotlight(ctx.id)
        .await
        .expect("Failed to get spotlight")
        .expect("Spotlight should exist");

    assert_eq!(spotlight.project_id, ctx.project_id);
    assert!(spotlight.created_on.is_some());
    assert!(!spotlight.adult);
    assert!(spotlight.metadata.is_none());
    assert!(spotlight.thumbnail.is_none());

    // Test 2: Get all spotlights
    let spotlights = ctx
        .repo
        .get_spotlights()
        .await
        .expect("Failed to get spotlights");

    assert_eq!(spotlights.len(), 1);
    assert_eq!(spotlights[0].id, spotlight.id);

    // Test 3: Delete spotlight
    ctx.repo
        .delete_spotlight(ctx.id)
        .await
        .expect("Failed to delete spotlight");

    // Test 4: Verify deletion
    let deleted_spotlight = ctx
        .repo
        .get_spotlight(ctx.id)
        .await
        .expect("Failed to query spotlight");

    assert!(deleted_spotlight.is_none());

    // Test 5: Add new spotlight with new project
    let new_project = ProjectDto::new()
        .metadata(Some(json!({
            "title": "New Test Project",
            "sub_title": "New Test Subtitle",
            "client": "New Test Client"
        })))
        .description(Some(json!({"details": "New test description"})))
        .visible(true)
        .adult(false)
        .build();

    let created_project = ctx
        .project_repo
        .create_project(&new_project)
        .await
        .expect("Failed to create new project");

    let new_project_id = created_project.id.expect("Project should have an ID");

    let new_spotlight = ctx
        .repo
        .add_spotlight(new_project_id)
        .await
        .expect("Failed to create new spotlight");

    assert_eq!(new_spotlight.project_id, new_project_id);
    assert!(new_spotlight.created_on.is_some());
    assert!(!new_spotlight.adult);

    // Test 6: Try to add spotlight for non-existent project
    let non_existent_id = Uuid::new_v4();
    let error = ctx
        .repo
        .add_spotlight(non_existent_id)
        .await
        .expect_err("Should fail with foreign key violation");

    match error {
        Error::InvalidInput(_) => (),
        _ => panic!("Expected InvalidInput error for foreign key violation"),
    }
}
