use std::{
	fs,
	io::{
		self,
		Read,
	},
};

use crate::prelude::*;

pub fn app() -> App {
	App::new("render")
	.about("Render a document to html.")
	.args(&[
	arg!(-o --out [FILE] "Write the output to a file (- for stdout)")
	.default_value("-")
	.hide_default_value(true),
	arg!(input: [FILE] "Input file. Omit to read from stdin."),
	arg!(-s --syntax [SYNTAX] "Input document syntax. If can't be inferred, defautls to markdown.")
	.possible_values(Syntax::VALUES)
	.ignore_case(true),
	])
}

fn run_(m: &ArgMatches) -> io::Result<()> {
	let (input, syntax) = match m.value_of("input") {
		None => {
			let mut buf = String::with_capacity(2048);
			io::stdin().lock().read_to_string(&mut buf)?;
			(buf, m.value_of_t("syntax").unwrap_or(Syntax::Markdown))
		}
		Some(p) => {
			let input = fs::read_to_string(p)?;
			let syntax = m.value_of_t("syntax").unwrap_or_else(|_| {
				p.rsplit_once('.')
					.and_then(|(_, e)| Syntax::from_ext(e))
					.unwrap_or(Syntax::Markdown)
			});
			(input, syntax)
		}
	};

	let rendered = syntax.render(&input);
	match m.value_of("out").unwrap() {
		"-" => print!("{rendered}"),
		file => {
			fs::write(file, &input)?;
			println!("✓ wrote to {file}");
		}
	}

	Ok(())
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	task::block_in_place(move || run_(m)).map_err(|e| e.into())
}
