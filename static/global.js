localize_dates();
tags_to_links();

function dates_callback(muts, _observer) {
	for (const mut of muts) {
		if (mut.type !== "childList") {
			continue;
		}
		for (const n of mut.nodeList) {
			for (const d of n.querySelectorAll("time")) {
				const dt = n.getAttribute("datetime");
				if (dt) {
					d.textContent = time_since(new Date(dt));
				}
			}
		}
	}
}

const date_observer = new MutationObserver(dates_callback);
date_observer.observe(document.querySelector("main"), {
	attributes: false,
	childList: true,
	subtree: true,
});

function format_since(date) {
	const month_names = [
		"January",
		"February",
		"March",
		"April",
		"May",
		"June",
		"July",
		"August",
		"September",
		"October",
		"November",
		"December",
	];

	const intervals = [
		["second", 1, 60, "just now"],
		["minute", 60, 3600, "a minute ago"],
		["hour", 3600, 86400, "an hour ago"],
		["day", 86400, 1209600, "yesterday"],
		["week", 604800, 3628800, "last week"],
		["month", 2592000, 31536000, "last month"],
	];

	const secs = (new Date() - date) / 1000;

	for (const [name, in_sec, up_to, special_case] of intervals) {
		if (secs < up_to) {
			const n = Math.floor(secs / in_sec);
			if (n == 1) {
				// return `1 ${name} ago`;
				return special_case;
			} else {
				return `${n} ${name}s ago`;
			}
		}
	}

	const n = Math.floor(secs / 31536000);
	if (n == 1) {
		const month = month_names[date.getMonth()];
		return `last year on ${month}`;
	} else {
		return `${n} years ago`;
	}
}

function localize_dates() {
	const dates = document.querySelectorAll("time");

	for (const date of dates) {
		const stamp = new Date(date.getAttribute("datetime"));
		// date.textContent = stamp.toLocaleString();
		date.textContent = format_since(stamp);
	}
}

function tags_to_links() {
	const tags = document.querySelectorAll("ul.tags > li");
	for (const tag of tags) {
		const content = tag.textContent;
		if (tag.childElementCount === 0 && content) {
			const params = new URLSearchParams({ kind: "article", query: content });
			tag.innerHTML = `<a href="/search?${params}"> ${content} </a>`;
		}
	}
}
