check:
    cargo check --all

build:
    cd minicbor-tests-nostd && cargo rustc -- -C link-arg=-nostartfiles
    cargo build -p minicbor
    cargo build -p minicbor-io
    cargo build --all --features="partial-skip-support"
    cargo build --all --features="partial-derive-support"
    cargo build --all --features="alloc,partial-derive-support"
    cargo build --all --features="std,half,derive"
    cargo build --all --all-features

test *ARGS:
    cargo test -p minicbor {{ARGS}}
    cargo test -p minicbor-io {{ARGS}}
    cargo test --all {{ARGS}}
    cargo test --all --features="partial-skip-support" {{ARGS}}
    cargo test --all --features="partial-derive-support" {{ARGS}}
    cargo test --all --features="alloc,partial-derive-support" {{ARGS}}
    cargo test --all --features="std,half,derive" {{ARGS}}
    cargo test --all --all-features {{ARGS}}

bench *ARGS:
    cd minicbor-tests && cargo bench {{ARGS}}

install:
    cargo install --path minicbor --features="std,half"

fuzz:
    cd minicbor-tests && cargo +nightly fuzz run tokenizer

doc:
    cargo doc --all --all-features

clean:
    cargo clean
