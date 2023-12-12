#!/bin/bash

BRANCH="${1:-develop}"

cd deps
rm -rf ckb-vm
git clone https://github.com/nervosnetwork/ckb-vm --depth 1 --branch ${BRANCH}
cd -

mkdir -p deps/ckb-vm/fuzz/corpus/asm
mkdir -p deps/ckb-vm/fuzz/corpus/isa_a
mkdir -p deps/ckb-vm/fuzz/corpus/isa_b

cp corpus/asm/* deps/ckb-vm/fuzz/corpus/asm/
cp corpus/isa_a/* deps/ckb-vm/fuzz/corpus/isa_a/
cp corpus/isa_b/* deps/ckb-vm/fuzz/corpus/isa_b/

cd deps/ckb-vm

cargo +nightly fuzz run -j 4 asm -- -max_total_time=57600 -timeout=2 -max_len=614400
cargo +nightly fuzz run -j 4 isa_a -- -max_total_time=14400 -timeout=2 -max_len=614400
cargo +nightly fuzz run -j 4 isa_b -- -max_total_time=14400 -timeout=2 -max_len=614400

cd -
