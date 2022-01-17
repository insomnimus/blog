pub trait DefaultExt {
	fn take(&mut self) -> Self;
}

impl<T: Default> DefaultExt for T {
	fn take(&mut self) -> Self {
		std::mem::take(self)
	}
}
