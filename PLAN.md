A BitTorrent client written in Rust. It also scrapes and indexes the DHT.

It connects to DHT nodes and gets info from them.

It accepts pasted-in magnet links or infohashes, keeps a record of them, and downloads the metainfo and checks peer count, speed, responsiveness.

It is a work in progress, just getting started.

Some things:

- a torrent list
- a torrent list in egui
- local is easier, do the easiest thing to make fastest progress (velocity is key!)
- also dht stuff in egui
- maybe a turbocharger egui template at some point
- Feynman "DISREGARD!" :)

The next step is add egui to dht-experiments.
