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

    // test for non-existent project
    let non_existent_id = Uuid::new_v4();
    let non_existent_project = ctx
        .repo
        .get_project_by_id(non_existent_id)
        .await
        .expect("Failed to query non-existent project");

    assert!(
        non_existent_project.is_none(),
        "Non-existent project should return None"
    );

    // Test 1: Get initial project
    let project = ctx
        .repo
        .get_project_by_id(ctx.id)
        .await
        .expect("Failed to query project")
        .expect("Project should exist");

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
        .expect("Failed to query updated project")
        .expect("Updated project should exist");

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
    println!("Thumbnail: {:?}", thumbnail_result);
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

#[tokio::test]
async fn test_project_complete_lifecycle() {
    let ctx = TestContext::new().await;

    // Test 1: Create and verify multiple projects
    let project2 = ctx
        .repo
        .create(&MetadataDto::new(
            Some("Second Project".to_string()),
            Some("Second Subtitle".to_string()),
            Some("Second Client".to_string()),
        ))
        .await
        .expect("Failed to create second project");

    // Test 2: Get all projects
    let all_projects = ctx
        .repo
        .get_projects()
        .await
        .expect("Failed to get all projects");
    assert_eq!(all_projects.len(), 2, "Should have two projects");

    // Test 3: Test filtering with different parameters
    let adult_projects = ctx
        .repo
        .get_projects_with_filter(1, 10, Some(true), true)
        .await
        .expect("Failed to get adult projects");
    println!("Adult projects: {:?}", adult_projects);

    let non_adult_projects = ctx
        .repo
        .get_projects_with_filter(1, 10, Some(false), true)
        .await
        .expect("Failed to get non-adult projects");
    println!("Non-adult projects: {:?}", non_adult_projects);

    let hidden_projects = ctx
        .repo
        .get_projects_with_filter(1, 10, None, false)
        .await
        .expect("Failed to get hidden projects");
    println!("Hidden projects: {:?}", hidden_projects);

    // Test 4: Add and verify content
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

    // Test 5: Get specific content by ID
    let content_by_id = ctx
        .repo
        .get_projects_content_by_id(ctx.id, content_result.id.unwrap())
        .await
        .expect("Failed to query content")
        .expect("Content should exist");

    assert_eq!(
        content_by_id.id, content_result.id,
        "Content IDs should match"
    );

    // Add test for non-existent content
    let non_existent_id = Uuid::new_v4();
    let non_existent_content = ctx
        .repo
        .get_projects_content_by_id(ctx.id, non_existent_id)
        .await
        .expect("Failed to query non-existent content");

    assert!(
        non_existent_content.is_none(),
        "Non-existent content should return None"
    );

    // Test content for non-existent project
    let non_existent_project_id = Uuid::new_v4();
    let content_for_non_existent_project = ctx
        .repo
        .get_projects_content_by_id(non_existent_project_id, content_result.id.unwrap())
        .await
        .expect("Failed to query content for non-existent project");

    assert!(
        content_for_non_existent_project.is_none(),
        "Content for non-existent project should return None"
    );

    // Test 6: Add and verify thumbnail
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

    // Test 7: Get thumbnail
    let thumbnails = ctx
        .repo
        .get_projects_content_thumbnail(ctx.id)
        .await
        .expect("Failed to get thumbnail");
    assert!(!thumbnails.is_empty());

    // Test 8: Get specific thumbnail by ID
    let thumbnail_by_id = ctx
        .repo
        .get_thumbnail_by_id(ctx.id, thumbnail_result.id.unwrap())
        .await
        .expect("Failed to get thumbnail by ID");
    assert!(thumbnail_by_id.is_some());

    // Test 9: Delete thumbnail
    ctx.repo
        .delete_thumbnail_by_id(ctx.id, thumbnail_result.id.unwrap())
        .await
        .expect("Failed to delete thumbnail");

    // Test 10: Verify thumbnail deletion
    let deleted_thumbnail = ctx
        .repo
        .get_thumbnail_by_id(ctx.id, thumbnail_result.id.unwrap())
        .await
        .expect("Failed to query deleted thumbnail");

    assert!(
        deleted_thumbnail.is_none(),
        "Deleted thumbnail should return None"
    );

    // Additional test for non-existent thumbnail
    let non_existent_id = Uuid::new_v4();
    let non_existent_thumbnail = ctx
        .repo
        .get_thumbnail_by_id(ctx.id, non_existent_id)
        .await
        .expect("Failed to query non-existent thumbnail");

    assert!(
        non_existent_thumbnail.is_none(),
        "Non-existent thumbnail should return None"
    );

    // Clean up
    ctx.repo
        .delete_project_content_by_id(ctx.id, content_result.id.unwrap())
        .await
        .expect("Failed to delete content");

    ctx.repo
        .delete_project_by_id(ctx.id)
        .await
        .expect("Failed to delete first project");

    ctx.repo
        .delete_project_by_id(project2.id.unwrap())
        .await
        .expect("Failed to delete second project");
}

#[tokio::test]
async fn test_error_cases() {
    let ctx = TestContext::new().await;

    // Test non-existent project
    let non_existent_id = Uuid::new_v4();
    let result = ctx
        .repo
        .get_project_by_id(non_existent_id)
        .await
        .expect("Failed to query project");
    assert!(result.is_none());

    // Test invalid content ID
    let result = ctx
        .repo
        .get_projects_content_by_id(ctx.id, Uuid::new_v4())
        .await
        .expect("Failed to query content");
    assert!(result.is_none());

    // Test pagination edge cases
    let result = ctx
        .repo
        .get_projects_with_filter(0, 0, None, true)
        .await
        .expect("Failed with 0 page");
    assert!(result.projects.is_empty());

    // Test invalid thumbnail ID
    let result = ctx
        .repo
        .get_thumbnail_by_id(ctx.id, Uuid::new_v4())
        .await
        .expect("Failed to query thumbnail");
    assert!(result.is_none());
}

#[tokio::test]
async fn test_edge_cases() {
    let ctx = TestContext::new().await;

    // Test large page sizes
    let _ = ctx
        .repo
        .get_projects_with_filter(1, 1000, None, true)
        .await
        .expect("Failed with large page size");

    // Test updating with empty metadata
    let empty_project = ProjectDto::new()
        .id(Some(ctx.id))
        .metadata(Some(MetadataDto::new(None, None, None)))
        .build();

    ctx.repo
        .update_project_entity(ctx.id, &empty_project)
        .await
        .expect("Failed to update with empty metadata");
}
