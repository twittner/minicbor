.PHONY: check build test doc clean bench

check:
	cargo check --all --exclude minicbor-tests-nostd

build:
	cargo rustc -p minicbor-tests-nostd -- -C link-arg=-nostartfiles
	cargo build -p minicbor
	cargo build -p minicbor-io
	cargo build --all --exclude minicbor-tests-nostd --features="std,half,derive"
	cargo build --all --exclude minicbor-tests-nostd --all-features

test:
	cargo test -p minicbor
	cargo test -p minicbor-io
	cargo test --all --exclude minicbor-tests-nostd
	cargo test --all --exclude minicbor-tests-nostd --features="std,half,derive"
	cargo test --all --exclude minicbor-tests-nostd --all-features

bench:
	cargo bench -p minicbor-tests

doc:
	cargo doc -p minicbor --features="std,half,derive"

clean:
	cargo clean
