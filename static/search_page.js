function submit_search(event) {
	if (event.keyCode !== 13) {
		return;
	}
	const query = document.querySelector("#search-query").value.trim();
	if (query) {
		const kind = document.querySelector('input[name="searchtype"]:checked').value;
		const params = new URLSearchParams({
			kind: kind,
			query: query,
		});
		const new_path = "/search?" + params;
		let port = "";
		if (location.port) {
			port = ":" + location.port;
		}
		window.location = location.protocol + "//" + location.hostname + port + new_path;
	}
}
