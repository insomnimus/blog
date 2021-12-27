var dates = document.querySelectorAll("time");
// alert("im here bitch");
// alert("have " + dates.length + " dates");
	for(i = 0; i< dates.length; i++) {
		const datetime = dates[i].getAttribute("datetime");
		if(datetime) {
			dates[i].innerHTML = luxon.DateTime.fromISO(datetime).toLocaleString();
			// dates[i].innerHTML = "nah";
			// alert("asdf");
		}
	}

