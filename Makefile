.PHONY: build test doc clean

build:
	cd minicbor-tests-nostd && cargo rustc -- -C link-arg=-nostartfiles
	cargo build -p minicbor
	cargo build -p minicbor-io
	cargo build --all --features="partial-skip-support"
	cargo build --all --features="partial-derive-support"
	cargo build --all --features="alloc,partial-derive-support"
	cargo build --all --features="std,half,derive"
	cargo build --all --all-features

test:
	cargo test -p minicbor
	cargo test -p minicbor-io
	cargo test --all
	cargo test --all --features="partial-skip-support"
	cargo test --all --features="partial-derive-support"
	cargo test --all --features="alloc,partial-derive-support"
	cargo test --all --features="std,half,derive"
	cargo test --all --all-features

doc:
	cargo doc --features="std,half,derive"

clean:
	cargo clean
