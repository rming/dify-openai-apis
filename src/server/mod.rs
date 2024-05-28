mod helper;
mod v1_handlers;

use axum::{
    http::HeaderMap,
    middleware,
    routing::{get, post},
    Router,
};
use dify_client::http::Method;
use std::collections::HashMap;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use v1_handlers::*;

pub use helper::AppState;

async fn html_handler() -> (HeaderMap, &'static [u8]) {
    let headers = HashMap::from([("Content-Type".to_string(), "application/json".to_string())]);
    ((&headers).try_into().unwrap(), "{}".as_bytes())
}

pub fn app_routes() -> Router<AppState> {
    let cors = CorsLayer::new()
        .allow_headers(Any)
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
        .allow_origin(Any);

    let v1_routes = Router::new()
        .route("/chat/completions", post(chat_completions_handler))
        .route_layer(middleware::from_fn(check_method))
        .layer(ServiceBuilder::new().layer(cors));

    let app_routes = Router::new()
        .route("/", get(html_handler))
        .nest("/v1", v1_routes);
    app_routes
}
