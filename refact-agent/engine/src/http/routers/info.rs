use indexmap::IndexMap;

use axum::http::Response;
use hyper::Body;
use serde_json::json;

use crate::custom_error::ScratchError;


pub fn get_build_info() -> IndexMap<&'static str, &'static str> {
    IndexMap::from([
        ("version", crate::version::build::PKG_VERSION),
        ("commit", crate::version::build::COMMIT_HASH),
        ("build_os", crate::version::build::BUILD_OS),
        ("rust_version", crate::version::build::RUST_VERSION),
        ("cargo_version", crate::version::build::CARGO_VERSION),
    ])
}

pub async fn handle_info() -> axum::response::Result<Response<Body>, ScratchError> {
    Ok(Response::builder()
        .header("Content-Type", "application/json")
        .body(Body::from(json!(get_build_info()).to_string()))
        .unwrap())
}
