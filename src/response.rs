use axum::{
	http::StatusCode,
	response::IntoResponse,
};

static NOT_FOUND: &str = include_str!("../templates/not_found.html");

pub enum Response<T: IntoResponse> {
	NotFound,
	Found(T),
}

impl<T: IntoResponse> IntoResponse for Response<T> {
	fn into_response(self) -> axum::Response {
		match self {
			Self::NotFound => (StatusCode::NOT_FOUND, NOT_FOUND).into_response(),
			Self::Found(x) => x.into_response(),
		}
	}
}

impl<T: IntoResponse> From<Option<T>> for Response<T> {
	fn from(o: Option<T>) -> Self {
		o.map(Self::found).unwrap_or(Self::NotFound)
	}
}
