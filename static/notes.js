const container = document.querySelector("#notes");
const more_button = document.querySelector("#loadmore");
more_button.addEventListener("click", function () {
	more_button.disabled = true;
	const posts = container.querySelectorAll("article");
	if (posts) {
		const last_id = posts[posts.length - 1].id?.slice(1) ?? 1;
		if (last_id <= 1) {
			return;
		}
		try {
			const was_last = load_posts(last_id);
			more_button.disabled = was_last;
		} catch (e) {
			console.log(e);
		}
	}
});

function load_posts(last_id) {
	const addr = "/api/notes?cursor=" + last_id;
	return fetch(addr)
		.then((response) => response.json())
		.then((data) => {
			for (const p of data.notes) {
				container.innerHTML += p;
				container.innerHTML += "\n<br>\n";
			}
			return data.notes.length === 0;
		});
}
