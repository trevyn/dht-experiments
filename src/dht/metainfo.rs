use std::sync::Arc;
use std::usize::MAX;
use tokio::sync::Mutex;

struct MetaInfoInner {
	size: usize,
	data: Vec<u8>,
	pieces: Vec<usize>,
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
			if inner.size != size {
				panic!("metadata_size doesn't match");
			}
		} else {
			let numpieces = (size + 16383) / 16384;
			*inner = Some(MetaInfoInner { size, data: vec![0; size], pieces: vec![0; numpieces] });
		}
	}

	pub async fn which_piece(&self) -> Option<usize> {
		let mut inner = self.inner.lock().await;
		let min = *inner.as_ref().unwrap().pieces.iter().min().unwrap();
		if min == MAX {
			return None;
		}
		let piece = inner.as_ref().unwrap().pieces.iter().position(|v| *v == min).unwrap();
		inner.as_mut().unwrap().pieces[piece] += 1;
		Some(piece)
	}

	pub async fn got_chunk(&self, i: usize, chunk: [u8; 16384]) {
		let mut inner = self.inner.lock().await;
		let offset = i * 16384;
		inner.as_mut().unwrap().data[offset..offset + 16384].clone_from_slice(&chunk);
		inner.as_mut().unwrap().pieces[i] = MAX;
	}

	fn verify() {}

	fn subscribe() {}
}
