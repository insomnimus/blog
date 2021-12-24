async fn start_watching(home: Arc<RwLock<Home>) -> Result<()> {
	let (tx, rx) = mpsc::channel();
		let mut watcher = notify::recommended_watcher(move |res| match res {
			Ok(ev) => {
				info!("received notify event: {:?}", &ev);
				tx.send_blocking(ev).unwrap();
			}
			Err(e) => error!("notify error: {}", e),
		})?;
		
		task::spawn_blocking({
			let posts_dir = home.posts_dir.clone();
			move || {
			if let Err(e) = watcher.watch(&posts_dir, RecursiveMode::NonRecursive) {
				error!("watcher failed to initialize: {}", e);
			}
		}
		});
		
		task::spawn(async move {
			loop {
				let ev = match rx.recv().await {
					Ok(ev) => ev,
					Err(e) => {
						error!("watcher thread is poisoned: {}", e);
						return;
					}
				};
				if matches!(ev.kind, EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)) {
							let mut home = home.write().await;
							home.reload().await;
						}
			}
		});
		Ok(())
}