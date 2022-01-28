const dates = document.querySelectorAll("time");

for(const date of dates) {
	const stamp = new Date(date.getAttribute("datetime"));
	date.textContent = stamp.toLocaleString();
}
