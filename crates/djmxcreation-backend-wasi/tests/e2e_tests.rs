use axum::Router;
use axum_test::TestServer;
use serde_json::json;
use std::env;
use std::sync::Once;

#[allow(dead_code)]
mod support;
use djmxcreation_backend_wasi::server::starter::build_router_with_auth;

static INIT: Once = Once::new();

#[tokio::test]
async fn media_reorder_is_atomic_scoped_and_rejects_stale_orders() {
    let (mut server, hub) = test_app_with_identity().await;
    let created = server
        .post("/api/portfolio/v1/projects")
        .json(&json!({"title":"Reorder test"}))
        .await
        .json::<serde_json::Value>();
    let project: uuid::Uuid = created["id"].as_str().unwrap().parse().unwrap();
    let path = format!("/api/portfolio/v1/projects/{project}");
    let (config, _) = test_util::shared_harness::shared_postgres().await;
    let mut conn = config.connect().await.unwrap();
    let ids: Vec<_> = (0..3).map(|_| uuid::Uuid::new_v4()).collect();
    for (index, id) in ids.iter().enumerate() {
        let content = json!({"bucket_name":"portfolio","file_name":format!("order/{id}.png"),"mime_type":"image/png","details":{"caption":"Keep this caption","alt":"Keep this alt","position":0}});
        conn.execute_params("INSERT INTO project_content(id,project_id,content,created_on) VALUES($1,$2,$3,NOW()+$4::int*INTERVAL '1 second')", &[id,&project,&content,&(index as i32)]).await.unwrap();
    }
    let reversed: Vec<_> = ids.iter().copied().rev().collect();
    let order_path = format!("{path}/media-order");
    server.clear_headers();
    server
        .put(&order_path)
        .json(&json!({"ids":reversed,"previous":ids}))
        .await
        .assert_status_unauthorized();
    let mut claims = hub.claims();
    claims["realm_access"] = json!({"roles":[]});
    server.add_header(
        "Authorization",
        format!("Bearer {}", support::sign(&claims, "hub-ed")),
    );
    server
        .put(&order_path)
        .json(&json!({"ids":reversed,"previous":ids}))
        .await
        .assert_status_forbidden();
    server.clear_headers();
    server.add_header("Authorization", format!("Bearer {}", hub.token()));
    for invalid in [
        vec![ids[0], ids[0], ids[2]],
        vec![ids[0], ids[1]],
        vec![ids[0], ids[1], uuid::Uuid::new_v4()],
    ] {
        server
            .put(&order_path)
            .json(&json!({"ids":invalid,"previous":ids}))
            .await
            .assert_status_bad_request();
    }
    server
        .put(&order_path)
        .json(&json!({"ids":reversed,"previous":ids}))
        .await
        .assert_status_ok();
    server
        .put(&order_path)
        .json(&json!({"ids":ids,"previous":ids}))
        .await
        .assert_status_bad_request();
    let current = server.get(&path).await.json::<serde_json::Value>();
    assert_eq!(
        current["contents"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item["id"].as_str().unwrap().parse().unwrap())
            .collect::<Vec<uuid::Uuid>>(),
        reversed
    );
    assert!(
        current["contents"]
            .as_array()
            .unwrap()
            .iter()
            .all(|item| item["caption"] == "Keep this caption" && item["alt"] == "Keep this alt")
    );
    let alternative = vec![ids[1], ids[2], ids[0]];
    let first = server
        .put(&order_path)
        .json(&json!({"ids":ids,"previous":reversed}));
    let second = server
        .put(&order_path)
        .json(&json!({"ids":alternative,"previous":reversed}));
    let (first, second) = tokio::join!(first, second);
    assert_eq!(
        [first.status_code(), second.status_code()]
            .iter()
            .filter(|status| status.is_success())
            .count(),
        1
    );
    server.delete(&path).await.assert_status_ok();
}

fn init_tracing() {
    INIT.call_once(|| {
        tracing_subscriber::fmt()
            .with_env_filter("debug,repository=trace,wasi_pg_client=debug")
            .try_init()
            .ok();
    });
}

async fn test_app() -> TestServer {
    test_app_with_identity().await.0
}

async fn test_app_with_identity() -> (TestServer, support::Hub) {
    init_tracing();
    let (_db_cfg, uri) = test_util::shared_harness::shared_postgres().await;

    unsafe {
        env::set_var("DATABASE_URL", &uri);
        env::set_var("STORAGE_ENDPOINT", "http://127.0.0.1:3900");
        env::set_var("STORAGE_ACCESS_KEY", "myaccesskey");
        env::set_var("STORAGE_SECRET_KEY", "mysecretkey");
        env::set_var("STORAGE_REGION", "us-west-1");
        env::set_var("STORAGE_BUCKET", "portfolio");
    }

    let hub = support::Hub::start().await;
    let router: Router = build_router_with_auth(
        app_config::config::Config::from_env().unwrap(),
        hub.verifier(),
    );
    let mut server = TestServer::new(router);
    server.add_header("Authorization", format!("Bearer {}", hub.token()));
    (server, hub)
}

#[tokio::test]
async fn anonymous_requests_cannot_administer_or_read_hidden_projects() {
    let (mut server, hub) = test_app_with_identity().await;
    let response = server
        .post("/api/portfolio/v1/projects")
        .json(&json!({"title":"Private project", "subTitle":"", "client":""}))
        .await;
    response.assert_status_ok();
    let id = response.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(response.json::<serde_json::Value>()["visible"], false);
    // A newly created project stays private until explicitly published.
    let response = server.put(&format!("/api/portfolio/v1/projects/{id}"))
        .json(&json!({"metadata":{"title":"Private project", "subTitle":"", "client":""}, "visible":false, "adult":false})).await;
    response.assert_status_ok();
    server.clear_headers();
    server
        .post("/api/portfolio/v1/projects")
        .json(&json!({"title":"Forbidden"}))
        .await
        .assert_status_unauthorized();
    server
        .get("/api/portfolio/v1/projects")
        .await
        .assert_status_unauthorized();
    server
        .get("/api/portfolio/v2/projects?page=1&size=10&visible=false")
        .await
        .assert_status_forbidden();
    server
        .get(&format!("/api/portfolio/v1/projects/{id}"))
        .await
        .assert_status_not_found();
    server
        .get("/api/portfolio/v2/projects?page=1&size=10&visible=true")
        .await
        .assert_status_ok();
    server.get("/api/about/v1/me").await.assert_status_ok();
    server
        .get("/api/contact/v1/information")
        .await
        .assert_status_ok();
    server.add_header("Authorization", format!("Bearer {}", hub.token()));
    server
        .get(&format!("/api/portfolio/v1/projects/{id}"))
        .await
        .assert_status_ok();
    server
        .delete(&format!("/api/portfolio/v1/projects/{id}"))
        .await
        .assert_status_ok();
}

#[tokio::test]
async fn wasi_e2e_ping() {
    let server = test_app().await;

    let response = server.get("/ping").await;
    response.assert_status_ok();
    response.assert_json(&json!({"message": "API is responsive"}));
}

#[tokio::test]
async fn wasi_e2e_about_me_crud() {
    let server = test_app().await;

    let response = server.get("/api/about/v1/me").await;
    response.assert_status_ok();

    let response = server
        .put("/api/about/v1/me/550e8400-e29b-41d4-a716-446655440000")
        .json(&json!({
            "firstName": "WASI",
            "lastName": "Test",
            "description": {"bio": "WASI integration test"}
        }))
        .await;
    response.assert_status_ok();

    let response = server.get("/api/about/v1/me").await;
    response.assert_status_ok();
    let body = response.json::<serde_json::Value>();
    assert_eq!(body["firstName"], "WASI");
    assert_eq!(body["lastName"], "Test");
}

#[tokio::test]
async fn wasi_e2e_contact_crud() {
    let server = test_app().await;

    let response = server.get("/api/contact/v1/information").await;
    response.assert_status_ok();

    let response = server
        .put("/api/contact/v1/information/550e8400-e29b-41d4-a716-446655440000")
        .json(&json!({
            "description": {"email": "wasi@example.com", "phone": "+111222333"}
        }))
        .await;
    response.assert_status_ok();
}

#[tokio::test]
async fn wasi_e2e_project_crud() {
    let server = test_app().await;

    let response = server
        .post("/api/portfolio/v1/projects")
        .json(&json!({
            "title": "WASI Project",
            "subTitle": "WASI Subtitle",
            "client": "WASI Client"
        }))
        .await;
    response.assert_status_ok();
    let body = response.json::<serde_json::Value>();
    let project_id = body["id"].as_str().unwrap().to_string();

    let response = server.get("/api/portfolio/v1/projects").await;
    response.assert_status_ok();

    let response = server
        .get("/api/portfolio/v2/projects")
        .add_query_param("page", 1)
        .add_query_param("size", 10)
        .add_query_param("visible", true)
        .await;
    response.assert_status_ok();

    let response = server
        .get(&format!("/api/portfolio/v1/projects/{project_id}"))
        .await;
    response.assert_status_ok();

    let response = server
        .put(&format!("/api/portfolio/v1/projects/{project_id}"))
        .json(&json!({
            "metadata": {"title": "Updated WASI", "subTitle": "Updated", "client": "Updated"},
            "visible": true,
            "adult": false
        }))
        .await;
    response.assert_status_ok();

    let response = server
        .delete(&format!("/api/portfolio/v1/projects/{project_id}"))
        .await;
    response.assert_status_ok();
}

#[tokio::test]
async fn wasi_e2e_spotlight_flow() {
    let server = test_app().await;

    let response = server
        .post("/api/portfolio/v1/projects")
        .json(&json!({"title": "Spotlight WASI", "subTitle": "Spot", "client": "Light"}))
        .await;
    response.assert_status_ok();
    let body = response.json::<serde_json::Value>();
    let project_id = body["id"].as_str().unwrap().to_string();

    let response = server
        .post("/api/portfolio/v1/projects/spotlights")
        .json(&json!({"projectId": project_id}))
        .await;
    response.assert_status_ok();
    let body = response.json::<serde_json::Value>();
    let spotlight_id = body["id"].as_str().unwrap().to_string();

    let response = server.get("/api/portfolio/v1/projects/spotlights").await;
    response.assert_status_ok();

    let response = server
        .get(&format!(
            "/api/portfolio/v1/projects/spotlights/{spotlight_id}"
        ))
        .await;
    response.assert_status_ok();

    let response = server
        .delete(&format!(
            "/api/portfolio/v1/projects/spotlights/{spotlight_id}"
        ))
        .await;
    response.assert_status_ok();

    let _ = server
        .delete(&format!("/api/portfolio/v1/projects/{project_id}"))
        .await;
}

#[tokio::test]
async fn wasi_e2e_metrics_endpoint() {
    let server = test_app().await;
    let response = server.get("/metrics").await;
    response.assert_status_ok();
}

#[tokio::test]
async fn home_settings_persist_validate_and_hide_ineligible_projects() {
    let (mut server, hub) = test_app_with_identity().await;
    let settings_path = "/api/home/v1/settings";
    let public_path = "/api/home/v1";
    server.clear_headers();
    server.get(public_path).await.assert_status_ok();
    server.get(settings_path).await.assert_status_unauthorized();
    server
        .put(settings_path)
        .json(&json!({"introduction":"Forbidden", "featuredProjectId":null}))
        .await
        .assert_status_unauthorized();
    let mut claims = hub.claims();
    claims["realm_access"] = json!({"roles":[]});
    server.add_header(
        "Authorization",
        format!("Bearer {}", support::sign(&claims, "hub-ed")),
    );
    server.get(settings_path).await.assert_status_forbidden();
    server
        .put(settings_path)
        .json(&json!({"introduction":"Forbidden", "featuredProjectId":null}))
        .await
        .assert_status_forbidden();
    server.clear_headers();
    server.add_header("Authorization", format!("Bearer {}", hub.token()));

    let created = server
        .post("/api/portfolio/v1/projects")
        .json(&json!({"title":"Home settings test"}))
        .await;
    created.assert_status_ok();
    let id: uuid::Uuid = created.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let project_path = format!("/api/portfolio/v1/projects/{id}");
    let payload = json!({"introduction":"  My introduction.  ", "featuredProjectId":id});
    server
        .put(settings_path)
        .json(&payload)
        .await
        .assert_status_bad_request();
    server
        .put(&project_path)
        .json(&json!({"metadata":{"title":"Home settings test"}, "visible":true, "adult":false}))
        .await
        .assert_status_ok();
    // A published project without a cover still cannot be featured.
    server
        .put(settings_path)
        .json(&payload)
        .await
        .assert_status_bad_request();
    let (config, _) = test_util::shared_harness::shared_postgres().await;
    let mut connection = config.connect().await.unwrap();
    let thumbnail = json!({"id":uuid::Uuid::new_v4(), "bucket_name":"portfolio", "file_name":"test/home.png", "mime_type":null});
    connection.execute_params("INSERT INTO project_content_thumbnail (project_id, content, created_on) VALUES ($1,$2,NOW())", &[&id, &thumbnail]).await.unwrap();
    let saved = server.put(settings_path).json(&payload).await;
    saved.assert_status_ok();
    assert_eq!(
        saved.json::<serde_json::Value>()["introduction"],
        "My introduction."
    );
    assert_eq!(
        server.get(settings_path).await.json::<serde_json::Value>()["featuredProjectId"],
        id.to_string()
    );

    server.clear_headers();
    assert_eq!(
        server.get(public_path).await.json::<serde_json::Value>()["featuredProjectId"],
        id.to_string()
    );
    server.add_header("Authorization", format!("Bearer {}", hub.token()));
    server
        .put(settings_path)
        .json(&json!({"introduction":"x".repeat(501), "featuredProjectId":id}))
        .await
        .assert_status_bad_request();
    server
        .put(settings_path)
        .json(&json!({"introduction":"Changed", "featuredProjectId":uuid::Uuid::new_v4()}))
        .await
        .assert_status_bad_request();
    assert_eq!(
        server.get(settings_path).await.json::<serde_json::Value>()["introduction"],
        "My introduction."
    );

    for (visible, adult) in [(false, false), (true, true)] {
        server.put(&project_path).json(&json!({"metadata":{"title":"Home settings test"}, "visible":visible, "adult":adult})).await.assert_status_ok();
        assert!(
            server.get(public_path).await.json::<serde_json::Value>()["featuredProjectId"]
                .is_null()
        );
        assert_eq!(
            server.get(settings_path).await.json::<serde_json::Value>()["featuredProjectId"],
            id.to_string()
        );
        server
            .put(settings_path)
            .json(&payload)
            .await
            .assert_status_bad_request();
    }
    server
        .put(&project_path)
        .json(&json!({"metadata":{"title":"Home settings test"}, "visible":true, "adult":false}))
        .await
        .assert_status_ok();
    assert_eq!(
        server.get(public_path).await.json::<serde_json::Value>()["featuredProjectId"],
        id.to_string()
    );
    // Replacing a cover with a non-image must also stop public featuring.
    let video = json!({"file_name":"test/home.mp4", "mime_type":"video/mp4"});
    connection
        .execute_params(
            "UPDATE project_content_thumbnail SET content=$1 WHERE project_id=$2",
            &[&video, &id],
        )
        .await
        .unwrap();
    assert!(
        server.get(public_path).await.json::<serde_json::Value>()["featuredProjectId"].is_null()
    );
    server
        .put(settings_path)
        .json(&payload)
        .await
        .assert_status_bad_request();
    connection
        .execute_params(
            "DELETE FROM project_content_thumbnail WHERE project_id=$1",
            &[&id],
        )
        .await
        .unwrap();
    assert!(
        server.get(public_path).await.json::<serde_json::Value>()["featuredProjectId"].is_null()
    );
    server.delete(&project_path).await.assert_status_ok();
    let after_delete = server.get(settings_path).await.json::<serde_json::Value>();
    assert!(after_delete["featuredProjectId"].is_null());
    assert_eq!(after_delete["introduction"], "My introduction.");
    server
        .put(settings_path)
        .json(&json!({"introduction":"", "featuredProjectId":null}))
        .await
        .assert_status_ok();
    assert_eq!(
        server.get(public_path).await.json::<serde_json::Value>(),
        json!({"introduction":"", "featuredProjectId":null})
    );
}
