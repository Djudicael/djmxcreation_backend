use aws_sdk_s3::primitives::{ByteStream, SdkBody};
use aws_smithy_runtime_api::client::http::{
    HttpClient, HttpConnector, HttpConnectorFuture, HttpConnectorSettings, SharedHttpConnector,
};
use aws_smithy_runtime_api::client::orchestrator::HttpRequest;
use aws_smithy_runtime_api::client::result::ConnectorError;
use aws_smithy_runtime_api::client::runtime_components::RuntimeComponents;
use aws_smithy_runtime_api::http::{Response as SmithyResponse, StatusCode};

/// A custom HTTP client for AWS SDK that routes requests through WASI 0.2 `wasi:http`
#[derive(Debug, Clone)]
pub struct WasiHttpClient;

impl WasiHttpClient {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WasiHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpConnector for WasiHttpClient {
    fn call(&self, request: HttpRequest) -> HttpConnectorFuture {
        let future = async move {
            // 1. Extract parts from the AWS Smithy Request
            let uri = request.uri().to_string();
            let method_string = request.method().to_string();
            let method = method_string.as_str();
            let headers_map = request.headers().clone();
            let body = request.into_body();

            // 2. Map HTTP Method
            let wasi_method = match method {
                "GET" => wasi::http::types::Method::Get,
                "POST" => wasi::http::types::Method::Post,
                "PUT" => wasi::http::types::Method::Put,
                "DELETE" => wasi::http::types::Method::Delete,
                "HEAD" => wasi::http::types::Method::Head,
                "OPTIONS" => wasi::http::types::Method::Options,
                "PATCH" => wasi::http::types::Method::Patch,
                "CONNECT" => wasi::http::types::Method::Connect,
                "TRACE" => wasi::http::types::Method::Trace,
                other => wasi::http::types::Method::Other(other.to_string()),
            };

            // 3. Map Headers
            let headers = wasi::http::types::Fields::new();
            for (name, value) in headers_map.iter() {
                let name = name.to_string();
                let val = value.to_string().into_bytes();
                headers.append(&name, &val).map_err(|e| {
                    ConnectorError::other(format!("Failed to append header: {:?}", e).into(), None)
                })?;
            }

            // 4. Parse URI
            let parsed_uri = url::Url::parse(&uri)
                .map_err(|e| ConnectorError::other(format!("Invalid URL: {:?}", e).into(), None))?;

            let scheme = match parsed_uri.scheme() {
                "http" => Some(wasi::http::types::Scheme::Http),
                "https" => Some(wasi::http::types::Scheme::Https),
                other => Some(wasi::http::types::Scheme::Other(other.to_string())),
            };

            let authority = parsed_uri.host_str().map(|s: &str| {
                if let Some(port) = parsed_uri.port() {
                    format!("{}:{}", s, port)
                } else {
                    s.to_string()
                }
            });

            let path_and_query = match parsed_uri.query() {
                Some(query) => format!("{}?{}", parsed_uri.path(), query),
                None => parsed_uri.path().to_string(),
            };

            // 5. Construct WASI Outgoing Request
            let wasi_request = wasi::http::types::OutgoingRequest::new(headers);
            wasi_request.set_method(&wasi_method).unwrap();
            wasi_request.set_scheme(scheme.as_ref()).unwrap();
            wasi_request.set_authority(authority.as_deref()).unwrap();
            wasi_request
                .set_path_with_query(Some(&path_and_query))
                .unwrap();

            // 6. Write Body
            // Extract the bytes from SdkBody (For AWS S3, payload is usually converted to bytes)
            // Note: In a fully streaming production setup, this would incrementally pipe chunks.
            // For AWS SDK on Wasm, resolving to memory first is standard to avoid async blocking issues.
            let req_body_bytes = ByteStream::new(body)
                .collect()
                .await
                .map_err(|e| {
                    ConnectorError::other(
                        format!("Failed to read request body: {}", e).into(),
                        None,
                    )
                })?
                .into_bytes();

            if !req_body_bytes.is_empty() {
                let outgoing_body = wasi_request.body().unwrap();
                let body_stream = outgoing_body.write().unwrap();
                body_stream
                    .blocking_write_and_flush(&req_body_bytes)
                    .map_err(|e| {
                        ConnectorError::other(
                            format!("Failed to write WASI body: {:?}", e).into(),
                            None,
                        )
                    })?;
                drop(body_stream);
                wasi::http::types::OutgoingBody::finish(outgoing_body, None).map_err(|e| {
                    ConnectorError::other(
                        format!("Failed to finish WASI body: {:?}", e).into(),
                        None,
                    )
                })?;
            }

            // 7. Dispatch Request via WASI Host
            let future_response = wasi::http::outgoing_handler::handle(wasi_request, None)
                .map_err(|e| {
                    ConnectorError::other(
                        format!("WASI HTTP dispatch failed: {:?}", e).into(),
                        None,
                    )
                })?;

            // 8. Poll for Response
            let pollable = future_response.subscribe();
            wasi::io::poll::poll(&[&pollable]);

            let incoming_response = future_response
                .get()
                .expect("WASI HTTP future was ready but returned None")
                .expect("WASI HTTP future failed")
                .map_err(|e| {
                    ConnectorError::other(format!("WASI HTTP response error: {:?}", e).into(), None)
                })?;

            // 9. Read Response Status and Headers
            let status = incoming_response.status();
            let res_headers = incoming_response.headers().entries();

            // 10. Read Response Body
            let incoming_body = incoming_response.consume().unwrap();
            let body_stream = incoming_body.stream().unwrap();

            let mut response_bytes = Vec::new();
            loop {
                match body_stream.blocking_read(8192) {
                    Ok(chunk) => {
                        response_bytes.extend_from_slice(&chunk);
                    }
                    Err(wasi::io::streams::StreamError::Closed) => {
                        break;
                    }
                    Err(wasi::io::streams::StreamError::LastOperationFailed(e)) => {
                        return Err(ConnectorError::other(
                            format!("WASI stream read failed: {:?}", e).into(),
                            None,
                        ));
                    }
                }
            }

            drop(body_stream);
            wasi::http::types::IncomingBody::finish(incoming_body);

            // 11. Build Final Smithy Response
            let mut final_response = SmithyResponse::new(
                StatusCode::try_from(status).unwrap_or(StatusCode::try_from(500).unwrap()),
                SdkBody::from(response_bytes),
            );

            for (name, val) in res_headers {
                if let Ok(value_str) = String::from_utf8(val) {
                    final_response.headers_mut().insert(name, value_str);
                }
            }

            Ok(final_response)
        };

        HttpConnectorFuture::new(future)
    }
}

impl HttpClient for WasiHttpClient {
    fn http_connector(
        &self,
        _settings: &HttpConnectorSettings,
        _components: &RuntimeComponents,
    ) -> SharedHttpConnector {
        SharedHttpConnector::new(self.clone())
    }
}
