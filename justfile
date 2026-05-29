trace:
	RUST_LOG=slim_osc=trace cargo run

[default]
run:
	cargo run

build:
	cargo build --release

profile:
	CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --release
