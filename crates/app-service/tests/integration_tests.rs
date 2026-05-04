use app_core::about_me::about_me_service::IAboutMeService;
use app_core::contact::contact_service::IContactService;
use app_core::dto::{
    about_me_dto::AboutMeDto, contact_dto::ContactDto, metadata_dto::MetadataDto,
};
use app_core::project::project_service::IProjectService;
use app_service::{
    about_me_service::AboutMeService, contact_service::ContactService,
    project_service::ProjectService,
};
use repository::{
    about_me_repository::AboutMeRepository, contact_repository::ContactRepository,
    project_repository::ProjectRepository, spotlight_repository::SpotlightRepository,
    storage_repository::StorageRepository,
};
use std::sync::Arc;
use test_util::shared_harness::shared_postgres;
use uuid::Uuid;

struct ServiceTestContext {
    about_me: AboutMeService,
    contact: ContactService,
    project: ProjectService,
    about_me_id: Uuid,
    contact_id: Uuid,
}

impl ServiceTestContext {
    async fn new() -> Self {
        let (db_config, _uri) = shared_postgres().await;

        let about_me_repo = Arc::new(AboutMeRepository::new(db_config.clone()));
        let contact_repo = Arc::new(ContactRepository::new(db_config.clone()));
        let project_repo = Arc::new(ProjectRepository::new(db_config.clone()));
        let spotlight_repo = Arc::new(SpotlightRepository::new(db_config.clone()));
        // Use a fake storage for service tests (real storage tested separately)
        let storage_repo = Arc::new(StorageRepository::new(fake_storage_client()));

        let about_me_id =
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").expect("uuid parse error");
        let contact_id = about_me_id;

        Self {
            about_me: AboutMeService::new(about_me_repo, storage_repo.clone(), "test-bucket".to_string()),
            contact: ContactService::new(contact_repo),
            project: ProjectService::new(
                project_repo,
                storage_repo,
                spotlight_repo,
                "test-bucket".to_string(),
            ),
            about_me_id,
            contact_id,
        }
    }
}

fn fake_storage_client() -> repository::config::storage::StorageClient {
    // For service integration tests we focus on DB logic; storage interactions
    // are covered by unit tests with fakes and by repository storage tests.
    use aws_sdk_s3::config::{Builder as S3ConfigBuilder, Credentials, Region};
    use aws_sdk_s3::Client;

    let credentials = Credentials::new("x", "x", None, None, "Static");
    let config = S3ConfigBuilder::new()
        .credentials_provider(credentials)
        .region(Region::new("us-east-1"))
        .endpoint_url("http://localhost:9000")
        .force_path_style(true)
        .build();
    repository::config::storage::StorageClient {
        inner: Client::from_conf(config),
    }
}

#[tokio::test]
async fn about_me_service_round_trip() {
    let ctx = ServiceTestContext::new().await;

    let me = ctx.about_me.about_me().await.expect("should get about_me");
    assert_eq!(me.first_name, "Your");
    assert_eq!(me.last_name, "Name");

    let updated = ctx
        .about_me
        .update_me(
            ctx.about_me_id,
            &AboutMeDto::new(
                None,
                "Grace".to_string(),
                "Hopper".to_string(),
                Some(serde_json::json!({"role": "Computer Scientist"})),
                None,
            ),
        )
        .await
        .expect("should update about_me");
    assert_eq!(updated.first_name, "Grace");
    assert_eq!(updated.last_name, "Hopper");
}

#[tokio::test]
async fn contact_service_round_trip() {
    let ctx = ServiceTestContext::new().await;

    let contact = ctx.contact.get_contact().await.expect("should get contact");
    assert!(contact.description.is_none());

    let updated = ctx
        .contact
        .update_contact(
            ctx.contact_id,
            &ContactDto {
                id: None,
                description: Some(serde_json::json!({
                    "email": "test@example.com",
                    "phone": "+1234567890"
                })),
            },
        )
        .await
        .expect("should update contact");
    assert!(updated.description.is_some());
}

#[tokio::test]
async fn project_service_lifecycle() {
    let ctx = ServiceTestContext::new().await;

    // Create
    let project = ctx
        .project
        .create_project(&MetadataDto::new(
            Some("Integration Test Project".to_string()),
            Some("Subtitle".to_string()),
            Some("Client".to_string()),
        ))
        .await
        .expect("should create project");
    let project_id = project.id.expect("project should have id");

    // Find
    let found = ctx
        .project
        .find_project(project_id)
        .await
        .expect("should find project");
    assert_eq!(found.id, Some(project_id));

    // Update
    ctx.project
        .update_project(
            project_id,
            &app_core::dto::project_dto::ProjectDto::new()
                .id(Some(project_id))
                .metadata(Some(MetadataDto::new(
                    Some("Updated".to_string()),
                    None,
                    None,
                )))
                .visible(true)
                .adult(false),
        )
        .await
        .expect("should update project");

    // Portfolio list
    let portfolio = ctx
        .project
        .get_portfolio_projects()
        .await
        .expect("should list portfolio");
    assert!(!portfolio.is_empty());

    // Filtered list
    let filtered = ctx
        .project
        .get_projects_with_filter(1, 10, None, true)
        .await
        .expect("should filter projects");
    assert!(!filtered.projects.is_empty());

    // Delete
    ctx.project
        .delete_project(project_id)
        .await
        .expect("should delete project");

    let not_found = ctx.project.find_project(project_id).await;
    assert!(not_found.is_err() || not_found.unwrap().id.is_none());
}

#[tokio::test]
async fn spotlight_service_flow() {
    let ctx = ServiceTestContext::new().await;

    // Create a project first
    let project = ctx
        .project
        .create_project(&MetadataDto::new(
            Some("Spotlight Test".to_string()),
            Some("Sub".to_string()),
            Some("Cli".to_string()),
        ))
        .await
        .expect("should create project");
    let project_id = project.id.unwrap();

    // Add spotlight
    let spotlight = ctx
        .project
        .add_spotlight(project_id)
        .await
        .expect("should add spotlight");
    let spotlight_id = spotlight.id.expect("spotlight should have id");

    // Get spotlight
    let found = ctx
        .project
        .get_spotlight(spotlight_id)
        .await
        .expect("should get spotlight");
    assert_eq!(found.id, Some(spotlight_id));

    // List spotlights
    let spotlights = ctx
        .project
        .get_spotlights()
        .await
        .expect("should list spotlights");
    assert!(!spotlights.is_empty());

    // Delete spotlight
    ctx.project
        .delete_spotlight(spotlight_id)
        .await
        .expect("should delete spotlight");

    // Cleanup project
    ctx.project
        .delete_project(project_id)
        .await
        .expect("should delete project");
}
