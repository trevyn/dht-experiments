#![allow(dead_code)]

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

#[derive(Default, Serialize)]
pub struct ExtensionHandshake {
	m: MetadataExtension,
}

#[derive(Serialize)]
pub struct MetadataExtension {
	pub ut_metadata: u8,
}

impl Default for MetadataExtension {
	fn default() -> Self {
		Self { ut_metadata: 1 }
	}
}
