use tokio::net::UdpSocket;

mod dht_structs;
use dht_structs::*;

mod dht_id;
use dht_id::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let ip = reqwest::get("https://api.ipify.org").await?.text().await?;

	dbg!(&ip);

	let id = id_from_ip(&ip.parse().unwrap());

	let enc = PingQuery { id }.into_bytes();

	let send_addr = "dht.transmissionbt.com:6881";
	// let send_addr = "router.utorrent.com:6881";

	let sock = UdpSocket::bind("0.0.0.0:55874").await?;

	let len = sock.send_to(&enc, send_addr).await?;
	println!("{:?} bytes sent", len);

	let mut buf = [0; 1500];
	loop {
		let (len, addr) = sock.recv_from(&mut buf).await?;
		println!(
			"{:?} bytes received from {:?}: {:?}",
			len,
			addr,
			String::from_utf8_lossy(&buf[..len])
		);
	}
}
