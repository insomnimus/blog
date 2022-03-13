mod about;
mod app;
mod article;
mod cmd;
mod display;
mod editor;
mod ext;
mod gc;
mod media;
mod music;
mod note;
mod prelude;
mod render;
mod syntax;
mod utility;

#[tokio::main]
async fn main() {
	if let Err(e) = app::run().await {
		eprintln!("error: {e:?}");
		std::process::exit(1);
	}
}
