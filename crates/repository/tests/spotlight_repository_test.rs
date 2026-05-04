use app_core::dto::content_dto::ContentDto;
use app_core::dto::metadata_dto::MetadataDto;
use app_core::project::project_repository::IProjectRepository;
use app_core::spotlight::spotlight_repository::ISpotlightRepository;
use app_error::Error;
use repository::project_repository::ProjectRepository;
use repository::spotlight_repository::SpotlightRepository;
use std::sync::Arc;
use test_util::shared_harness::shared_postgres;
use uuid::Uuid;

struct TestContext {
    repo: SpotlightRepository,
    project_repo: ProjectRepository,
    id: Uuid,
    project_id: Uuid,
}

impl TestContext {
    async fn new() -> Self {
        let (config, _uri) = shared_postgres().await;

        let repo = SpotlightRepository::new(config.clone());
        let project_repo = ProjectRepository::new(config.clone());

        let created_project = project_repo
            .create(&MetadataDto::new(
                Some("Test Project".to_string()),
                Some("Test Subtitle".to_string()),
                Some("Test Client".to_string()),
            ))
            .await
            .expect("Failed to create test project");

        let project_id = created_project.id.expect("Project should have an ID");

        let content = ContentDto {
            id: None,
            bucket_name: "test_bucket".to_string(),
            file_name: "test_file.jpg".to_string(),
            mime_type: Some("image/jpeg".to_string()),
        };

        project_repo
            .add_project_content(project_id, &content)
            .await
            .expect("Failed to add content");
        project_repo
            .add_project_thumbnail(project_id, &content)
            .await
            .expect("Failed to add project thumbnail");

        let spotlight = repo
            .add_spotlight(project_id)
            .await
            .expect("Failed to create initial spotlight");

        Self {
            repo,
            project_repo,
            id: spotlight.id.expect("Spotlight should have an ID"),
            project_id,
        }
    }
}

#[tokio::test]
async fn test_spotlight_crud_operations() {
    let ctx = TestContext::new().await;

    let spotlight = ctx
        .repo
        .get_spotlight(ctx.id)
        .await
        .expect("Failed to get spotlight")
        .expect("Spotlight should exist");

    assert_eq!(spotlight.project_id, ctx.project_id);
    assert!(spotlight.created_on.is_some());
    assert!(!spotlight.adult);
    assert!(spotlight.metadata.is_some());
    assert!(spotlight.thumbnail.is_some());

    let spotlights = ctx
        .repo
        .get_spotlights()
        .await
        .expect("Failed to get spotlights");

    assert_eq!(spotlights.len(), 1);
    assert_eq!(spotlights[0].id, spotlight.id);

    ctx.repo
        .delete_spotlight(ctx.id)
        .await
        .expect("Failed to delete spotlight");

    let deleted_spotlight = ctx
        .repo
        .get_spotlight(ctx.id)
        .await
        .expect("Failed to query spotlight");

    assert!(deleted_spotlight.is_none());

    let metadata = MetadataDto::new(
        Some("New Test Project".to_string()),
        Some("New Test Subtitle".to_string()),
        Some("New Test Client".to_string()),
    );

    let created_project = ctx
        .project_repo
        .create(&metadata)
        .await
        .expect("Failed to create new project");

    let new_project_id = created_project.id.expect("Project should have an ID");

    let new_spotlight = ctx
        .repo
        .add_spotlight(new_project_id)
        .await
        .expect("Failed to create new spotlight");

    assert_eq!(new_spotlight.project_id, new_project_id);

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
