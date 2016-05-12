all:
	cargo build --release

debug:
	cargo build

test:
	cargo test --lib

clean:
	rm -rf *.data *.info *.ovflow
	cargo clean
