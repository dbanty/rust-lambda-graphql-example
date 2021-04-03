build:
	cargo build --release --target x86_64-unknown-linux-musl
	rm -rf bootstrap
	mkdir "bootstrap"
	cp ./target/x86_64-unknown-linux-musl/release/graphql-example bootstrap/bootstrap

deploy: build
	npm run deploy

local: build
	npm run template
	sam local start-api
