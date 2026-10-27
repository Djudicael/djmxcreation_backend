use app_config::storage_configuration::StorageConfiguration;
use app_core::optimization::IMediaProcessing;
use djmxcreation_media_worker::{app_router, processing};
use image::{ImageEncoder, codecs::png::PngEncoder};
use repository::{
    config::{db::DatabaseConfig, storage::get_storage_client},
    optimization_repository::{OptimizationRepository, VariantWrite},
};
use serde_json::{Value, json};
use std::{sync::Arc, time::Duration};
use uuid::Uuid;

const TOKEN: &str = "test-only-media-processing-token-with-more-than-32-characters";

#[tokio::test]
async fn durable_pipeline_fallback_leases_and_deletion() {
    unsafe {
        std::env::set_var("MEDIA_OPTIMIZATION_ENABLED", "true");
        std::env::set_var("MEDIA_PROCESSING_TOKEN", TOKEN);
    }
    let (db, uri) = test_util::shared_harness::shared_postgres().await;
    let endpoint = test_util::shared_harness::shared_rustfs().await;
    let bucket = format!("optimization-{}", Uuid::new_v4().simple());
    let config = StorageConfiguration::new(
        endpoint.clone(),
        "rustfsadmin".into(),
        "rustfsadmin".into(),
        "us-east-1".into(),
        bucket.clone(),
    );
    let storage = get_storage_client(config).unwrap();
    repository::config::storage::ensure_bucket(&bucket, &storage)
        .await
        .unwrap();
    let repo = OptimizationRepository::new(db.clone(), storage.clone());
    let server = axum_test::TestServer::new(app_router(repo.clone()));
    server
        .post("/internal/media/process-due")
        .await
        .assert_status_unauthorized();
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(90))
        .build()
        .unwrap();
    let platform_http = reqwest::Client::builder()
        .http2_prior_knowledge()
        .timeout(Duration::from_secs(90))
        .build()
        .unwrap();
    let mut external = None;
    let mut origin = None;
    if let Ok(probe) = std::env::var("WCP_MEDIA_PROBE") {
        let port = std::net::TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let component = std::env::var("DJMX_MEDIA_COMPONENT").unwrap_or_else(|_| {
            format!(
                "{}/../../target/wasm32-wasip2/wasi-release/djmxcreation_media_worker.wasm",
                env!("CARGO_MANIFEST_DIR")
            )
        });
        let log = std::fs::File::create(format!(
            "{}/../../target/wcp-media-pipeline.log",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let child = tokio::process::Command::new(probe)
            .arg(component)
            .arg(port.to_string())
            .env("DATABASE_URL", uri)
            .env("STORAGE_ENDPOINT", &endpoint)
            .env("STORAGE_BUCKET", &bucket)
            .env("STORAGE_ACCESS_KEY", "rustfsadmin")
            .env("STORAGE_SECRET_KEY", "rustfsadmin")
            .env("STORAGE_REGION", "us-east-1")
            .env("MEDIA_OPTIMIZATION_ENABLED", "true")
            .env("MEDIA_PROCESSING_TOKEN", TOKEN)
            .stdout(log.try_clone().unwrap())
            .stderr(log)
            .kill_on_drop(true)
            .spawn()
            .unwrap();
        external = Some(child);
        origin = Some(format!("http://127.0.0.1:{port}"));
        let mut ready = false;
        for _ in 0..600 {
            if let Ok(response) = platform_http
                .get(format!("{}/health", origin.as_ref().unwrap()))
                .send()
                .await
                && response.status().is_success()
            {
                ready = true;
                break;
            }
            if external.as_mut().unwrap().try_wait().unwrap().is_some() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        assert!(
            ready,
            "Platform runtime did not start; inspect target/wcp-media-pipeline.log"
        );
        let unauth = platform_http
            .post(format!(
                "{}/internal/media/process-due",
                origin.as_ref().unwrap()
            ))
            .send()
            .await
            .unwrap();
        assert_eq!(unauth.status(), 401);
    }
    let mut conn = db.connect().await.unwrap();
    let project = Uuid::new_v4();
    conn.execute_params(
        "INSERT INTO project(id,created_on,visible,adult) VALUES($1,NOW(),TRUE,FALSE)",
        &[&project],
    )
    .await
    .unwrap();
    let width = std::env::var("MEDIA_TEST_WIDTH")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(1200);
    let height = std::env::var("MEDIA_TEST_HEIGHT")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(width * 2 / 3);
    let pixels = image::RgbImage::from_fn(width, height, |x, y| {
        image::Rgb([(x % 255) as u8, (y % 255) as u8, 100])
    });
    let mut original = Vec::new();
    PngEncoder::new(&mut original)
        .write_image(
            pixels.as_raw(),
            width,
            height,
            image::ExtendedColorType::Rgb8,
        )
        .unwrap();
    let media = insert(&repo, &storage, &project, &bucket, &original).await;
    conn.query_params("SELECT enqueue_media_optimization($1)", &[&media])
        .await
        .unwrap();
    assert_eq!(
        repo.images(&[media]).await.unwrap()[&media].status,
        "pending"
    );
    assert!(
        repo.images(&[media]).await.unwrap()[&media]
            .sources
            .is_empty()
    );
    let mut statuses = Vec::new();
    for _ in 0..4 {
        let started = std::time::Instant::now();
        let result: Value = if let Some(origin) = &origin {
            let response = platform_http
                .post(format!("{origin}/internal/media/process-due"))
                .header("X-Media-Worker-Key", TOKEN)
                .send()
                .await
                .unwrap();
            assert!(
                response.status().is_success(),
                "Platform processing returned HTTP {}; inspect runtime log",
                response.status()
            );
            response.json().await.unwrap()
        } else {
            let response = server
                .post("/internal/media/process-due")
                .add_header("X-Media-Worker-Key", TOKEN)
                .await;
            response.assert_status_ok();
            response.json()
        };
        statuses.push(result["status"].as_str().unwrap().to_owned());
        eprintln!(
            "Image {width}x{height}: step {} returned {} in {:?}",
            statuses.len(),
            result["status"],
            started.elapsed()
        );
        if result["status"] == "ready" {
            break;
        }
    }
    let ready = repo.images(&[media]).await.unwrap().remove(&media).unwrap();
    let largest = ready
        .sources
        .iter()
        .map(|source| source.width as u32)
        .max()
        .unwrap();
    let steps = processing::WIDTHS
        .iter()
        .position(|target| *target >= largest)
        .map(|position| position + 1)
        .unwrap_or(4);
    assert_eq!(statuses.len(), steps);
    assert_eq!(statuses.last().unwrap(), "ready");
    assert!(
        statuses[..steps - 1]
            .iter()
            .all(|status| status == "pending")
    );
    assert!(
        ready
            .sources
            .iter()
            .all(|source| source.width as u64 * source.height as u64
                <= processing::MAX_OUTPUT_PIXELS)
    );
    assert_eq!(ready.status, "ready");
    assert_eq!(ready.width, Some(width as i32));
    assert!(ready.sources.iter().any(|source| source.width == 400));
    assert!(ready.sources.iter().any(|source| source.width == 800));
    assert!(
        ready
            .sources
            .iter()
            .all(|source| source.width <= width as i32)
    );
    for source in &ready.sources {
        let response = http.get(&source.url).send().await.unwrap();
        assert!(response.status().is_success());
        let decoded = image::load_from_memory(&response.bytes().await.unwrap()).unwrap();
        assert_eq!(decoded.width(), source.width as u32);
        assert_eq!(decoded.height(), source.height as u32);
    }
    let key = format!("originals/{media}.png");
    assert_eq!(
        repo.storage
            .inner
            .get_object()
            .bucket(&bucket)
            .key(&key)
            .send()
            .await
            .unwrap()
            .body
            .collect()
            .await
            .unwrap()
            .into_bytes()
            .as_ref(),
        original
    );
    // Deletion hides immediately, keeps durable keys, and clears all derived files.
    conn.execute_params("DELETE FROM project_content WHERE id=$1", &[&media])
        .await
        .unwrap();
    assert!(!repo.images(&[media]).await.unwrap().contains_key(&media));
    repo.cleanup(32).await.unwrap();
    assert!(
        repo.storage
            .inner
            .head_object()
            .bucket(&bucket)
            .key(&key)
            .send()
            .await
            .is_err()
    );
    conn.execute_params(
        "UPDATE media_cleanup SET available_at=NOW() WHERE bucket=$1",
        &[&bucket],
    )
    .await
    .unwrap();
    repo.cleanup(32).await.unwrap();
    assert_eq!(
        count(&db, "SELECT COUNT(*) AS count FROM media_asset").await,
        0
    );
    // Expired invocations are fenced out, and deletion during CPU work cannot resurrect a file.
    let second = insert(&repo, &storage, &project, &bucket, &original).await;
    conn.query_params("SELECT enqueue_media_optimization($1)", &[&second])
        .await
        .unwrap();
    let old = repo.claim().await.unwrap().unwrap();
    conn.execute_params(
        "UPDATE media_optimization SET lease_until=NOW()-INTERVAL '1 second' WHERE media_id=$1",
        &[&second],
    )
    .await
    .unwrap();
    let fresh = repo.claim().await.unwrap().unwrap();
    assert_ne!(old.token, fresh.token);
    let output = || VariantWrite {
        target: 400,
        width: 400,
        height: 267,
        extension: "jpg",
        mime: "image/jpeg",
        bytes: vec![1, 2, 3],
    };
    assert!(!repo.store(&old, output()).await.unwrap());
    conn.execute_params("DELETE FROM project_content WHERE id=$1", &[&second])
        .await
        .unwrap();
    assert!(!repo.store(&fresh, output()).await.unwrap());
    conn.execute_params(
        "UPDATE media_cleanup SET available_at=NOW() WHERE bucket=$1",
        &[&bucket],
    )
    .await
    .unwrap();
    repo.cleanup(32).await.unwrap();
    // Cleanup survives a storage outage, without needing the optional worker.
    let third = insert(&repo, &storage, &project, &bucket, &original).await;
    conn.execute_params("DELETE FROM project_content WHERE id=$1", &[&third])
        .await
        .unwrap();
    let unavailable = get_storage_client(StorageConfiguration::new(
        "http://127.0.0.1:1".into(),
        "test".into(),
        "test".into(),
        "us-east-1".into(),
        bucket.clone(),
    ))
    .unwrap();
    let offline = OptimizationRepository::new(db.clone(), unavailable);
    assert_eq!(offline.cleanup(32).await.unwrap(), 0);
    assert!(count(&db, "SELECT COUNT(*) AS count FROM media_cleanup").await > 0);
    conn.execute_params(
        "UPDATE media_cleanup SET available_at=NOW() WHERE bucket=$1",
        &[&bucket],
    )
    .await
    .unwrap();
    repo.cleanup(32).await.unwrap();
    assert_eq!(
        count(&db, "SELECT COUNT(*) AS count FROM media_cleanup").await,
        0
    );
    // Cascading project deletion schedules original cleanup even without optimization jobs.
    insert(&repo, &storage, &project, &bucket, &original).await;
    insert(&repo, &storage, &project, &bucket, &original).await;
    conn.execute_params("DELETE FROM project WHERE id=$1", &[&project])
        .await
        .unwrap();
    repo.cleanup(32).await.unwrap();
    assert_eq!(
        count(&db, "SELECT COUNT(*) AS count FROM media_asset").await,
        0
    );
    unsafe {
        std::env::set_var("MEDIA_OPTIMIZATION_ENABLED", "false");
    }
    assert!(repo.images(&[media]).await.unwrap().is_empty());
    if let Some(mut child) = external {
        let _ = child.kill().await;
    }
}

async fn insert(
    repo: &OptimizationRepository,
    storage: &repository::config::storage::StorageClient,
    project: &Uuid,
    bucket: &str,
    bytes: &[u8],
) -> Uuid {
    let id = Uuid::new_v4();
    let key = format!("originals/{id}.png");
    storage
        .inner
        .put_object()
        .bucket(bucket)
        .key(&key)
        .content_type("image/png")
        .body(bytes.to_vec().into())
        .send()
        .await
        .unwrap();
    let data = json!({"bucket_name":bucket,"file_name":key,"mime_type":"image/png"});
    let mut conn = repo.db.connect().await.unwrap();
    conn.execute_params(
        "INSERT INTO project_content(id,project_id,content,created_on) VALUES($1,$2,$3,NOW())",
        &[&id, project, &data],
    )
    .await
    .unwrap();
    id
}
async fn count(db: &Arc<DatabaseConfig>, sql: &str) -> i64 {
    let mut conn = db.connect().await.unwrap();
    conn.query(sql).await.unwrap().into_rows()[0]
        .get_by_name("count")
        .unwrap()
}

#[test]
fn image_processing_preserves_proportions_transparency_and_orientation() {
    let pixels =
        image::RgbaImage::from_fn(100, 200, |x, y| image::Rgba([x as u8, y as u8, 80, 90]));
    let mut png = Vec::new();
    PngEncoder::new(&mut png)
        .write_image(pixels.as_raw(), 100, 200, image::ExtendedColorType::Rgba8)
        .unwrap();
    let output = processing::resize(&png, 400).unwrap();
    assert_eq!((output.width, output.height), (100, 200));
    assert_eq!(output.baseline.mime, "image/png");
    assert_eq!(
        image::load_from_memory(&output.baseline.bytes)
            .unwrap()
            .to_rgba8()
            .get_pixel(50, 50)[3],
        90
    );
    assert!(processing::resize(b"not an image", 400).is_err());
    let pixels = image::RgbaImage::from_pixel(800, 400, image::Rgba([80, 100, 150, 90]));
    let mut png = Vec::new();
    PngEncoder::new(&mut png)
        .write_image(pixels.as_raw(), 800, 400, image::ExtendedColorType::Rgba8)
        .unwrap();
    let resized = processing::resize(&png, 400).unwrap();
    assert_eq!((resized.width, resized.height), (400, 200));
    assert_eq!(
        image::load_from_memory(&resized.baseline.bytes)
            .unwrap()
            .to_rgba8()
            .get_pixel(100, 50)[3],
        90
    );
    let pixels = image::RgbImage::new(600, 400);
    let mut jpeg = Vec::new();
    let mut encoder = image::codecs::jpeg::JpegEncoder::new(&mut jpeg);
    // TIFF little-endian IFD: orientation 6 (90 degrees clockwise).
    encoder
        .set_exif_metadata(vec![
            73, 73, 42, 0, 8, 0, 0, 0, 1, 0, 18, 1, 3, 0, 1, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0,
        ])
        .unwrap();
    encoder
        .write_image(pixels.as_raw(), 600, 400, image::ExtendedColorType::Rgb8)
        .unwrap();
    let rotated = processing::resize(&jpeg, 400).unwrap();
    assert_eq!((rotated.width, rotated.height), (400, 600));
}
