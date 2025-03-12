#!/bin/bash

set -e

cargo build -r
ld.lld -s --nmagic target/release/libbindmount.a -o bindmount
llvm-strip --strip-sections bindmount
