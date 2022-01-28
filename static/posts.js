const container = document.querySelector("#posts");

addEventListener("scroll", () => {
	const {scrollHeight, scrollTop, clientHeight} = document.documentElement;
	if(scrollTop + clientHeight > scrollHeight - 5) {
		setTimeout(fetch_more, 5000);
	}
});

function fetch_more() {
	const posts = container.querySelector("article");
	if(posts.length == 0) {
		return;
	}
	const last_id = posts[posts.length - 1].id.substring(1);
	if(last_id === "p1") {
		return;
	}
	const addr = "/api/posts?cursor=" + last_id;
	fetch(addr)
	.then(response => response.json())
	.then(data => {
		for(const p of data.posts) {
			container.innerHTML += "\n<hr>\n" + p;
		}
	});
}
