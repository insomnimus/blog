const dates = document.querySelectorAll("time");

for (const date of dates) {
	const stamp = new Date(date.getAttribute("datetime"));
	// date.textContent = stamp.toLocaleString();
	date.textContent = format_since(stamp);
}

function format_since(date) {
	const secs = (new Date() - date) / 1000;
	const intervals = [
		["second", 1, 60],
		["minute", 60, 3600],
		["hour", 3600, 86400],
		["day", 86400, 1209600],
		["week", 604800, 3628800],
		["month", 2592000, 46656000],
	];

	for (const [name, in_sec, up_to] of intervals) {
		if (secs < up_to) {
			const n = Math.floor(secs / in_sec);
			if (n == 1) {
				return `${n} ${name} ago`;
			} else {
				return `${n} ${name}s ago`;
			}
		}
	}

	const n = Math.floor(secs / 31536000);
	if (n == 1) {
		return `${n} year ago`;
	} else {
		return `${n} years ago`;
	}
}
