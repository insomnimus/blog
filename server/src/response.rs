use axum::{
	http::StatusCode,
	response::Html,
};

use crate::xml::Xml;

pub const E400: (StatusCode, &str) = (StatusCode::BAD_REQUEST, "Bad request.");
pub const E404: (StatusCode, &str) = (StatusCode::NOT_FOUND, "Page not found.");
pub const E500: (StatusCode, &str) = (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong.");
pub const E503: (StatusCode, &str) = (StatusCode::SERVICE_UNAVAILABLE, "Service unavailable.");

type ErrorResponse = (StatusCode, &'static str);

pub trait ResponseExt<T>: Sized {
	fn or_code(self, resp: ErrorResponse) -> Result<T, (StatusCode, &'static str)>;

	fn or_400(self) -> Result<T, ErrorResponse> {
		self.or_code(E400)
	}

	fn or_404(self) -> Result<T, ErrorResponse> {
		self.or_code(E404)
	}

	fn or_500(self) -> Result<T, ErrorResponse> {
		self.or_code(E500)
	}

	fn or_503(self) -> Result<T, ErrorResponse> {
		self.or_code(E503)
	}
}

impl<T: Sized> ResponseExt<T> for Option<T> {
	fn or_code(self, resp: ErrorResponse) -> Result<T, (StatusCode, &'static str)> {
		self.ok_or(resp)
	}
}

impl<T: Sized, E: std::fmt::Display> ResponseExt<T> for Result<T, E> {
	fn or_code(self, resp: ErrorResponse) -> Result<T, (StatusCode, &'static str)> {
		match self {
			Ok(x) => Ok(x),
			Err(e) => {
				crate::prelude::error!(target: "", "{e}");
				Err(resp)
			}
		}
	}
}

pub trait ResultExt<T, E>: Sized {
	fn html(self) -> Result<Html<T>, E>;
	fn xml(self) -> Result<Xml<T>, E>;
}

impl<T: Sized, E> ResultExt<T, E> for Result<T, E> {
	fn html(self) -> Result<Html<T>, E> {
		self.map(Html)
	}

	fn xml(self) -> Result<Xml<T>, E> {
		self.map(Xml)
	}
}
