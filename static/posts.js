const container = document.querySelector("#posts");
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
			load_posts(last_id);
		} catch (e) {
			console.log(e);
		} finally {
			more_button.disabled = false;
		}
	}
});

function load_posts(last_id) {
	const addr = "/api/posts?cursor=" + last_id;
	fetch(addr)
		.then((response) => response.json())
		.then((data) => {
			for (const p of data.posts) {
				container.innerHTML += p;
				container.innerHTML += "\n<br>\n";
			}
		});
}
