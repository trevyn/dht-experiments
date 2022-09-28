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
		let infohash =
			select!(Infohash "WHERE name IS NULL AND attempts is NULL ORDER BY RANDOM() LIMIT 1").unwrap();
		execute!("UPDATE infohash SET attempts = CASE WHEN attempts IS NULL THEN 1 ELSE attempts + 1 END WHERE infohash = " infohash.infohash.unwrap()).unwrap();
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
