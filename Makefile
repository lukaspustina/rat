.PHONY: all test unit ignored clippy install_clippy kcov docker_run

KCOV_VERSION=v33

all: clippy tests

test:
	cargo test

unit:
	cargo test --lib

ignored:
	cargo test -- --ignored

clippy:
	rustup run nightly cargo clippy

install_clippy:
	rustup run nightly cargo install clippy

kcov:
	docker run --security-opt seccomp=unconfined -v $$(pwd):/source ragnaroek/kcov:$(KCOV_VERSION) --exclude-pattern=/.cargo,/usr/lib --verify /source/target/cov /source/target/debug/rat

docker_run:
	docker run -v $$(pwd):/volume -w /volume -t clux/muslrust:nightly sh -c "cargo run"

