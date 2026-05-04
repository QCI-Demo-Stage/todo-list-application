use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};

#[derive(Clone)]
pub struct AppState {
    docs_enabled: bool,
    openapi_spec: Arc<[u8]>,
    swagger_index_html: Arc<[u8]>,
}

pub fn load_assets(workspace_root: &std::path::Path) -> std::io::Result<(Vec<u8>, Vec<u8>)> {
    let spec_path = workspace_root.join("api-spec.yaml");
    let swagger_path = workspace_root.join("swagger").join("index.html");
    let openapi_spec = std::fs::read(spec_path)?;
    let swagger_index_html = std::fs::read(swagger_path)?;
    Ok((openapi_spec, swagger_index_html))
}

pub fn docs_enabled_for_env(deploy_env: &str) -> bool {
    deploy_env == "development" || deploy_env == "staging"
}

pub fn app_for_env(deploy_env: &str, openapi_spec: Vec<u8>, swagger_index_html: Vec<u8>) -> Router {
    let state = AppState {
        docs_enabled: docs_enabled_for_env(deploy_env),
        openapi_spec: Arc::from(openapi_spec.into_boxed_slice()),
        swagger_index_html: Arc::from(swagger_index_html.into_boxed_slice()),
    };

    Router::new()
        .route("/health", get(health))
        .fallback(fallback)
        .with_state(state)
}

async fn health() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "application/json")],
        r#"{"status":"ok"}"#,
    )
}

async fn fallback(State(state): State<AppState>, req: axum::http::Request<Body>) -> Response {
    if *req.method() != Method::GET {
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    }

    let path = req.uri().path();

    if !state.docs_enabled {
        return StatusCode::NOT_FOUND.into_response();
    }

    match path {
        "/api-spec.yaml" => {
            let mut res = Response::new(Body::from(state.openapi_spec.as_ref().to_vec()));
            res.headers_mut().insert(
                axum::http::header::CONTENT_TYPE,
                HeaderValue::from_static("application/yaml; charset=utf-8"),
            );
            res
        }
        "/api-docs" => {
            let mut res = Response::new(Body::empty());
            *res.status_mut() = StatusCode::MOVED_PERMANENTLY;
            res.headers_mut().insert(
                axum::http::header::LOCATION,
                HeaderValue::from_static("/api-docs/"),
            );
            res
        }
        p if p == "/api-docs/" || p.starts_with("/api-docs/") => {
            let mut res = Response::new(Body::from(state.swagger_index_html.as_ref().to_vec()));
            res.headers_mut().insert(
                axum::http::header::CONTENT_TYPE,
                HeaderValue::from_static("text/html; charset=utf-8"),
            );
            res
        }
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

pub fn deploy_env_from_process() -> String {
    std::env::var("NODE_ENV").unwrap_or_else(|_| "staging".to_string())
}

pub fn listen_port_from_process() -> u16 {
    let raw = std::env::var("PORT").ok();
    let Some(raw) = raw else {
        return 8000;
    };
    raw.parse::<u16>().ok().filter(|&n| n > 0).unwrap_or(8000)
}

pub fn workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::Request;
    use tower::ServiceExt;

    fn sample_spec() -> Vec<u8> {
        b"openapi: 3.0.3\n".to_vec()
    }

    fn sample_swagger() -> Vec<u8> {
        b"<!DOCTYPE html><html></html>".to_vec()
    }

    #[tokio::test]
    async fn get_health_returns_json_ok() {
        let app = app_for_env("staging", sample_spec(), sample_swagger());
        let res = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(
            res.headers()
                .get(axum::http::header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok()),
            Some("application/json")
        );
        let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
        assert_eq!(body.as_ref(), br#"{"status":"ok"}"#);
    }

    #[tokio::test]
    async fn docs_routes_404_when_production() {
        let app = app_for_env("production", sample_spec(), sample_swagger());
        let spec = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api-spec.yaml")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(spec.status(), StatusCode::NOT_FOUND);

        let docs = app
            .oneshot(
                Request::builder()
                    .uri("/api-docs/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(docs.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn get_api_docs_redirects_to_slash() {
        let app = app_for_env("staging", sample_spec(), sample_swagger());
        let res = app
            .oneshot(
                Request::builder()
                    .uri("/api-docs")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::MOVED_PERMANENTLY); // HTTP 301
        assert_eq!(
            res.headers().get(axum::http::header::LOCATION),
            Some(&HeaderValue::from_static("/api-docs/"))
        );
    }
}
