use axum::Router;
use axum_test::TestServer;
use serde_json::json;

use djmxcreation_backend_axum::server::starter::build_router;

async fn test_app() -> TestServer {
    let router: Router = build_router().await;
    TestServer::new(router).expect("should create test server")
}

#[tokio::test]
async fn e2e_ping() {
    let server = test_app().await;

    let response = server.get("/ping").await;
    response.assert_status_ok();
    response.assert_json(&json!({"message": "API is responsive"}));
}

#[tokio::test]
async fn e2e_about_me_crud() {
    let server = test_app().await;

    // GET /api/about/v1/me
    let response = server.get("/api/about/v1/me").await;
    response.assert_status_ok();

    // PUT /api/about/v1/me/{id}
    let response = server
        .put("/api/about/v1/me/550e8400-e29b-41d4-a716-446655440000")
        .json(&json!({
            "firstName": "Test",
            "lastName": "User",
            "description": {"bio": "Integration test"}
        }))
        .await;
    response.assert_status_ok();

    // Verify update
    let response = server.get("/api/about/v1/me").await;
    response.assert_status_ok();
    let body = response.json::<serde_json::Value>();
    assert_eq!(body["firstName"], "Test");
    assert_eq!(body["lastName"], "User");
}

#[tokio::test]
async fn e2e_contact_crud() {
    let server = test_app().await;

    let response = server.get("/api/contact/v1/information").await;
    response.assert_status_ok();

    let response = server
        .put("/api/contact/v1/information/550e8400-e29b-41d4-a716-446655440000")
        .json(&json!({
            "description": {"email": "e2e@example.com", "phone": "+999888777"}
        }))
        .await;
    response.assert_status_ok();
}

#[tokio::test]
async fn e2e_project_crud() {
    let server = test_app().await;

    // POST /api/portfolio/v1/projects
    let response = server
        .post("/api/portfolio/v1/projects")
        .json(&json!({
            "title": "E2E Project",
            "subTitle": "End to end",
            "client": "Test Runner"
        }))
        .await;
    response.assert_status_ok();
    let body = response.json::<serde_json::Value>();
    let project_id = body["id"].as_str().unwrap().to_string();

    // GET /api/portfolio/v1/projects
    let response = server.get("/api/portfolio/v1/projects").await;
    response.assert_status_ok();

    // GET /api/portfolio/v2/projects
    let response = server
        .get("/api/portfolio/v2/projects")
        .add_query_param("page", 1)
        .add_query_param("size", 10)
        .add_query_param("visible", true)
        .await;
    response.assert_status_ok();

    // GET /api/portfolio/v1/projects/{id}
    let response = server
        .get(&format!("/api/portfolio/v1/projects/{project_id}"))
        .await;
    response.assert_status_ok();

    // PUT /api/portfolio/v1/projects/{id}
    let response = server
        .put(&format!("/api/portfolio/v1/projects/{project_id}"))
        .json(&json!({
            "metadata": {"title": "Updated E2E", "subTitle": "Updated", "client": "Updated"},
            "visible": true,
            "adult": false
        }))
        .await;
    response.assert_status_ok();

    // DELETE /api/portfolio/v1/projects/{id}
    let response = server
        .delete(&format!("/api/portfolio/v1/projects/{project_id}"))
        .await;
    response.assert_status_ok();
}

#[tokio::test]
async fn e2e_spotlight_flow() {
    let server = test_app().await;

    // Create a project to spotlight
    let response = server
        .post("/api/portfolio/v1/projects")
        .json(&json!({"title": "Spotlight E2E", "subTitle": "Spot", "client": "Light"}))
        .await;
    response.assert_status_ok();
    let body = response.json::<serde_json::Value>();
    let project_id = body["id"].as_str().unwrap().to_string();

    // POST /api/portfolio/v1/projects/spotlights
    let response = server
        .post("/api/portfolio/v1/projects/spotlights")
        .json(&json!({"projectId": project_id}))
        .await;
    response.assert_status_ok();
    let body = response.json::<serde_json::Value>();
    let spotlight_id = body["id"].as_str().unwrap().to_string();

    // GET /api/portfolio/v1/projects/spotlights
    let response = server.get("/api/portfolio/v1/projects/spotlights").await;
    response.assert_status_ok();

    // GET /api/portfolio/v1/projects/spotlights/{id}
    let response = server
        .get(&format!("/api/portfolio/v1/projects/spotlights/{spotlight_id}"))
        .await;
    response.assert_status_ok();

    // DELETE /api/portfolio/v1/projects/spotlights/{id}
    let response = server
        .delete(&format!("/api/portfolio/v1/projects/spotlights/{spotlight_id}"))
        .await;
    response.assert_status_ok();

    // Cleanup project
    let _ = server
        .delete(&format!("/api/portfolio/v1/projects/{project_id}"))
        .await;
}

#[tokio::test]
async fn e2e_metrics_endpoint() {
    let server = test_app().await;
    let response = server.get("/metrics").await;
    response.assert_status_ok();
}
