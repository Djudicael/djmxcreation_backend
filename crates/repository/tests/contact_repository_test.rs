use app_core::contact::contact_repository::IContactRepository;
use app_core::dto::contact_dto::ContactDto;
use repository::contact_repository::ContactRepository;
use std::sync::Arc;
use test_util::shared_harness::shared_postgres;
use uuid::Uuid;

struct TestContext {
    repo: ContactRepository,
    id: Uuid,
}

impl TestContext {
    async fn new() -> Self {
        let (config, _uri) = shared_postgres().await;
        let id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").expect("uuid parse error");
        let repo = ContactRepository::new(config);
        Self { repo, id }
    }
}

#[tokio::test]
async fn test_contact_crud_operations() {
    let ctx = TestContext::new().await;

    // Test 1: Get initial contact (seeded by V5 migration)
    let contact = ctx
        .repo
        .get_contact()
        .await
        .expect("Failed to get initial contact");
    assert!(contact.description.is_none()); // V5 inserts NULL description

    // Test 2: Update contact
    let updated = ctx
        .repo
        .update_contact(
            ctx.id,
            &ContactDto {
                id: None,
                description: Some(serde_json::json!({
                    "email": "updated@example.com",
                    "phone": "+9876543210",
                    "linkedin": "https://linkedin.com/in/test"
                })),
            },
        )
        .await
        .expect("Failed to update contact");

    assert!(updated.description.is_some());

    // Test 3: Get updated contact
    let final_contact = ctx
        .repo
        .get_contact()
        .await
        .expect("Failed to get updated contact");
    assert!(final_contact.description.is_some());
}
