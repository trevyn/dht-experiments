mod dht;

use futures::StreamExt;
use log::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	if std::env::var_os("RUST_LOG").is_none() {
		std::env::set_var("RUST_LOG", "info")
	}

	pretty_env_logger::init_timed();

	info!("start");

	dht::launch_dht().await?;

	info!("dht launched");

	tokio::spawn(async move {
		let mut s = dht::get_peers("dd8255ecdc7ca55fb0bbf81323d87062db1f6d1c");

		while let Some(_x) = s.next().await {
			// dbg!(x).ok();
		}

		dbg!("None");
	});

	tokio::time::sleep(std::time::Duration::MAX).await;

	Ok(())
}
