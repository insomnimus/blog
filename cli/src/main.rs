mod about;
mod app;
mod article;
mod display;
mod ext;
mod music;
mod post;
mod prelude;
mod render;
mod sftp;
mod utility;

#[tokio::main]
async fn main() {
	if let Err(e) = app::run().await {
		eprintln!("error: {e}");
		std::process::exit(1);
	}
}
