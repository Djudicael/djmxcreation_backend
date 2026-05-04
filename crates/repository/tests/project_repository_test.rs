use app_core::dto::{content_dto::ContentDto, metadata_dto::MetadataDto, project_dto::ProjectDto};
use app_core::project::project_repository::IProjectRepository;
use repository::project_repository::ProjectRepository;
use serde_json::json;
use std::sync::Arc;
use test_util::shared_harness::shared_postgres;
use uuid::Uuid;

struct TestContext {
    repo: ProjectRepository,
    id: Uuid,
}

impl TestContext {
    async fn new() -> Self {
        let (config, _uri) = shared_postgres().await;
        let repo = ProjectRepository::new(config);

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

    assert_eq!(
        metadata.title.as_ref().expect("Title should exist"),
        "Test Project",
    );
    assert_eq!(
        metadata.sub_title.as_ref().expect("Subtitle should exist"),
        "Test Subtitle",
    );
    assert_eq!(
        metadata.client.as_ref().expect("Client should exist"),
        "Test Client",
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
        .contents(vec![]);

    ctx.repo
        .update_project_entity(ctx.id, &updated_project)
        .await
        .expect("Failed to update project");

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

#[tokio::test]
async fn test_project_complete_lifecycle() {
    let ctx = TestContext::new().await;

    let project2 = ctx
        .repo
        .create(&MetadataDto::new(
            Some("Second Project".to_string()),
            Some("Second Subtitle".to_string()),
            Some("Second Client".to_string()),
        ))
        .await
        .expect("Failed to create second project");

    let all_projects = ctx
        .repo
        .get_projects()
        .await
        .expect("Failed to get all projects");
    assert_eq!(all_projects.len(), 2, "Should have two projects");

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

    let content_by_id = ctx
        .repo
        .get_projects_content_by_id(ctx.id, content_result.id.unwrap())
        .await
        .expect("Failed to query content")
        .expect("Content should exist");

    assert_eq!(content_by_id.id, content_result.id);

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

    let thumbnails = ctx
        .repo
        .get_projects_content_thumbnail(ctx.id)
        .await
        .expect("Failed to get thumbnail");
    assert!(!thumbnails.is_empty());

    ctx.repo
        .delete_thumbnail_by_id(ctx.id, thumbnail_result.id.unwrap())
        .await
        .expect("Failed to delete thumbnail");

    let deleted_thumbnail = ctx
        .repo
        .get_thumbnail_by_id(ctx.id, thumbnail_result.id.unwrap())
        .await
        .expect("Failed to query deleted thumbnail");
    assert!(deleted_thumbnail.is_none());

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

    let non_existent_id = Uuid::new_v4();
    let result = ctx
        .repo
        .get_project_by_id(non_existent_id)
        .await
        .expect("Failed to query project");
    assert!(result.is_none());

    let result = ctx
        .repo
        .get_projects_content_by_id(ctx.id, Uuid::new_v4())
        .await
        .expect("Failed to query content");
    assert!(result.is_none());

    let result = ctx
        .repo
        .get_thumbnail_by_id(ctx.id, Uuid::new_v4())
        .await
        .expect("Failed to query thumbnail");
    assert!(result.is_none());
}
