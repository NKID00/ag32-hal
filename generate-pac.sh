#!/bin/bash

set -euxo pipefail

TMP=`mktemp -d`
svdtools patch --post-validate svd/patch.yaml "$TMP/patched.svd"
svd2rust -s --edition 2024 --target riscv --settings settings.yaml -i "$TMP/patched.svd" -o pac
cp device.x pac/
rm -rf pac/src
form -i pac/lib.rs -o pac/src && rm pac/lib.rs
cargo fmt -p ag32-pac
