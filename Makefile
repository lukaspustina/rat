.PHONY: all test clippy install_clippy kcov docker_run

KCOV_VERSION=v33

all: clippy tests

test:
	cargo test

clippy:
	RUST_BACKTRACE=1 rustup run nightly cargo clippy

install_clippy:
	RUST_BACKTRACE=1 rustup run nightly cargo install clippy

kcov:
	docker run --security-opt seccomp=unconfined -v $$(pwd):/source ragnaroek/kcov:$(KCOV_VERSION) --exclude-pattern=/.cargo,/usr/lib --verify /source/target/cov /source/target/debug/rat

docker_run:
	docker run -v $$(pwd):/volume -w /volume -t clux/muslrust:nightly sh -c "cargo run"

