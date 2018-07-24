TIMESTAMP =  $(shell date '+%Y-%m-%d-%H%M%S')

all: build

run:
	cargo run --release -- -v run --tgt ./contest/model/FA001_tgt.mdl

build:
	cargo build --release

clippy:
	cargo +nightly clippy

test:
	cargo test --release

ci: test
	cargo run --release -- -v ci

gdb:
	cargo build
	rust-gdb --args ~/src/build/rust/icfp2018/debug/icfp2018 -v run --target ./contest/model/FA001_tgt.mdl

doc:
	cargo doc --open

zip-trace:
	mkdir ./contest/submit/$(TIMESTAMP)
	cp -a ./contest/trace/default/*.nbt ./contest/submit/$(TIMESTAMP)
	cp -a ./contest/submit/*.nbt ./contest/submit/$(TIMESTAMP)
	cd ./contest/submit/$(TIMESTAMP) && zip -r ../$(TIMESTAMP)-trace.zip *.nbt
	shasum -a 256 ./contest/submit/$(TIMESTAMP)-trace.zip > ./contest/submit/$(TIMESTAMP)-sha256.txt
	cp ./contest/submit/$(TIMESTAMP)-trace.zip ~/drive/public/2018/
	cp ./contest/submit/$(TIMESTAMP)-sha256.txt  ~/drive/public/2018/

.PHONY: ci zip
