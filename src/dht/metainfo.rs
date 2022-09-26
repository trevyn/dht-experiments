use serde::{Deserialize, Serialize};
use std::ops::Rem;
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use std::sync::Arc;
use std::usize::MAX;
use tokio::sync::Mutex;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct MetadataMessage {
	pub msg_type: usize,
	pub piece: usize,
	pub total_size: Option<usize>,
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

impl<'a> MetadataMessage {
	pub fn from_bytes(buf: &'a [u8]) -> Result<Self, serde_bencode::Error> {
		serde_bencode::de::from_bytes::<Self>(buf)
	}
}

struct MetaInfoInner {
	size: usize,
	data: Vec<u8>,
	pieces: Vec<usize>,
}

fn num_pieces_for_size(size: usize) -> usize {
	(size + 16383) / 16384
}

impl MetaInfoInner {
	fn new(size: usize) -> Self {
		Self { size, data: vec![0; size], pieces: vec![0; num_pieces_for_size(size)] }
	}
	fn num_pieces(&self) -> usize {
		num_pieces_for_size(self.size)
	}
}

#[derive(Clone)]
pub struct MetaInfo {
	infohash: [u8; 20],
	inner: Arc<Mutex<Option<MetaInfoInner>>>,
}

impl MetaInfo {
	pub fn new(infohash: [u8; 20]) -> Self {
		Self { infohash, inner: Default::default() }
	}

	pub fn infohash(&self) -> [u8; 20] {
		self.infohash
	}

	pub async fn got_size(&self, size: usize) {
		let mut inner = self.inner.lock().await;
		if let Some(inner) = inner.as_ref() {
			assert_eq!(inner.size, size);
		} else {
			*inner = Some(MetaInfoInner::new(size));
		}
	}

	pub async fn which_piece(&self) -> Option<usize> {
		let mut guard = self.inner.lock().await;
		let inner = guard.as_mut().unwrap();
		let min = *inner.pieces.iter().min().unwrap();
		if min == MAX {
			return None;
		}
		let piece = inner.pieces.iter().position(|v| *v == min).unwrap();
		inner.pieces[piece] += 1;
		Some(piece)
	}

	pub async fn got_metadata_message(&self, data: &[u8]) {
		let msg = MetadataMessage::from_bytes(data).unwrap();
		let mut guard = self.inner.lock().await;
		let inner = guard.as_mut().unwrap();
		assert_eq!(msg.total_size, Some(inner.size));
		let start = msg.piece * 16384;
		let end = std::cmp::min(start + 16384, inner.size);
		inner.data[start..end].clone_from_slice(&data[(data.len() - (end - start))..data.len()]);
		dbg!(self.verify(&inner.data));
	}

	async fn got_piece(&self, i: usize, piece: &[u8]) {
		let mut guard = self.inner.lock().await;
		let inner = guard.as_mut().unwrap();
		let offset = i * 16384;
		inner.data[offset..offset + piece.len()].clone_from_slice(piece);
		inner.pieces[i] = MAX;
		dbg!(self.verify(&inner.data));
	}

	fn verify(&self, data: &[u8]) -> bool {
		use sha1::{Digest, Sha1};
		let mut hasher = Sha1::new();
		hasher.update(data);
		let result: [u8; 20] = hasher.finalize().into();
		result == self.infohash
	}

	fn subscribe() {}
}
