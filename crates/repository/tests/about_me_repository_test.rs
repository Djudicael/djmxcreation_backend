use app_config::database_configuration::DatabaseConfiguration;
use app_core::about_me::about_me_repository::IAboutMeRepository;
use app_core::dto::about_me_dto::AboutMeDto;
use app_core::dto::content_dto::ContentDto;
use repository::about_me_repository::AboutMeRepository;
use repository::config::db::db_client;
use serde_json::json;
use std::process::Command;
use std::sync::Arc;
use test_util::postgresql::init_postgresql;
use tokio::sync::Mutex;
use uuid::Uuid;

#[tokio::test]
async fn test_update_about_me() {
    let (repo, id) = setup_test_db().await;

    let about_me = repo
        .update_about_me(
            id,
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

    assert_eq!(about_me.first_name, "Alice");
    assert_eq!(about_me.last_name, "Doe");
}

#[tokio::test]
async fn test_get_about_me() {
    let (repo, _id) = setup_test_db().await;

    let about_me = repo.get_about_me().await.expect("Failed to get about_me");

    assert!(about_me.first_name.len() > 0);
    assert!(about_me.last_name.len() > 0);
}

// #[tokio::test]
// async fn test_get_about_me_by_id() {
//     let (repo, id) = setup_test_db().await;

//     let about_me = repo
//         .get_about_me_by_id(id)
//         .await
//         .expect("Failed to get about_me by id");

//     assert_eq!(about_me.first_name, "Test");
//     assert_eq!(about_me.last_name, "User");
// }

// #[tokio::test]
// async fn test_update_photo() {
//     let (repo, id) = setup_test_db().await;

//     let content = ContentDto {
//         id: None,
//         bucket_name: "test_bucket".to_string(),
//         file_name: "test_file".to_string(),
//         mime_type: None,
//     };

//     repo.update_photo(id, &content)
//         .await
//         .expect("Failed to update photo");

//     let updated_about_me = repo
//         .get_about_me_by_id(id)
//         .await
//         .expect("Failed to fetch updated about_me");

//     assert!(updated_about_me.photo.is_some());
// }

// #[tokio::test]
// async fn test_delete_about_me_photo() {
//     let (repo, id) = setup_test_db().await;

//     repo.delete_about_me_photo(id)
//         .await
//         .expect("Failed to delete photo");

//     let updated_about_me = repo
//         .get_about_me_by_id(id)
//         .await
//         .expect("Failed to fetch updated about_me");

//     assert!(updated_about_me.photo.is_none());
// }

async fn start_podman() {
    let output = Command::new("podman")
        .args([
            "run",
            "--rm",
            "--name",
            "test-postgres",
            "-e",
            "POSTGRES_USER=postgres",
            "-e",
            "POSTGRES_PASSWORD=postgres",
            "-e",
            "POSTGRES_DB=portfolio",
            "-p",
            "5432:5432",
            "-d",
            "docker.io/library/postgres:latest", // ✅ Add PostgreSQL image
        ])
        .output()
        .expect("Failed to start Podman");

    if !output.status.success() {
        eprintln!("Podman failed: {:?}", output);
    } else {
        println!(
            "Podman started: {:?}",
            String::from_utf8_lossy(&output.stdout)
        );
    }

    // Wait for PostgreSQL to be ready
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
}

async fn setup_test_db() -> (AboutMeRepository, Uuid) {
    // Start Podman before connecting
    // start_podman().await;
    let test_db_config = DatabaseConfiguration {
        pg_user: "postgres".to_string(),
        pg_password: "postgres".to_string(),
        pg_host: "localhost".to_string(),
        pg_db: "portfolio".to_string(),
        pg_app_max_con: 5,
    };

    let (podman, image) = init_postgresql(&test_db_config).expect("Failed to init PostgreSQL");
    let _ = podman.start(image).await.expect("Failed to run PostgreSQL");

    let client = db_client(&test_db_config)
        .await
        .expect("Failed to connect to the test database");

    let repo = AboutMeRepository::new(Arc::new(Mutex::new(client)));

    let id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").expect("uuid parse error");

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

    (repo, id)
}
