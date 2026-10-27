#[allow(dead_code)]
mod support;
use app_config::{
    config::Config, database_configuration::DatabaseConfiguration,
    storage_configuration::StorageConfiguration,
};
use axum_test::TestServer;
use djmxcreation_backend_wasi::server::starter::build_router_with_auth;
use serde_json::{Value, json};

#[tokio::test]
async fn direct_upload_checks_parts_permissions_metadata_and_range_playback() {
    let (_, uri) = test_util::shared_harness::shared_postgres().await;
    let endpoint = test_util::shared_harness::shared_rustfs().await;
    let storage = StorageConfiguration::new(
        endpoint,
        "rustfsadmin".into(),
        "rustfsadmin".into(),
        "us-east-1".into(),
        "media-test".into(),
    );
    let client = repository::config::storage::get_storage_client(storage.clone()).unwrap();
    repository::config::storage::ensure_bucket("media-test", &client)
        .await
        .unwrap();
    repository::config::storage::configure_media_cors(
        "media-test",
        &client,
        vec!["http://localhost:3008".into()],
    )
    .await
    .unwrap();
    let hub = support::Hub::start().await;
    let mut server = TestServer::new(build_router_with_auth(
        Config {
            database: DatabaseConfiguration::new(uri, 5),
            storage,
            ..Default::default()
        },
        hub.verifier(),
    ));
    let auth = format!("Bearer {}", hub.token());
    server.add_header("Authorization", auth.clone());
    let created = server
        .post("/api/portfolio/v1/projects")
        .json(&json!({"title":"Direct upload"}))
        .await;
    created.assert_status_ok();
    let project = created.json::<Value>()["id"].as_str().unwrap().to_owned();
    let path = format!("/api/portfolio/v1/projects/{project}");
    let request = json!({"fileName":"film.mp4", "mimeType":"video/mp4", "size":8*1024*1024+100});
    server.clear_headers();
    server
        .post(&format!("{path}/uploads"))
        .json(&request)
        .await
        .assert_status_unauthorized();
    let mut claims = hub.claims();
    claims["realm_access"] = json!({"roles":[]});
    server.add_header(
        "Authorization",
        format!("Bearer {}", support::sign(&claims, "hub-ed")),
    );
    server
        .post(&format!("{path}/uploads"))
        .json(&request)
        .await
        .assert_status_forbidden();
    server.clear_headers();
    server.add_header("Authorization", auth);
    server
        .post(&format!("{path}/uploads"))
        .json(&json!({"fileName":"film.mp4","mimeType":"video/mp4","size":2147483649_i64}))
        .await
        .assert_status_bad_request();
    server
        .post(&format!("{path}/uploads"))
        .json(&json!({"fileName":"../film.mp4","mimeType":"video/mp4","size":100}))
        .await
        .assert_status_bad_request();
    server
        .post(&format!("{path}/uploads"))
        .json(&json!({"fileName":"film.mov","mimeType":"video/quicktime","size":100}))
        .await
        .assert_status_bad_request();
    let begun = server.post(&format!("{path}/uploads")).json(&request).await;
    begun.assert_status_ok();
    let session = begun.json::<Value>();
    assert_eq!(session["partCount"], 2);
    let upload = format!("{path}/uploads/{}", session["id"].as_str().unwrap());
    server
        .post(&format!("{upload}/complete"))
        .await
        .assert_status_bad_request();
    server
        .post(&format!("{upload}/parts/3"))
        .await
        .assert_status_bad_request();
    let http = reqwest::Client::new();
    for (part, size) in [(1, 8 * 1024 * 1024), (2, 100)] {
        let signed = server.post(&format!("{upload}/parts/{part}")).await;
        signed.assert_status_ok();
        let url = signed.json::<Value>()["url"].as_str().unwrap().to_owned();
        // Uploading a part twice is safe and represents a retry after a lost response.
        for _ in 0..2 {
            let result = http.put(&url).body(vec![7_u8; size]).send().await.unwrap();
            assert!(
                result.status().is_success(),
                "storage status {}",
                result.status()
            );
        }
    }
    let completed = server.post(&format!("{upload}/complete")).await;
    completed.assert_status_ok();
    server
        .post(&format!("{upload}/complete"))
        .await
        .assert_status_ok();
    server
        .post(&format!("{upload}/parts/1"))
        .await
        .assert_status_bad_request();
    let id = completed.json::<Value>()["id"].as_str().unwrap().to_owned();
    server.put(&format!("{path}/media/{id}")).json(&json!({"caption":"On set", "alt":"", "behindScenes":true,"position":3,"posterId":null})).await.assert_status_ok();
    server
        .put(&format!("{path}/media/{id}"))
        .json(&json!({"position":-1}))
        .await
        .assert_status_bad_request();
    server
        .put(&format!("{path}/media/{id}"))
        .json(&json!({"posterId":id}))
        .await
        .assert_status_bad_request();
    let project_data = server.get(&path).await.json::<Value>();
    assert_eq!(project_data["contents"].as_array().unwrap().len(), 1);
    let content = &project_data["contents"][0];
    assert_eq!(content["caption"], "On set");
    assert_eq!(content["behindScenes"], true);
    assert_eq!(content["mimeType"], "video/mp4");
    let range = http
        .get(content["url"].as_str().unwrap())
        .header("Range", "bytes=0-9")
        .send()
        .await
        .unwrap();
    assert_eq!(range.status(), 206);
    assert_eq!(range.bytes().await.unwrap().len(), 10);
    let cancelled = server
        .post(&format!("{path}/uploads"))
        .json(&request)
        .await
        .json::<Value>();
    let cancelled_path = format!("{path}/uploads/{}", cancelled["id"].as_str().unwrap());
    server
        .post(&format!("{cancelled_path}/abort"))
        .await
        .assert_status_ok();
    server
        .post(&format!("{cancelled_path}/abort"))
        .await
        .assert_status_ok();
    server
        .post(&format!("{cancelled_path}/complete"))
        .await
        .assert_status_bad_request();
    server.delete(&path).await.assert_status_ok();
}
