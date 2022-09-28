mod dht;

use dht::Infohash;
use futures::StreamExt;
use log::*;
use turbosql::*;

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
		let infohash = select!(Infohash "WHERE name IS NULL ORDER BY RANDOM() LIMIT 1").unwrap();
		dbg!(hex::encode(infohash.infohash.unwrap()));
		let mut s = dht::get_peers(hex::encode(infohash.infohash.unwrap()));

		while let Some(_x) = s.next().await {
			// dbg!(x).ok();
		}

		dbg!("None");
	});

	tokio::time::sleep(std::time::Duration::MAX).await;

	Ok(())
}
