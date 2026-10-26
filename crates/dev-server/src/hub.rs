use anyhow::{Context, Result, ensure};
use base64::Engine;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub async fn request(
    client: &reqwest::Client,
    method: reqwest::Method,
    url: &str,
    body: Option<Value>,
    token: Option<&str>,
    expected: &[u16],
) -> Result<Value> {
    let mut request = client.request(method.clone(), url);
    if let Some(body) = body {
        request = request.json(&body);
    }
    if let Some(token) = token {
        request = request.bearer_auth(token);
    }
    let response = request
        .send()
        .await
        .with_context(|| format!("{method} {} failed", url.split('?').next().unwrap_or(url)))?;
    ensure!(
        expected.contains(&response.status().as_u16()),
        "{method} {}: HTTP {} (expected {expected:?})",
        url.split('?').next().unwrap_or(url),
        response.status().as_u16()
    );
    let body = response.bytes().await?;
    Ok(serde_json::from_slice(&body).unwrap_or(Value::Null))
}

pub async fn login(
    client: &reqwest::Client,
    base: &str,
    email: &str,
    password: &str,
    realm: &str,
) -> Result<Value> {
    request(client, reqwest::Method::POST, &format!("{base}/oidc/login"), Some(json!({"email":email, "password":password, "realm":realm, "client_id":if realm == "master" {"admin-ui"} else {"portfolio"}, "requested_scopes":if realm == "master" {vec!["openid","profile","admin"]} else {vec!["openid","profile","roles"]}})), None, &[200]).await
}

/// Exercise Authorization Code + S256, rather than using the password endpoint's
/// access token (the Hub's password flow does not emit access-token role claims).
async fn authorization_code_login(
    base: &str,
    redirect: &str,
    email: &str,
    password: &str,
) -> Result<Value> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(60))
        .build()?;
    let login = client.post(format!("{base}/realms/portfolio/login"))
        .json(&json!({"email":email,"password":password,"client_id":"portfolio","requested_scopes":["openid","profile","roles"]})).send().await?;
    ensure!(
        login.status().is_success(),
        "Hub sign-in failed: HTTP {}",
        login.status()
    );
    let mut cookies = std::collections::BTreeMap::new();
    remember_cookies(login.headers(), &mut cookies);
    let verifier = format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    );
    let state = uuid::Uuid::new_v4().simple().to_string();
    let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(Sha256::digest(verifier.as_bytes()));
    let mut authorize = reqwest::Url::parse(&format!(
        "{base}/realms/portfolio/protocol/openid-connect/auth"
    ))?;
    authorize.query_pairs_mut().extend_pairs([
        ("client_id", "portfolio"),
        ("response_type", "code"),
        ("redirect_uri", redirect),
        ("scope", "openid profile roles"),
        ("code_challenge", challenge.as_str()),
        ("code_challenge_method", "S256"),
        ("state", state.as_str()),
    ]);
    for _ in 0..4 {
        let cookie = cookies
            .iter()
            .map(|(name, value)| format!("{name}={value}"))
            .collect::<Vec<_>>()
            .join("; ");
        let response = client
            .get(authorize.clone())
            .header(reqwest::header::COOKIE, cookie)
            .send()
            .await?;
        remember_cookies(response.headers(), &mut cookies);
        if response.status().is_redirection() {
            let location = response
                .headers()
                .get(reqwest::header::LOCATION)
                .context("authorization redirect missing")?
                .to_str()?;
            let location = authorize.join(location)?;
            ensure!(
                location.as_str().starts_with(redirect),
                "Hub redirected away from the registered callback"
            );
            let params: std::collections::BTreeMap<_, _> =
                location.query_pairs().into_owned().collect();
            ensure!(
                params.get("state") == Some(&state),
                "authorization state mismatch"
            );
            let code = params.get("code").context("authorization code missing")?;
            let response = client
                .post(format!(
                    "{base}/realms/portfolio/protocol/openid-connect/token"
                ))
                .form(&[
                    ("grant_type", "authorization_code"),
                    ("client_id", "portfolio"),
                    ("redirect_uri", redirect),
                    ("code", code.as_str()),
                    ("code_verifier", verifier.as_str()),
                ])
                .send()
                .await?;
            ensure!(
                response.status().is_success(),
                "PKCE code exchange failed: HTTP {}",
                response.status()
            );
            return Ok(response.json().await?);
        }
        ensure!(
            response.status().is_success(),
            "authorization failed: HTTP {}",
            response.status()
        );
        let html = response.text().await?;
        let consent = html
            .split("name=\"consent_token\" value=\"")
            .nth(1)
            .and_then(|tail| tail.split('"').next())
            .context("Hub did not return an authorization code or consent form")?;
        authorize
            .query_pairs_mut()
            .append_pair("consent_action", "allow")
            .append_pair("consent_token", consent);
    }
    anyhow::bail!("too many authorization steps")
}

fn remember_cookies(
    headers: &reqwest::header::HeaderMap,
    cookies: &mut std::collections::BTreeMap<String, String>,
) {
    for value in headers.get_all(reqwest::header::SET_COOKIE) {
        if let Ok(value) = value.to_str()
            && let Some((name, value)) = value
                .split(';')
                .next()
                .and_then(|pair| pair.split_once('='))
        {
            cookies.insert(name.to_string(), value.to_string());
        }
    }
}

struct Admin<'a> {
    client: &'a reqwest::Client,
    base: &'a str,
    token: String,
}
impl Admin<'_> {
    async fn api(&self, path: &str, method: reqwest::Method, body: Option<Value>) -> Result<Value> {
        request(
            self.client,
            method,
            &format!("{}/api/{path}", self.base),
            body,
            Some(&self.token),
            &[200, 201, 204],
        )
        .await
    }
    async fn ensure(&self, path: &str, field: &str, value: &str, body: Value) -> Result<Value> {
        let existing = self.api(path, reqwest::Method::GET, None).await?;
        if let Some(item) = existing["items"].as_array().and_then(|items| {
            items
                .iter()
                .find(|item| item[field].as_str() == Some(value))
        }) {
            return Ok(item.clone());
        }
        self.api(
            path.split('?').next().unwrap(),
            reqwest::Method::POST,
            Some(body),
        )
        .await
    }
}

pub async fn provision(
    client: &reqwest::Client,
    base: &str,
    admin_url: &str,
    credentials: &crate::stack::Credentials,
) -> Result<()> {
    let session = login(
        client,
        base,
        &credentials.email,
        &credentials.password,
        "master",
    )
    .await?;
    let admin = Admin {
        client,
        base,
        token: session["access_token"]
            .as_str()
            .context("missing bootstrap access token")?
            .into(),
    };
    let realm = admin
        .ensure(
            "realms?limit=100",
            "name",
            "portfolio",
            json!({"name":"portfolio","display_name":"DJMX Portfolio","enabled":true}),
        )
        .await?;
    let rid = realm["id"].as_str().context("missing realm id")?;
    let body = json!({"realm_id":rid,"client_id":"portfolio","client_type":"public","name":"Portfolio administration","redirect_uris":[format!("{admin_url}/")],"post_logout_redirect_uris":[format!("{admin_url}/")],"allowed_scopes":["openid","profile","roles","portfolio-api"],"allowed_grant_types":["authorization_code","refresh_token"],"pkce_required":true,"token_endpoint_auth_method":"none","enabled":true});
    let app = admin
        .ensure(
            &format!("clients?realm_id={rid}&limit=100"),
            "client_id",
            "portfolio",
            body.clone(),
        )
        .await?;
    let cid = app["id"].as_str().context("missing client id")?;
    admin
        .api(&format!("clients/{cid}"), reqwest::Method::PUT, Some(body))
        .await?;
    let scope = admin
        .ensure(
            &format!("scopes?realm_id={rid}"),
            "name",
            "portfolio-api",
            json!({"realm_id":rid,"name":"portfolio-api","enabled":true}),
        )
        .await?;
    let sid = scope["id"].as_str().context("missing scope id")?;
    admin.ensure(&format!("scopes/{sid}/mappers"), "name", "api-audience", json!({"name":"api-audience","mapper_type":"audience","claim_value":"portfolio-api","add_to_access_token":true,"add_to_id_token":false,"add_to_userinfo":false})).await?;
    let assigned = admin
        .api(&format!("clients/{cid}/scopes"), reqwest::Method::GET, None)
        .await?;
    if !assigned["items"].as_array().is_some_and(|items| {
        items
            .iter()
            .any(|item| item["scope_id"].as_str().or(item["id"].as_str()) == Some(sid))
    }) {
        admin
            .api(
                &format!("clients/{cid}/scopes"),
                reqwest::Method::POST,
                Some(json!({"scope_id":sid,"assignment_type":"default"})),
            )
            .await?;
    }
    let role = admin
        .ensure(
            &format!("roles?realm_id={rid}"),
            "name",
            "admin",
            json!({"realm_id":rid,"name":"admin","permissions":[]}),
        )
        .await?;
    for (email, name, grant) in [
        (&credentials.email, "portfolio-admin", true),
        (&credentials.reader_email, "portfolio-reader", false),
    ] {
        let user = admin.ensure(&format!("users?realm_id={rid}&limit=100"), "email", email, json!({"realm_id":rid,"email":email,"password":credentials.password,"username":name,"enabled":true})).await?;
        let uid = user["id"].as_str().context("missing user id")?;
        if grant {
            let roles = admin
                .api(&format!("users/{uid}/roles"), reqwest::Method::GET, None)
                .await?;
            if !roles["items"]
                .as_array()
                .is_some_and(|items| items.iter().any(|r| r["id"] == role["id"]))
            {
                admin
                    .api(
                        &format!("users/{uid}/roles"),
                        reqwest::Method::POST,
                        Some(json!({"role_id":role["id"]})),
                    )
                    .await?;
            }
        }
    }
    Ok(())
}

pub async fn smoke(
    state: &crate::stack::Runtime,
    credentials: &crate::stack::Credentials,
) -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()?;
    for app in ["hub", "admin", "portfolio"] {
        for path in ["/", "/js/index.js"] {
            request(
                &client,
                reqwest::Method::GET,
                &format!("{}{path}", state.urls[app]),
                None,
                None,
                &[200],
            )
            .await?;
        }
    }
    for path in [
        "/api/about/v1/me",
        "/api/contact/v1/information",
        "/api/portfolio/v2/projects?page=1&size=10&visible=true",
    ] {
        request(
            &client,
            reqwest::Method::GET,
            &format!("{}{path}", state.urls["portfolio"]),
            None,
            None,
            &[200],
        )
        .await?;
    }
    let private = format!("{}/api/portfolio/v1/projects", state.urls["admin"]);
    request(
        &client,
        reqwest::Method::POST,
        &private,
        Some(json!({"title":"Forbidden"})),
        None,
        &[401],
    )
    .await?;
    let admin = authorization_code_login(
        &state.urls["hub"],
        &format!("{}/", state.urls["admin"]),
        &credentials.email,
        &credentials.password,
    )
    .await?;
    let reader = authorization_code_login(
        &state.urls["hub"],
        &format!("{}/", state.urls["admin"]),
        &credentials.reader_email,
        &credentials.password,
    )
    .await?;
    request(
        &client,
        reqwest::Method::POST,
        &private,
        Some(json!({"title":"Forbidden"})),
        reader["access_token"].as_str(),
        &[403],
    )
    .await?;
    let token = admin["access_token"]
        .as_str()
        .context("missing access token")?;
    request(
        &client,
        reqwest::Method::GET,
        &private,
        None,
        Some(token),
        &[200],
    )
    .await?;
    let project = request(
        &client,
        reqwest::Method::POST,
        &private,
        Some(json!({"title":"Stack smoke test","subTitle":"WASI","client":"Local test"})),
        Some(token),
        &[200],
    )
    .await?;
    let id = project["id"].as_str().context("missing project id")?;
    let project_url = format!("{private}/{id}");
    let result: Result<()> = async {
        let multipart = "--djmx-smoke\r\nContent-Disposition: form-data; name=\"photo\"; filename=\"stack-smoke.txt\"\r\nContent-Type: text/plain\r\n\r\nDJMX stack storage smoke test\r\n--djmx-smoke--\r\n";
        let uploaded = client.patch(format!("{project_url}/contents"))
            .bearer_auth(token).header("Content-Type", "multipart/form-data; boundary=djmx-smoke")
            .body(multipart).send().await?;
        ensure!(uploaded.status().is_success(), "media upload failed: HTTP {}", uploaded.status());
        let uploaded: Value = uploaded.json().await?;
        ensure!(uploaded.as_array().is_some_and(|items| !items.is_empty()), "media upload returned no content");
        request(&client, reqwest::Method::PUT, &project_url, Some(json!({"metadata":{"title":"Stack smoke updated","subTitle":"","client":""},"visible":true,"adult":false})), Some(token), &[200]).await?;
        let project = request(&client, reqwest::Method::GET, &project_url, None, None, &[200]).await?;
        let media_url = project["contents"][0]["url"].as_str().context("missing media download URL")?;
        let downloaded = client.get(media_url).send().await?;
        ensure!(downloaded.status().is_success(), "media download failed");
        ensure!(downloaded.text().await? == "DJMX stack storage smoke test", "media content changed");
        let refreshed = client.post(format!("{}/realms/portfolio/protocol/openid-connect/token",state.urls["hub"])).form(&[("grant_type","refresh_token"),("client_id","portfolio"),("refresh_token",admin["refresh_token"].as_str().context("missing refresh token")?)]).send().await?;
        ensure!(refreshed.status().is_success(), "refresh token rejected");
        let refreshed: Value = refreshed.json().await?;
        request(&client, reqwest::Method::GET, &private, None, refreshed["access_token"].as_str(), &[200]).await?;
        Ok(())
    }.await;
    let cleanup = request(
        &client,
        reqwest::Method::DELETE,
        &project_url,
        None,
        Some(token),
        &[200, 204],
    )
    .await;
    result?;
    cleanup?;
    println!(
        "PASS: frontends, real Hub login/JWKS, public API, 401/403, project CRUD, S3 upload/download and refresh"
    );
    Ok(())
}
