use axum::{
    Router,
    body::{Body, to_bytes},
    extract::{Request, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use tokio::task::JoinHandle;

#[derive(Clone)]
struct Proxy {
    client: reqwest::Client,
    frontend: u16,
    backend: u16,
    hub: bool,
}

fn backend_path(path: &str, hub: bool) -> bool {
    let parts: Vec<_> = path.trim_matches('/').split('/').collect();
    let realm_protocol = parts.len() >= 3
        && parts[0] == "realms"
        && matches!(
            parts[2],
            "login" | "protocol" | ".well-known" | "account" | "organization"
        );
    if hub {
        path.starts_with("/api/")
            || path.starts_with("/oidc/")
            || path.starts_with("/.well-known/")
            || path.starts_with("/health")
            || path.starts_with("/account")
            || realm_protocol
    } else {
        path.starts_with("/api/") || path == "/ping"
    }
}

fn strip_hop_headers(headers: &mut HeaderMap) {
    let connection = headers
        .get(header::CONNECTION)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    for name in connection.split(',').map(str::trim) {
        if !name.is_empty() {
            headers.remove(name);
        }
    }
    for name in [
        "connection",
        "keep-alive",
        "proxy-authenticate",
        "proxy-authorization",
        "te",
        "trailer",
        "transfer-encoding",
        "upgrade",
    ] {
        headers.remove(name);
    }
}

async fn forward(State(proxy): State<Proxy>, request: Request) -> Response {
    let target = if backend_path(request.uri().path(), proxy.hub) {
        proxy.backend
    } else {
        proxy.frontend
    };
    let (mut parts, body) = request.into_parts();
    strip_hop_headers(&mut parts.headers);
    let body = match to_bytes(body, 64 * 1024 * 1024).await {
        Ok(body) => body,
        Err(_) => return StatusCode::PAYLOAD_TOO_LARGE.into_response(),
    };
    let url = format!(
        "http://127.0.0.1:{target}{}",
        parts
            .uri
            .path_and_query()
            .map(|pq| pq.as_str())
            .unwrap_or("/")
    );
    let upstream = match proxy
        .client
        .request(parts.method, url)
        .headers(parts.headers)
        .body(body)
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => return (StatusCode::BAD_GATEWAY, "stack service unavailable").into_response(),
    };
    let status = upstream.status();
    let mut headers = upstream.headers().clone();
    strip_hop_headers(&mut headers);
    let body = match upstream.bytes().await {
        Ok(body) => body,
        Err(_) => return StatusCode::BAD_GATEWAY.into_response(),
    };
    let mut response = Response::new(Body::from(body));
    *response.status_mut() = status;
    *response.headers_mut() = headers;
    response
}

pub async fn start(
    port: u16,
    frontend: u16,
    backend: u16,
    hub: bool,
) -> anyhow::Result<JoinHandle<()>> {
    let state = Proxy {
        client: reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(std::time::Duration::from_secs(60))
            .build()?,
        frontend,
        backend,
        hub,
    };
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port)).await?;
    let app = Router::new().fallback(forward).with_state(state);
    Ok(tokio::spawn(async move {
        if let Err(error) = axum::serve(listener, app).await {
            tracing::error!(%error, "stack proxy failed");
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn proxy_routes_protocol_and_api_calls_preserving_cookies_and_status() {
        async fn backend() -> Response {
            let mut response = (StatusCode::UNAUTHORIZED, "backend").into_response();
            response
                .headers_mut()
                .append(header::SET_COOKIE, "one=1; HttpOnly".parse().unwrap());
            response
                .headers_mut()
                .append(header::SET_COOKIE, "two=2; Secure".parse().unwrap());
            response
        }
        let back = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let back_port = back.local_addr().unwrap().port();
        let front = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let front_port = front.local_addr().unwrap().port();
        let back_task = tokio::spawn(async move {
            axum::serve(back, Router::new().fallback(backend))
                .await
                .unwrap();
        });
        let front_task = tokio::spawn(async move {
            axum::serve(front, Router::new().fallback(|| async { "frontend" }))
                .await
                .unwrap();
        });
        let port = crate::stack::spare_port().unwrap();
        let task = start(port, front_port, back_port, true).await.unwrap();
        let client = reqwest::Client::new();
        for path in [
            "/api/users",
            "/realms/portfolio/protocol/openid-connect/token",
            "/realms/portfolio/login",
            "/realms/portfolio/organization/identity-provider?email=test@example.com",
        ] {
            let response = client
                .get(format!("http://127.0.0.1:{port}{path}"))
                .send()
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
            assert_eq!(
                response
                    .headers()
                    .get_all(header::SET_COOKIE)
                    .iter()
                    .count(),
                2
            );
        }
        assert_eq!(
            client
                .get(format!("http://127.0.0.1:{port}/realms/example"))
                .send()
                .await
                .unwrap()
                .text()
                .await
                .unwrap(),
            "frontend"
        );
        task.abort();
        back_task.abort();
        front_task.abort();
    }
}
