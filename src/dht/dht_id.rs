use rand::prelude::*;
use std::net::Ipv4Addr;

pub fn id_from_ip(ip: &Ipv4Addr) -> [u8; 20] {
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

fn magic_prefix_from_ip(ip: &Ipv4Addr, seed_r: u8) -> [u8; 3] {
	let r32: u32 = seed_r.into();
	let magic: u32 = 0x030f3fff;
	let ip_int: u32 = u32::from_be_bytes(ip.octets());
	let nonsense: u32 = (ip_int & magic) | (r32 << 29);
	let crc: u32 = crc::Crc::<u32>::new(&crc::CRC_32_ISCSI).checksum(&nonsense.to_be_bytes());
	crc.to_be_bytes()[..3].try_into().unwrap()
}
