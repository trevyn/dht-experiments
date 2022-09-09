mod dht_structs;
use dht_structs::*;

mod dht_id;
use dht_id::*;

use log::*;
use tokio::net::UdpSocket;
use turbosql::*;

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

pub async fn launch_dht() -> Result<(), Box<dyn std::error::Error>> {
	let ip = reqwest::get("https://api.ipify.org").await?.text().await?;

	info!("external ip is {:?}", ip);

	let id = match select!(Option<SelfId> "WHERE ip = " ip)? {
		Some(SelfId { id, .. }) => id.unwrap(),
		None => {
			let id = id_from_ip(&ip.parse().unwrap());
			SelfId { rowid: None, ip: Some(ip), id: Some(id) }.insert()?;
			id
		}
	};

	let sock = std::sync::Arc::new(UdpSocket::bind("0.0.0.0:55874").await?);
	let sock_clone = sock.clone();

	let mut target = [0u8; 20];
	rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut target);

	tokio::spawn(async move {
		for node in select!(Vec<Node>).unwrap().into_iter() {
			let host = node.host.as_ref().unwrap();

			let _ =
				sock_clone.send_to(&SampleInfohashesQuery { id, target }.into_bytes(), &host).await;

			tokio::time::sleep(std::time::Duration::from_millis(100)).await;
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
		}
	});

	tokio::spawn(async move {
		let mut buf = [0; 1500];
		loop {
			let (len, addr) = match sock.recv_from(&mut buf).await {
				Ok(stuff) => stuff,
				Err(e) => {
					error!("recv_from failed: {:?}", e);
					std::process::exit(1);
				}
			};

			// println!(
			// 	"{:?} bytes received from {:?}: {:?}",
			// 	len,
			// 	addr,
			// 	String::from_utf8_lossy(&buf[..len])
			// );

			tokio::task::spawn_blocking(move || {
				let _ = execute!("UPDATE node SET last_response_ms = " now_ms() " WHERE host = " addr.to_string());
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
				Infohash { infohash: Some(infohash.try_into().unwrap()), ..Default::default() }
					.insert()?;
			}
		}
	}

	if let ResponseArgs { nodes: Some(Bytes::Bytes(nodes)), .. } = response {
		for chunk in nodes.chunks_exact(26) {
			let a: Result<dht_structs::CompactInfo, _> = bincode::deserialize(chunk);

			if let Ok(a) = a {
				let host = format!("{}:{}", a.ip_string(), a.port());
				// dbg!(&host);
				if select!(Option<Node> "WHERE host = " host)?.is_none() {
					Node { host: Some(host), id: Some(a.id), ..Default::default() }.insert()?;
				} else {
					execute!("UPDATE node SET id = " a.id " WHERE host = " host)?;
				}
			}
		}
	}
	Ok(())
}
