use axum::{
	http::StatusCode,
	response::Html,
};

pub const E404: &str = "Page not found.";
pub const E500: &str = "Something went wrong.";
pub const E503: &str = "Service unavailable.";

type ErrorResponse = (StatusCode, &'static str);

pub trait ResponseExt<T>: Sized {
	fn or_code(self, code: StatusCode, body: &'static str)
		-> Result<T, (StatusCode, &'static str)>;

	fn or_404(self) -> Result<T, ErrorResponse> {
		self.or_code(StatusCode::NOT_FOUND, E404)
	}

	fn or_500(self) -> Result<T, ErrorResponse> {
		self.or_code(StatusCode::INTERNAL_SERVER_ERROR, E500)
	}

	fn or_503(self) -> Result<T, ErrorResponse> {
		self.or_code(StatusCode::SERVICE_UNAVAILABLE, E503)
	}
}

impl<T: Sized> ResponseExt<T> for Option<T> {
	fn or_code(
		self,
		code: StatusCode,
		body: &'static str,
	) -> Result<T, (StatusCode, &'static str)> {
		self.ok_or((code, body))
	}
}

impl<T: Sized, E: std::fmt::Display> ResponseExt<T> for Result<T, E> {
	fn or_code(
		self,
		code: StatusCode,
		body: &'static str,
	) -> Result<T, (StatusCode, &'static str)> {
		match self {
			Ok(x) => Ok(x),
			Err(e) => {
				crate::prelude::error!("{}", e);
				Err((code, body))
			}
		}
	}
}

pub trait ResultExt<T, E>: Sized {
	fn html(self) -> Result<Html<T>, E>;
}

impl<T: Sized, E> ResultExt<T, E> for Result<T, E> {
	fn html(self) -> Result<Html<T>, E> {
		self.map(Html)
	}
}
