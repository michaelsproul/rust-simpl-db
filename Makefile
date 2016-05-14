all: gendata
	cargo build --release

gendata:
	make -C reference gendata

debug:
	cargo build

test:
	cargo test --lib

clean:
	rm -rf *.data *.info *.ovflow
	cargo clean

.PHONY: gendata
