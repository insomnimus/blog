use crate::prelude::*;
use super::Post;

pub fn app() -> App<'static> {
	App::new("delete")
	.about("Delete posts.")
	.group(ArgGroup::new("criteria").args(&["id", "last"]).required(true))
	.args(&[
	arg!(id: [ID] "The ID of the post.")
	.validator(validate::<i32>("the value must be a whole number")),
	arg!(--last "Delete the last post instead."),
	arg!(-y --yes "Do not prompt for confirmation."),
	])
}

pub async fn run(m: &ArgMatches) -> Result<()> {
	let yes = m.is_present("yes");
	
	let mut tx = db().begin().await?;
	
	let post = match m.value_of_t::<i32>("id") {
		Ok(id) => {
			
		}
	}
}