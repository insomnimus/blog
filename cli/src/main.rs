mod about;
mod app;
mod article;
mod cmd;
mod display;
mod ext;
mod media;
mod music;
mod post;
mod prelude;
mod render;
mod utility;

#[tokio::main]
async fn main() {
	if let Err(e) = app::run().await {
		eprintln!("error: {e:?}");
		std::process::exit(1);
	}
}
