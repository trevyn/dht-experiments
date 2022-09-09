mod dht;

use log::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	if std::env::var_os("RUST_LOG").is_none() {
		std::env::set_var("RUST_LOG", "info")
	}

	pretty_env_logger::init_timed();

	dht::launch_dht().await?;

	info!("dht launched");

	tokio::time::sleep(std::time::Duration::from_secs(99999)).await;

	Ok(())

	// let send_addr = "dht.transmissionbt.com:6881";

	// let info_hash =
	// 	hex::decode("").unwrap().try_into().unwrap();

	// let info_hash = rand::thread_rng().gen::<[u8; 20]>();

	// let nodes = select!(Vec<Node>)?;
}
