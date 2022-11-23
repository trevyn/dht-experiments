mod dht;

use clap::Parser;
use dht::{Infohash, Node};
use futures::StreamExt;
use log::*;
use std::process::exit;
use std::sync::Mutex;
use turbosql::*;

#[derive(Parser, Debug)]
struct Args {
	/// Sample infohashes from DHT
	#[arg(long, default_value_t = false)]
	sample: bool,

	/// Harvest metainfo files
	#[arg(long, default_value_t = false)]
	harvest: bool,

	/// Interface to bind to for network connections
	#[arg(short, long)]
	interface: Option<String>,

	/// Port to use for DHT
	#[arg(short, long, default_value_t = 55874)]
	port: u16,
}

static STATUS: Mutex<String> = Mutex::new(String::new());

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	if std::env::var_os("RUST_LOG").is_none() {
		std::env::set_var("RUST_LOG", "info")
	}

	tracing_subscriber::fmt::init();

	let args = Args::parse();

	info!("start");

	dht::launch_dht(args.interface, args.port).await?;

	info!("dht launched");

	tokio::spawn(async move {
		loop {
			let infohash =
				select!(Infohash "WHERE name IS NULL ORDER BY attempts, RANDOM() LIMIT 1").unwrap();
			execute!("UPDATE infohash SET attempts = CASE WHEN attempts IS NULL THEN 1 ELSE attempts + 1 END WHERE infohash = " infohash.infohash.unwrap()).unwrap();
			dbg!(hex::encode(infohash.infohash.unwrap()));
			let mut s = dht::get_peers(hex::encode(infohash.infohash.unwrap()));

			while let Some(status) = s.next().await {
				if let Ok(dht::Progress::Progress { status }) = status {
					*STATUS.lock().unwrap() = status;
				}
			}

			info!("complete");

			if !args.harvest {
				exit(0);
			}
		}
	});

	let native_options = eframe::NativeOptions::default();
	eframe::run_native("dht-experiments", native_options, Box::new(|cc| Box::new(MyEguiApp::new(cc))));

	info!("sleeping!");

	tokio::time::sleep(std::time::Duration::MAX).await;

	Ok(())
}

use eframe::egui;

#[derive(Default)]
struct MyEguiApp {}

impl MyEguiApp {
	fn new(cc: &eframe::CreationContext<'_>) -> Self {
		// Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
		// Restore app state using cc.storage (requires the "persistence" feature).
		// Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
		// for e.g. egui::PaintCallback.
		Self::default()
	}
}

impl eframe::App for MyEguiApp {
	fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
		egui::CentralPanel::default().show(ctx, |ui| {
			ui.heading("Hello World!");
			ui.heading(STATUS.lock().unwrap().as_str());
		});
		ctx.request_repaint();
	}
}
