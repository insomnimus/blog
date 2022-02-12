use axum::{
	body::{
		self,
		Bytes,
		Full,
	},
	http::{
		header,
		HeaderValue,
	},
	response::{
		IntoResponse,
		Response,
	},
};

#[derive(Debug, Clone, Copy)]
pub struct Xml<T>(pub T);

impl<T> IntoResponse for Xml<T>
where
	T: Into<Full<Bytes>>,
{
	fn into_response(self) -> Response {
		let mut res = Response::new(body::boxed(self.0.into()));
		res.headers_mut().insert(
			header::CONTENT_TYPE,
			HeaderValue::from_static("application/xml"),
		);
		res
	}
}
