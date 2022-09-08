#[derive(Debug, serde::Serialize)]
struct PingQuery {
	#[serde(with = "serde_bytes_array")]
	id: [u8; 20],
}

impl PingQuery {
	fn into_bytes(self) -> Vec<u8> {
		Query { y: "q", q: "ping", a: self }.to_bytes()
	}
}

#[derive(Debug, serde::Serialize)]
struct Query<T> {
	y: &'static str,
	q: &'static str,
	a: T,
}

impl<T: Serialize> Query<T> {
	fn to_bytes(&self) -> Vec<u8> {
		serde_bencode::to_bytes(&self).unwrap()
	}
}

use rand::prelude::*;
use serde::Serialize;
use std::net::IpAddr;
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let ip = reqwest::get("https://api.ipify.org").await?.text().await?;

	dbg!(&ip);

	let id = from_ip(&IpAddr::V4(ip.parse().unwrap()));

	let enc = PingQuery { id }.into_bytes();

	let send_addr = "dht.transmissionbt.com:6881";
	// let send_addr = "router.utorrent.com:6881";

	let sock = UdpSocket::bind("0.0.0.0:55874").await?;

	let len = sock.send_to(&enc, send_addr).await?;
	println!("{:?} bytes sent", len);

	let mut buf = [0; 1024];
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

fn from_ip(ip: &IpAddr) -> [u8; 20] {
	let mut rng = thread_rng();
	let r: u8 = rng.gen();

	let magic_prefix = magic_prefix_from_ip(ip, r);

	let mut bytes = [0; 20];

	bytes[0] = magic_prefix[0];
	bytes[1] = magic_prefix[1];
	bytes[2] = (magic_prefix[2] & 0xf8) | (rng.gen::<u8>() & 0x7);
	for item in bytes.iter_mut().take(20 - 1).skip(3) {
		*item = rng.gen();
	}
	bytes[20 - 1] = r;

	bytes
}

fn magic_prefix_from_ip(ip: &IpAddr, seed_r: u8) -> [u8; 3] {
	match ip {
		IpAddr::V4(ipv4) => {
			let r32: u32 = seed_r.into();
			let magic: u32 = 0x030f3fff;
			let ip_int: u32 = u32::from_be_bytes(ipv4.octets());
			let nonsense: u32 = (ip_int & magic) | (r32 << 29);
			let crc: u32 = crc::crc32::checksum_castagnoli(&nonsense.to_be_bytes());
			crc.to_be_bytes()[..3].try_into().unwrap()
		}
		IpAddr::V6(_) => unimplemented!(),
	}
}

mod serde_bytes_array {
	use serde::de::Error;
	use serde::{Deserializer, Serializer};

	/// This just specializes [`serde_bytes::serialize`] to `<T = [u8]>`.
	pub(crate) fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serde_bytes::serialize(bytes, serializer)
	}

	/// This takes the result of [`serde_bytes::deserialize`] from `[u8]` to `[u8; N]`.
	pub(crate) fn deserialize<'de, D, const N: usize>(deserializer: D) -> Result<[u8; N], D::Error>
	where
		D: Deserializer<'de>,
	{
		let slice: &[u8] = serde_bytes::deserialize(deserializer)?;
		let array: [u8; N] = slice.try_into().map_err(|_| {
			let expected = format!("[u8; {}]", N);
			D::Error::invalid_length(slice.len(), &expected.as_str())
		})?;
		Ok(array)
	}
}
