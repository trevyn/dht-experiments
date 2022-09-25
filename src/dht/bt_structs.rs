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

#[derive(Debug)]
pub struct Handshake {
	pub info_hash: [u8; 20],
	pub peer_id: [u8; 20],
}

impl Handshake {
	pub fn to_bytes(&self) -> Vec<u8> {
		#[derive(Debug, Encode)]
		struct HandshakeInner {
			magic: &'static str,
			reserved: [u8; 8],
			info_hash: [u8; 20],
			peer_id: [u8; 20],
		}

		let mut out = bincode::encode_to_vec(
			HandshakeInner {
				magic: "BitTorrent protocol",
				reserved: [0, 0, 0, 0, 0, 0x10, 0, 0],
				info_hash: self.info_hash,
				peer_id: self.peer_id,
			},
			CONFIG,
		)
		.unwrap();

		let ext = serde_bencode::to_bytes(&ExtensionHandshake::default()).unwrap();

		let mut len: u32 = ext.len().try_into().unwrap();
		len += 2;

		out.extend_from_slice(&len.to_be_bytes());
		out.push(20);
		out.push(0);
		out.extend_from_slice(&ext);

		out
	}
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ExtensionHandshake {
	pub m: MetadataExtension,
	pub metadata_size: Option<usize>,
}

impl<'a> ExtensionHandshake {
	pub fn from_bytes(buf: &'a [u8]) -> Result<ExtensionHandshake, serde_bencode::Error> {
		serde_bencode::de::from_bytes::<ExtensionHandshake>(buf)
	}
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MetadataExtension {
	pub ut_metadata: Option<u8>,
}

impl Default for MetadataExtension {
	fn default() -> Self {
		Self { ut_metadata: Some(2) }
	}
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct MetadataMessage {
	pub msg_type: usize,
	pub piece: usize,
}

impl MetadataMessage {
	pub fn to_bytes(&self, extension_id: u8) -> Vec<u8> {
		let msg = serde_bencode::to_bytes(self).unwrap();

		let mut out = Vec::with_capacity(msg.len() + 6);

		out.extend_from_slice(&((msg.len() + 2) as u32).to_be_bytes());
		out.push(20);
		out.push(extension_id);
		out.extend_from_slice(&msg);

		out
	}
}
