use app_core::about_me::about_me_repository::IAboutMeRepository;
use app_core::dto::about_me_dto::AboutMeDto;
use app_core::dto::content_dto::ContentDto;
use repository::about_me_repository::AboutMeRepository;
use std::sync::Arc;
use test_util::shared_harness::shared_postgres;
use uuid::Uuid;

struct TestContext {
    repo: AboutMeRepository,
    id: Uuid,
}

impl TestContext {
    async fn new() -> Self {
        let id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").expect("uuid parse error");
        let (config, _uri) = shared_postgres().await;
        let repo = AboutMeRepository::new(config);
        Self { repo, id }
    }
}

#[tokio::test]
async fn test_about_me_crud_operations() {
    let ctx = TestContext::new().await;

    let about_me = ctx
        .repo
        .get_about_me_by_id(ctx.id)
        .await
        .expect("Failed to get initial about_me");
    assert_eq!(about_me.first_name, "Your");
    assert_eq!(about_me.last_name, "Name");

    let updated = ctx
        .repo
        .update_about_me(
            ctx.id,
            &AboutMeDto {
                id: None,
                first_name: "Alice".to_string(),
                last_name: "Doe".to_string(),
                description: Some(serde_json::json!({"bio": "Artist"})),
                photo: None,
            },
        )
        .await
        .expect("Failed to update about_me");
    assert_eq!(updated.first_name, "Alice");
    assert_eq!(updated.last_name, "Doe");

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

    let general = ctx
        .repo
        .get_about_me()
        .await
        .expect("Failed to get about_me");
    assert!(!general.first_name.is_empty());
    assert!(!general.last_name.is_empty());
}
