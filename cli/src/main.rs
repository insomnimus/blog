mod app;
mod article;
mod display;
mod post;
mod prelude;
mod render;
mod utility;

#[tokio::main(flavor = "current_thread")]
async fn main() {
	if let Err(e) = app::run().await {
		eprintln!("error: {}", e);
		std::process::exit(1);
	}
}
