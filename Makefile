.PHONY: build test fuzz doc clean

build:
	cd minicbor-tests-nostd && cargo rustc -- -C link-arg=-nostartfiles
	cargo build -p minicbor
	cargo build -p minicbor-io
	cargo build --all --features="derive"
	cargo build --all --features="alloc,derive"
	cargo build --all --features="std,half,derive"
	cargo build --all --all-features

test:
	cargo test -p minicbor
	cargo test -p minicbor-io
	cargo test -p minicbor-tests --features="derive"
	cargo test --all
	cargo test --all --features="derive"
	cargo test --all --features="alloc,derive"
	cargo test --all --features="std,half,derive"
	cargo test --all --all-features

check:
	cargo check -p minicbor-tests-client
	cargo check -p minicbor-tests-client --features="alloc"
	cargo check -p minicbor-tests-client --features="derive"
	cargo check -p minicbor-tests-client --features="std"
	cargo check -p minicbor-tests-client --features="half"
	cargo check -p minicbor-tests-client --features="alloc,derive"
	cargo check -p minicbor-tests-client --features="std,derive"
	cargo check -p minicbor-tests-client --features="alloc,derive,half"
	cargo check -p minicbor-tests-client --features="std,derive,half"

fuzz:
	(cd minicbor-tests && cargo +nightly fuzz run tokenizer)

doc:
	cargo doc --features="std,half,derive"

clean:
	cargo clean
