use app_config::database_configuration::DatabaseConfiguration;
use app_core::dto::{content_dto::ContentDto, metadata_dto::MetadataDto, project_dto::ProjectDto};
use app_core::project::project_repository::IProjectRepository;
use repository::config::db::db_client;
use repository::project_repository::ProjectRepository;
use serde_json::json;
use std::sync::Arc;
use test_util::postgresql::{init_postgresql, PostgresContainer};
use tokio::sync::Mutex;
use uuid::Uuid;

struct TestContext {
    repo: ProjectRepository,
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
        };

        let (podman, image) = init_postgresql(&test_db_config).expect("Failed to init PostgreSQL");
        let container = podman.start(image).await.expect("Failed to run PostgreSQL");
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let url = container.url().await.expect("Failed to get container URL");
        println!("Container started: {:?}", url);

        let client = db_client(&test_db_config, Some(&url))
            .await
            .expect("Failed to connect to the test database");

        let repo = ProjectRepository::new(Arc::new(Mutex::new(client)));
        let id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").expect("uuid parse error");

        // Initialize test data
        let project = repo
            .create(&MetadataDto::new(
                Some("Test Project".to_string()),
                Some("Test Subtitle".to_string()),
                Some("Test Client".to_string()),
            ))
            .await
            .expect("Failed to create test project");

        Self {
            repo,
            id: project.id.unwrap(),
            _container: container,
        }
    }
}

#[tokio::test]
async fn test_project_crud_operations() {
    let ctx = TestContext::new().await;

    // Test 1: Get initial project
    let project = ctx
        .repo
        .get_project_by_id(ctx.id)
        .await
        .expect("Failed to get project");
    let metadata = project.metadata.expect("Project metadata should exist");

    // Assert title
    assert_eq!(
        metadata.title.as_ref().expect("Title should exist"),
        "Test Project",
        "Project title should match the initial value"
    );

    // Assert subtitle
    assert_eq!(
        metadata.sub_title.as_ref().expect("Subtitle should exist"),
        "Test Subtitle",
        "Project subtitle should match the initial value"
    );

    // Assert client
    assert_eq!(
        metadata.client.as_ref().expect("Client should exist"),
        "Test Client",
        "Project client should match the initial value"
    );

    // Test 2: Update project
    let updated_metadata = MetadataDto::new(
        Some("Updated Project".to_string()),
        Some("Updated Subtitle".to_string()),
        Some("Updated Client".to_string()),
    );

    let updated_project = ProjectDto::new()
        .id(Some(ctx.id))
        .metadata(Some(updated_metadata))
        .description(Some(json!({"details": "Test description"})))
        .visible(true)
        .adult(false)
        .contents(vec![])
        .build();

    // Perform update
    ctx.repo
        .update_project_entity(ctx.id, &updated_project)
        .await
        .expect("Failed to update project");

    // Fetch updated project to verify changes
    let updated = ctx
        .repo
        .get_project_by_id(ctx.id)
        .await
        .expect("Failed to get updated project");

    let updated_metadata = updated.metadata.expect("Updated metadata should exist");
    assert_eq!(
        updated_metadata.title.as_ref().expect("Title should exist"),
        "Updated Project",
        "Project title should match updated value"
    );
    assert_eq!(
        updated_metadata
            .sub_title
            .as_ref()
            .expect("Subtitle should exist"),
        "Updated Subtitle",
        "Project subtitle should match updated value"
    );
    assert_eq!(
        updated_metadata
            .client
            .as_ref()
            .expect("Client should exist"),
        "Updated Client",
        "Project client should match updated value"
    );

    // Test 3: Add project content
    let content = ContentDto {
        id: None,
        bucket_name: "test_bucket".to_string(),
        file_name: "test_file.jpg".to_string(),
        mime_type: Some("image/jpeg".to_string()),
    };

    let content_result = ctx
        .repo
        .add_project_content(ctx.id, &content)
        .await
        .expect("Failed to add content");
    assert!(content_result.content.is_some());

    // Test 4: Add thumbnail
    let thumbnail = ContentDto {
        id: None,
        bucket_name: "thumbnails".to_string(),
        file_name: "thumb.jpg".to_string(),
        mime_type: Some("image/jpeg".to_string()),
    };

    let thumbnail_result = ctx
        .repo
        .add_project_thumbnail(ctx.id, &thumbnail)
        .await
        .expect("Failed to add thumbnail");
    assert!(thumbnail_result.content.is_some());

    // Test 5: Get projects with filter
    let projects = ctx
        .repo
        .get_projects_with_filter(1, 10, None, true)
        .await
        .expect("Failed to get filtered projects");
    assert!(!projects.projects.is_empty());

    // Test 6: Delete content
    let contents = ctx
        .repo
        .get_projects_contents(ctx.id)
        .await
        .expect("Failed to get contents");

    for content in contents {
        ctx.repo
            .delete_project_content_by_id(ctx.id, content.id.unwrap())
            .await
            .expect("Failed to delete content");
    }

    // Test 7: Delete project
    ctx.repo
        .delete_project_by_id(ctx.id)
        .await
        .expect("Failed to delete project");
}
