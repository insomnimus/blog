let dates = document.querySelectorAll("time");

for(i = 0; i< dates.length; i++) {
	const datetime = dates[i].getAttribute("datetime");
	if(datetime) {
		dates[i].innerHTML = luxon.DateTime.fromISO(datetime).toLocaleString();
		// dates[i].innerHTML = "nah";
		// alert("asdf");
	}
}

