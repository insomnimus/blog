use axum::{
	http::StatusCode,
	response::Html,
};

pub trait ResultExt<T: Sized, E> {
	fn or_code_with(self, code: StatusCode, msg: impl AsRef<str>) -> Result<T, StatusCode>;
	fn html(self) -> Result<Html<T>, E>;

	fn or_code(self, code: StatusCode) -> Result<T, StatusCode>
	where
		Self: Sized,
	{
		self.or_code_with(code, "")
	}

	fn or_404_with(self, msg: impl AsRef<str>) -> Result<T, StatusCode>
	where
		Self: Sized,
	{
		self.or_code_with(StatusCode::NOT_FOUND, msg)
	}

	fn or_404(self) -> Result<T, StatusCode>
	where
		Self: Sized,
	{
		self.or_code(StatusCode::NOT_FOUND)
	}

	fn or_503_with(self, msg: impl AsRef<str>) -> Result<T, StatusCode>
	where
		Self: Sized,
	{
		self.or_code_with(StatusCode::INTERNAL_SERVER_ERROR, msg)
	}

	fn or_503(self) -> Result<T, StatusCode>
	where
		Self: Sized,
	{
		self.or_code(StatusCode::INTERNAL_SERVER_ERROR)
	}
}

impl<T: Sized, E: std::fmt::Display> ResultExt<T, E> for Result<T, E> {
	fn or_code_with(self, code: StatusCode, msg: impl AsRef<str>) -> Result<T, StatusCode> {
		self.map_err(|e| {
			crate::prelude::error!("{} {}", msg.as_ref(), e);
			code
		})
	}

	fn or_code(self, code: StatusCode) -> Result<T, StatusCode> {
		self.map_err(|e| {
			crate::prelude::error!("{}", e);
			code
		})
	}

	fn html(self) -> Result<Html<T>, E> {
		self.map(Html)
	}
}

pub trait OptionExt<T: Sized> {
	fn or_404(self) -> Result<T, StatusCode>;
	fn html(self) -> Option<Html<T>>;
}

impl<T: Sized> OptionExt<T> for Option<T> {
	fn or_404(self) -> Result<T, StatusCode> {
		self.ok_or(StatusCode::NOT_FOUND)
	}

	fn html(self) -> Option<Html<T>> {
		self.map(Html)
	}
}
