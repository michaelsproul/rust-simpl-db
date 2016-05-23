all: gendata
	cargo build --release

gendata:
	make -C reference gendata

debug:
	cargo build

test:
	cargo test --lib

clean:
	rm -rf *.data *.info *.ovflow dist
	cargo clean

dist: clean
	@mkdir -p dist
	@tar cf dist/ass2.tar \
		src tests Makefile \
		Cargo.toml \
		Cargo.lock \
		.gitignore \
		create delete gendata insert select test.sh stats\
		README.md > /dev/null
	@echo
	@echo "   ##############################################"
	@echo "   ##          tar'd to ./dist/ass2.tar        ##"
	@echo "   ##############################################"
	@echo

.PHONY: gendata dist
