use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderValue, Method, StatusCode},
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

/// Project root (`Cargo.toml`), so asset paths resolve reliably when tests run outside the cwd.
pub fn workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Unset **`NODE_ENV`** behaves like **staging** (prior deployment story).
pub fn deploy_env_from_process() -> String {
    std::env::var("NODE_ENV").unwrap_or_else(|_| "staging".to_string())
}

/// Default port **8000**; invalid or missing **`PORT`** falls back to **8000**.
pub fn listen_port_from_process() -> u16 {
    std::env::var("PORT")
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .filter(|p| *p > 0)
        .unwrap_or(8000)
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
        [(header::CONTENT_TYPE, "application/json")],
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
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/yaml; charset=utf-8"),
            );
            res
        }
        "/api-docs" => {
            let mut res = Response::new(Body::empty());
            *res.status_mut() = StatusCode::MOVED_PERMANENTLY;
            res.headers_mut()
                .insert(header::LOCATION, HeaderValue::from_static("/api-docs/"));
            res
        }
        p if p == "/api-docs/" || p.starts_with("/api-docs/") => {
            let mut res = Response::new(Body::from(state.swagger_index_html.as_ref().to_vec()));
            res.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/html; charset=utf-8"),
            );
            res
        }
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    fn fixture_assets() -> (Vec<u8>, Vec<u8>) {
        (
            b"openapi: 3.0.3\n".to_vec(),
            b"<!DOCTYPE html><html></html>".to_vec(),
        )
    }

    #[tokio::test]
    async fn health_returns_json_ok() {
        let (spec, swagger) = fixture_assets();
        let app = app_for_env("staging", spec, swagger);
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
                .get(header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok()),
            Some("application/json")
        );
        let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
        assert_eq!(body.as_ref(), br#"{"status":"ok"}"#);
    }

    #[tokio::test]
    async fn production_hides_docs() {
        let (spec, swagger) = fixture_assets();
        let app = app_for_env("production", spec, swagger);
        for uri in ["/api-spec.yaml", "/api-docs/", "/api-docs"] {
            let res = app
                .clone()
                .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(res.status(), StatusCode::NOT_FOUND, "uri={uri}");
        }
    }

    #[tokio::test]
    async fn staging_serves_spec_and_docs() {
        let (spec, swagger) = fixture_assets();
        let app = app_for_env("staging", spec.clone(), swagger.clone());

        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api-spec.yaml")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api-docs/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn api_docs_redirects_to_slash() {
        let app = app_for_env("staging", fixture_assets().0, fixture_assets().1);
        let res = app
            .oneshot(
                Request::builder()
                    .uri("/api-docs")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::MOVED_PERMANENTLY);
        assert_eq!(
            res.headers().get(header::LOCATION),
            Some(&HeaderValue::from_static("/api-docs/"))
        );
    }

    #[tokio::test]
    async fn post_returns_405() {
        let app = app_for_env("staging", fixture_assets().0, fixture_assets().1);
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api-docs/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn development_enables_docs() {
        let (spec, swagger) = fixture_assets();
        let app = app_for_env("development", spec, swagger);
        let res = app
            .oneshot(
                Request::builder()
                    .uri("/api-spec.yaml")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }
}
