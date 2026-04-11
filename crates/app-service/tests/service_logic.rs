use app_core::about_me::about_me_service::IAboutMeService;
use app_core::dto::{
    about_me_dto::AboutMeDto, content_dto::ContentDto, project_content_dto::ProjectContentDto,
    project_dto::ProjectDto,
};
use app_core::project::project_service::IProjectService;
use app_service::{about_me_service::AboutMeService, project_service::ProjectService};
use repository::{
    fake_about_me_repository::create_about_me_repository,
    fake_project_repository::create_project_repository,
    fake_spotlight_repository::create_spotlight_repository,
    fake_storage_repository::create_storage_repository,
};
use uuid::Uuid;

fn run<F, T>(future: F) -> T
where
    F: std::future::Future<Output = T>,
{
    futures::executor::block_on(future)
}

#[test]
fn about_me_resolves_photo_url_from_storage() {
    let about_me_id = Uuid::from_u128(1);
    let about_me = AboutMeDto::new(
        Some(about_me_id),
        "Ada".to_string(),
        "Lovelace".to_string(),
        None,
        Some(ContentDto::new(
            Some(Uuid::from_u128(11)),
            "bucket-a".to_string(),
            "about/profile.png".to_string(),
            Some("image/png".to_string()),
        )),
    );

    let (about_me_repo, _about_me_probe) = create_about_me_repository(about_me.clone(), about_me);
    let (storage_repo, storage_probe) =
        create_storage_repository("https://cdn.example/about/profile.png");
    let service = AboutMeService::new(about_me_repo, storage_repo, "bucket-a".to_string());

    let result = run(service.about_me()).expect("about_me should succeed");
    let view = serde_json::to_value(result).expect("view should serialize");

    assert_eq!(view["firstName"], "Ada");
    assert_eq!(view["lastName"], "Lovelace");
    assert_eq!(view["photoUrl"], "https://cdn.example/about/profile.png");
    assert_eq!(
        storage_probe.url_requests(),
        vec![("bucket-a".to_string(), "about/profile.png".to_string())]
    );
}

#[test]
fn add_profile_picture_uploads_under_about_prefix_and_removes_previous_photo() {
    let about_me_id = Uuid::from_u128(2);
    let old_photo = ContentDto::new(
        Some(Uuid::from_u128(21)),
        "bucket-a".to_string(),
        "about/old.png".to_string(),
        Some("image/png".to_string()),
    );
    let about_me = AboutMeDto::new(
        Some(about_me_id),
        "Grace".to_string(),
        "Hopper".to_string(),
        None,
        Some(old_photo.clone()),
    );
    let new_bytes = vec![1_u8, 2, 3, 4];

    let (about_me_repo, about_me_probe) = create_about_me_repository(about_me.clone(), about_me);
    let (storage_repo, storage_probe) =
        create_storage_repository("https://cdn.example/about/new.png");
    let service = AboutMeService::new(about_me_repo, storage_repo, "bucket-a".to_string());

    run(service.add_profile_picture(about_me_id, "new.png".to_string(), &new_bytes))
        .expect("add_profile_picture should succeed");

    assert_eq!(
        storage_probe.uploaded_files(),
        vec![(
            "bucket-a".to_string(),
            "about/new.png".to_string(),
            new_bytes
        )]
    );
    assert_eq!(
        about_me_probe
            .last_updated_photo()
            .expect("photo should be updated")
            .1
            .file_name,
        "about/new.png"
    );
    assert_eq!(
        storage_probe.removed_objects(),
        vec![("bucket-a".to_string(), "about/old.png".to_string())]
    );
}

#[test]
fn delete_photo_removes_previous_photo_from_storage() {
    let about_me_id = Uuid::from_u128(5);
    let about_me = AboutMeDto::new(
        Some(about_me_id),
        "Katherine".to_string(),
        "Johnson".to_string(),
        None,
        Some(ContentDto::new(
            Some(Uuid::from_u128(51)),
            "bucket-a".to_string(),
            "about/old.png".to_string(),
            Some("image/png".to_string()),
        )),
    );

    let (about_me_repo, about_me_probe) = create_about_me_repository(about_me.clone(), about_me);
    let (storage_repo, storage_probe) = create_storage_repository("https://cdn.example/unused");
    let service = AboutMeService::new(about_me_repo, storage_repo, "bucket-a".to_string());

    run(service.delete_photo(about_me_id)).expect("delete_photo should succeed");

    assert_eq!(about_me_probe.deleted_photo_ids(), vec![about_me_id]);
    assert_eq!(
        storage_probe.removed_objects(),
        vec![("bucket-a".to_string(), "about/old.png".to_string())]
    );
}

#[test]
fn add_project_uploads_using_bucket_prefixed_key_and_returns_resolved_url() {
    let project_id = Uuid::from_u128(3);
    let project_content = ProjectContentDto::new(
        Some(Uuid::from_u128(31)),
        project_id,
        Some(ContentDto::new(
            Some(Uuid::from_u128(32)),
            "portfolio-assets".to_string(),
            "portfolio-assets/hero.png".to_string(),
            Some("image/png".to_string()),
        )),
        None,
    );

    let (project_repo, project_probe) = create_project_repository(
        Some(ProjectDto::new()),
        Some(project_content.clone()),
        None,
        None,
    );
    let (storage_repo, storage_probe) =
        create_storage_repository("https://cdn.example/portfolio-assets/hero.png");
    let service = ProjectService::new(
        project_repo,
        storage_repo,
        create_spotlight_repository(),
        "portfolio-assets".to_string(),
    );

    let result = run(service.add_project(project_id, "hero.png".to_string(), &[9_u8, 8, 7]))
        .expect("add_project should succeed");
    let view = serde_json::to_value(result).expect("content view should serialize");

    assert_eq!(
        storage_probe.uploaded_files(),
        vec![(
            "portfolio-assets".to_string(),
            "portfolio-assets/hero.png".to_string(),
            vec![9_u8, 8, 7],
        )]
    );
    assert_eq!(project_probe.added_content_requests().len(), 1);
    assert_eq!(view["mimeType"], "image/png");
    assert_eq!(view["url"], "https://cdn.example/portfolio-assets/hero.png");
}

#[test]
fn delete_project_content_removes_storage_object_and_thumbnail_entry() {
    let project_id = Uuid::from_u128(4);
    let content_id = Uuid::from_u128(41);
    let thumbnail_id = Uuid::from_u128(42);
    let content = ProjectContentDto::new(
        Some(content_id),
        project_id,
        Some(ContentDto::new(
            Some(content_id),
            "portfolio-assets".to_string(),
            "portfolio-assets/content.png".to_string(),
            Some("image/png".to_string()),
        )),
        None,
    );
    let thumbnail = ProjectContentDto::new(
        Some(thumbnail_id),
        project_id,
        Some(ContentDto::new(
            Some(thumbnail_id),
            "portfolio-assets".to_string(),
            "portfolio-assets/content.png".to_string(),
            Some("image/png".to_string()),
        )),
        None,
    );

    let (project_repo, project_probe) = create_project_repository(
        Some(ProjectDto::new()),
        None,
        Some(content.clone()),
        Some(thumbnail),
    );
    let (storage_repo, storage_probe) = create_storage_repository("https://cdn.example/unused");
    let service = ProjectService::new(
        project_repo,
        storage_repo,
        create_spotlight_repository(),
        "portfolio-assets".to_string(),
    );

    run(service.delete_project_content(project_id, content_id))
        .expect("delete_project_content should succeed");

    assert_eq!(
        storage_probe.removed_objects(),
        vec![(
            "portfolio-assets".to_string(),
            "portfolio-assets/content.png".to_string()
        )]
    );
    assert_eq!(
        project_probe.deleted_content_requests(),
        vec![(project_id, content_id)]
    );
    assert_eq!(
        project_probe.deleted_thumbnail_requests(),
        vec![(project_id, thumbnail_id)]
    );
}
