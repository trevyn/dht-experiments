#[path = "serde_bytes_array.rs"]
mod serde_bytes_array;

use bincode::{Decode, Encode};
use log::*;
use serde::{Deserialize, Serialize};

static CONFIG: bincode::config::Configuration<
	bincode::config::LittleEndian,
	bincode::config::Varint,
	bincode::config::SkipFixedArrayLength,
> = bincode::config::standard().skip_fixed_array_length();

#[derive(Debug, Serialize)]
pub struct PingQuery {
	#[serde(with = "serde_bytes_array")]
	pub id: [u8; 20],
}

impl PingQuery {
	pub fn into_bytes(self) -> Vec<u8> {
		Query { t: "0".into(), v: "XX01", y: "q", q: "ping", a: self }.to_bytes()
	}
}

#[derive(Debug, Serialize)]
pub struct GetPeersQuery {
	#[serde(with = "serde_bytes_array")]
	pub id: [u8; 20],
	#[serde(with = "serde_bytes_array")]
	pub info_hash: [u8; 20],
}

impl GetPeersQuery {
	pub fn into_bytes(self) -> Vec<u8> {
		Query { t: self.info_hash.into(), v: "XX01", y: "q", q: "get_peers", a: self }.to_bytes()
	}
}

#[derive(Debug, Serialize)]
pub struct FindNodeQuery {
	#[serde(with = "serde_bytes_array")]
	pub id: [u8; 20],
	#[serde(with = "serde_bytes_array")]
	pub target: [u8; 20],
}

impl FindNodeQuery {
	pub fn into_bytes(self) -> Vec<u8> {
		Query { t: "0".into(), v: "XX01", y: "q", q: "find_node", a: self }.to_bytes()
	}
}

#[derive(Debug, Serialize)]
pub struct SampleInfohashesQuery {
	#[serde(with = "serde_bytes_array")]
	pub id: [u8; 20],
	#[serde(with = "serde_bytes_array")]
	pub target: [u8; 20],
}

impl SampleInfohashesQuery {
	pub fn into_bytes(self) -> Vec<u8> {
		Query { t: "0".into(), v: "XX01", y: "q", q: "sample_infohashes", a: self }.to_bytes()
	}
}

#[derive(Debug, Decode)]
pub struct CompactInfo {
	pub id: [u8; 20],
	pub ip: [u8; 4],
	port: [u8; 2],
}

impl CompactInfo {
	pub fn host(&self) -> String {
		format!("{}:{}", self.ip_string(), self.port())
	}
	pub fn ip_string(&self) -> String {
		format!("{}.{}.{}.{}", self.ip[0], self.ip[1], self.ip[2], self.ip[3])
	}
	pub fn port(&self) -> u16 {
		u16::from_be_bytes(self.port)
	}
}

#[derive(Debug, Serialize)]
struct Query<T> {
	#[serde(with = "serde_bytes")]
	t: Vec<u8>,
	v: &'static str,
	y: &'static str,
	q: &'static str,
	a: T,
}

impl<T: Serialize> Query<T> {
	fn to_bytes(&self) -> Vec<u8> {
		serde_bencode::to_bytes(&self).unwrap()
	}
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Hash)]
#[serde(untagged)]
pub enum Peer {
	#[serde(with = "serde_bytes")]
	Peer(Vec<u8>),
}

impl Peer {
	pub fn host(&self) -> String {
		format!("{}:{}", self.ip_string(), self.port())
	}
	pub fn ip_string(&self) -> String {
		let Peer::Peer(peer) = self;
		format!("{}.{}.{}.{}", peer[0], peer[1], peer[2], peer[3])
	}
	pub fn port(&self) -> u16 {
		let Peer::Peer(peer) = self;
		u16::from_be_bytes(peer[4..6].try_into().unwrap())
	}
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum Bytes {
	#[serde(with = "serde_bytes")]
	Bytes(Vec<u8>),
}

impl Default for &Bytes {
	fn default() -> Self {
		static EMPTY: Bytes = Bytes::Bytes(Vec::new());
		&EMPTY
	}
}

#[derive(Clone, Debug, Deserialize)]
pub struct ResponseArgs {
	#[serde(with = "serde_bytes")]
	pub id: Vec<u8>,
	pub token: Option<Bytes>,
	pub nodes: Option<Bytes>,
	pub values: Option<Vec<Peer>>,
	pub samples: Option<Bytes>,
	pub interval: Option<i64>,
	pub num: Option<i64>,
}

impl ResponseArgs {
	pub fn id(&self) -> [u8; 20] {
		let mut a = [0; 20];
		a.copy_from_slice(&self.id);
		a
	}
	pub fn nodes(&self) -> Vec<CompactInfo> {
		let Bytes::Bytes(bytes) = self.nodes.as_ref().unwrap_or_default();
		bytes.chunks_exact(26).map(|c| bincode::decode_from_slice(c, CONFIG).unwrap().0).collect()
	}
}

#[derive(Debug, Deserialize)]
pub struct Response {
	#[serde(with = "serde_bytes")]
	pub t: Vec<u8>,
	// y: &'static str,
	// q: &'static str,
	r: ResponseArgs,
}

impl<'a> Response {
	pub fn from_bytes(buf: &'a [u8]) -> Result<ResponseArgs, serde_bencode::Error> {
		let x = serde_bencode::de::from_bytes::<Response>(buf);
		let x = x?;
		Ok(x.r)
	}
}
