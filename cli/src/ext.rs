use sqlx::types::chrono::{
	DateTime,
	NaiveDateTime,
	Utc,
};

pub trait DateTimeExt {
	type Output;
	fn to_local(&self) -> Self::Output;
}

impl DateTimeExt for DateTime<Utc> {
	type Output = String;

	fn to_local(&self) -> String {
		self.format("%x:%H:%M").to_string()
	}
}

impl DateTimeExt for Option<DateTime<Utc>> {
	type Output = Option<String>;

	fn to_local(&self) -> Option<String> {
		self.as_ref().map(|d| d.to_local())
	}
}

impl DateTimeExt for NaiveDateTime {
	type Output = String;

	fn to_local(&self) -> String {
		DateTime::from_utc(*self, Utc).to_local()
	}
}

impl DateTimeExt for Option<NaiveDateTime> {
	type Output = Option<String>;

	fn to_local(&self) -> Option<String> {
		self.map(|d| d.to_local())
	}
}

pub trait DefaultExt {
	fn take(&mut self) -> Self;
}

impl<T: Default> DefaultExt for T {
	fn take(&mut self) -> Self {
		std::mem::take(self)
	}
}
