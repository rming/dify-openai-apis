use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use dify_client::Client as DifyClient;

#[derive(Clone)]
pub struct AppState {
    pub dify: DifyClient,
}

pub struct AppError(anyhow::Error);
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(header::CONTENT_TYPE, "application/json")],
            format!("{{\"error\": \"{}\"}}", self.0.to_string()),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
