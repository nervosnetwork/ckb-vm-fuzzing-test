#!/bin/bash

BRANCH="${1:-develop}"

mkdir -p deps/ckb-vm-${BRANCH}/fuzz/corpus/asm
mkdir -p deps/ckb-vm-${BRANCH}/fuzz/corpus/isa_a
mkdir -p deps/ckb-vm-${BRANCH}/fuzz/corpus/isa_b

cp corpus/asm/* deps/ckb-vm-${BRANCH}/fuzz/corpus/asm/
cp corpus/isa_a/* deps/ckb-vm-${BRANCH}/fuzz/corpus/isa_a/
cp corpus/isa_b/* deps/ckb-vm-${BRANCH}/fuzz/corpus/isa_b/

echo "nightly" >> deps/ckb-vm-${BRANCH}/rust-toolchain
cd deps/ckb-vm-${BRANCH}

cargo fuzz run -j 4 asm
cargo fuzz run -j 4 isa_a
cargo fuzz run -j 4 isa_b

cd -
