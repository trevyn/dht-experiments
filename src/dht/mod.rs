#![allow(unused_macros)]

mod dht_structs;
use dht_structs::*;

mod dht_id;
use dht_id::*;

use log::*;
use tokio::{
	io::{AsyncReadExt, AsyncWriteExt},
	net::UdpSocket,
};
use turbosql::*;

use once_cell::sync::{Lazy, OnceCell};

#[derive(Debug)]
pub enum Progress<T> {
	Progress { status: String },
	Complete { result: T, status: String },
}

type ProgressStream<T> =
	std::pin::Pin<Box<dyn futures::Stream<Item = Result<Progress<T>, tracked::StringError>> + Send>>;

macro_rules! progress {
	($($args:tt),*) => {{
		Progress::Progress{status:format!($($args),*)}
	}};
}

macro_rules! complete {
	(result:$ret:expr, status:$($args:tt),*) => {{
		Progress::Complete{result:$ret.into(), status:format!($($args),*)}
	}};
	(status:$arg:expr, result:$ret:expr) => {{
		Progress::Complete{result:$ret.into(), status:format!($arg)}
	}};
	({result:$ret:expr, status:$($args:tt),*}) => {{
		Progress::Complete{result:$ret.into(), status:format!($($args),*)}
	}};
	({status:$arg:expr, result:$ret:expr}) => {{
		Progress::Complete{result:$ret.into(), status:format!($arg)}
	}};
	($ret:expr, $($args:tt),*) => {{
		Progress::Complete{result:$ret.into(), status:format!($($args),*)}
	}};
}

macro_rules! err {
	($($args:tt),*) => {{
		::core::result::Result::Err(format!($($args),*))
	}};
}

#[derive(Turbosql, Default)]
struct SelfId {
	rowid: Option<i64>,
	ip: Option<String>,
	id: Option<[u8; 20]>,
}

#[derive(Turbosql, Default)]
struct Node {
	rowid: Option<i64>,
	host: Option<String>,
	id: Option<[u8; 20]>,
	// last_ping_attempt_ms: Option<i64>,
	last_response_ms: Option<i64>,
}

#[derive(Turbosql, Default)]
struct Infohash {
	rowid: Option<i64>,
	infohash: Option<[u8; 20]>,
}

static BROADCAST: Lazy<tokio::sync::broadcast::Sender<ResponseArgs>> =
	Lazy::new(|| tokio::sync::broadcast::channel(200).0);

static SOCK: OnceCell<tokio::net::UdpSocket> = OnceCell::new();

static SELF_ID: OnceCell<[u8; 20]> = OnceCell::new();

#[tracked::tracked]
pub async fn launch_dht() -> Result<(), tracked::StringError> {
	let ip = reqwest::get("https://api.ipify.org").await?.text().await?;

	info!("external ip is {:?}", ip);

	SELF_ID
		.set(match select!(Option<SelfId> "WHERE ip = " ip)? {
			Some(SelfId { id, .. }) => id.unwrap(),
			None => {
				let id = id_from_ip(&ip.parse().unwrap());
				SelfId { rowid: None, ip: Some(ip), id: Some(id) }.insert()?;
				id
			}
		})
		.map_err(|_| "SELF_ID already set")?;

	SOCK.set(UdpSocket::bind("0.0.0.0:55874").await?).map_err(|_| "SOCK already set")?;

	// let sock = std::sync::Arc::new();
	// let sock_clone = sock.clone();

	// let mut target = [0u8; 20];
	// rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut target);

	// tokio::spawn(async move {
	// 	for node in select!(Vec<Node>).unwrap().into_iter() {
	// 		let host = node.host.as_ref().unwrap();

	// 		SOCK.get()
	// 			.unwrap()
	// 			.send_to(
	// 				&SampleInfohashesQuery { id: *SELF_ID.get().unwrap(), target }.into_bytes(),
	// 				&host,
	// 			)
	// 			.await;

	// 		tokio::time::sleep(std::time::Duration::from_millis(100)).await;
	// println!("{:?} bytes sent to {:?}", len, host);

	// let hostip = host.split(':').into_iter().next().unwrap();
	// let host = format!("{}:6881", hostip);
	// let len = sock.send_to(&PingQuery { id }.into_bytes(), &host).await?;
	// println!("{:?} bytes sent to {:?}", len, host);

	// let host = format!("{}:6882", hostip);
	// let len = sock.send_to(&PingQuery { id }.into_bytes(), &host).await?;
	// println!("{:?} bytes sent to {:?}", len, host);

	// let host = format!("{}:6883", hostip);
	// let len = sock.send_to(&PingQuery { id }.into_bytes(), &host).await?;
	// println!("{:?} bytes sent to {:?}", len, host);

	// let len = sock.send_to(&FindNodeQuery { id, target }.into_bytes(), &host).await?;
	// println!("{:?} bytes sent to {:?}", len, host);
	// 	}
	// });

	tokio::spawn(async move {
		let mut buf = [0; 1500];
		loop {
			let (len, addr) = SOCK.get().unwrap().recv_from(&mut buf).await.unwrap_or_else(|e| {
				error!("recv_from failed: {:?}", e);
				std::process::exit(1);
			});

			// println!(
			// 	"{:?} bytes received from {:?}: {:?}",
			// 	len,
			// 	addr,
			// 	String::from_utf8_lossy(&buf[..len])
			// );

			tokio::task::spawn_blocking(move || {
				execute!(
					"UPDATE node"
					"SET last_response_ms = " now_ms()
					"WHERE host = " addr.to_string()
				)
				.unwrap();
			});

			if let Ok(response) = Response::from_bytes(&buf[..len]) {
				tokio::task::spawn_blocking(move || {
					if let Err(e) = process_response(addr.to_string(), response) {
						warn!("process_response error: {:?}", e);
					}
				});
			}
		}
	});

	Ok(())
}

fn process_response(
	addr: String,
	response: ResponseArgs,
) -> Result<(), Box<dyn std::error::Error>> {
	if let ResponseArgs { num, interval, samples: Some(Bytes::Bytes(ref samples)), .. } = response {
		println!(
			"got {} bytes of samples, total {:?}, interval {:?} from {:?}",
			samples.len(),
			num,
			interval,
			addr.to_string()
		);

		for infohash in samples.chunks_exact(20) {
			if select!(Option<Infohash> "WHERE infohash = " infohash)?.is_none() {
				Infohash { infohash: Some(infohash.try_into().unwrap()), ..Default::default() }.insert()?;
			}
		}
	}

	for node in response.nodes() {
		let host = node.host();
		execute!(
			"INSERT INTO node(host, id)"
			"VALUES (" host, node.id ")"
			"ON CONFLICT(host) DO UPDATE SET id = " node.id
		)?;
	}

	BROADCAST.send(response)?;

	Ok(())
}

#[tracked::tracked]
pub fn get_peers(infohash: impl Into<String>) -> ProgressStream<String> {
	let infohash = infohash.into();
	Box::pin(async_stream::try_stream! {
		let mut packets_sent = 0;
		let mut packets_recv = 0;
		let mut our_ids = std::collections::HashSet::new();
		let mut peers = std::collections::HashSet::new();

		let info_hash: [u8; 20] =
			hex::decode(&infohash)?.try_into().map_err(|_| "infohash not 20 hex bytes")?;

		yield progress!("loading for infohash {infohash}");

		// err!("ohno")?;
		// 	// yield complete!({
		// 	// 	result: "datagoeshere",
		// 	// 	status: "loading complete for infohash {infohash}",
		// 	// });

		let mut receiver = BROADCAST.subscribe();

		for node in
			select!(Vec<Node> "WHERE rowid IN (SELECT rowid FROM node ORDER BY RANDOM() LIMIT 20)")?
				.into_iter()
		{
			// let host = ;
			our_ids.insert(node.id.unwrap());

			SOCK
				.get()
				.unwrap()
				.send_to(
					&GetPeersQuery { id: *SELF_ID.get().unwrap(), info_hash }.into_bytes(),
					&node.host.as_ref().unwrap(),
				)
				.await?;

			packets_sent += 1;
		}

		loop {
			yield progress!(
				"loading dht for infohash {infohash}; sent {packets_sent}, recv {packets_recv}, peers {}",
				(peers.len())
			);

			let response = receiver.recv().await.unwrap();

			packets_recv += 1;

			if our_ids.contains(&response.id()) {
				for node in response.nodes() {
					if our_ids.insert(node.id) {
						SOCK
							.get()
							.unwrap()
							.send_to(
								&GetPeersQuery { id: *SELF_ID.get().unwrap(), info_hash }.into_bytes(),
								&node.host(),
							)
							.await
							.map_err(|e| warn!("{:?} {:?}", e, node.host()))
							.ok();

						packets_sent += 1;
					}
				}

				if let Some(values) = response.values {
					for peer in values {
						if peers.insert(peer.clone()) {
							// println!("{}", peer.host());
							tokio::spawn(async move {
								if let Ok(mut stream) = tokio::net::TcpStream::connect(peer.host()).await {
									info!("connected {}", peer.host());

									let n = stream
										.write(&Handshake { info_hash, peer_id: *SELF_ID.get().unwrap() }.to_bytes())
										.await
										.unwrap();
									info!("write {} bytes to {}", n, peer.host());

									let mut buffer = [0; 1024];
									let n = stream.read(&mut buffer[..]).await.unwrap();
									info!("read {} bytes from {}", n, peer.host());
								}
							});
						}
					}
				}
			}
		}

		// yield complete! {
		// 	status: "loading complete for infohash",
		// 	result: "datagoeshere"
		// };
		// 	// yield complete!("done", "loading for infohash {infohash}");
	})
}
