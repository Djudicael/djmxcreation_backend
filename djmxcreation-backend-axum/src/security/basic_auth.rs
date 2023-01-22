use app_config::security_config::SecurityConfig;
use axum::{
    headers::{authorization::Basic, Authorization},
    middleware::Next,
    response::Response,
    TypedHeader,
};
use hyper::{Request, StatusCode};

#[derive(Clone)]pub struct BasicAuth {
    pub security_config: SecurityConfig,
}

impl BasicAuth {
    pub fn new(security_config: SecurityConfig) -> Self {
        Self { security_config }
    }

    pub async fn auth<B>(
        &self,
        // run the `TypedHeader` extractor
        TypedHeader(auth): TypedHeader<Authorization<Basic>>,
        // you can also add more extractors here but the last
        // extractor must implement `FromRequest` which
        // `Request` does
        request: Request<B>,
        next: Next<B>,
    ) -> Result<Response, StatusCode> {
        if self.token_is_valid(auth.0) {
            let response = next.run(request).await;
            Ok(response)
        } else {
            Err(StatusCode::UNAUTHORIZED)
        }
    }

    fn token_is_valid(&self, token: Basic) -> bool {
        let SecurityConfig { username, password } = &self.security_config;

        username == token.username() && password == token.password()
    }
}
