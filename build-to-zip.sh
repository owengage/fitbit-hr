#!/bin/bash
set -ex
cargo build --bin lambda --release --target x86_64-unknown-linux-musl
cp target/x86_64-unknown-linux-musl/release/lambda bootstrap
zip lambda.zip bootstrap
rm bootstrap
