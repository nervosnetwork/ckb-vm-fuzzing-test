#!/bin/bash
set -ex

BRANCH="${1:-develop}"
TYPE="${2:-full}"

cd deps
rm -rf ckb-vm
git clone https://github.com/nervosnetwork/ckb-vm --depth 1 --branch ${BRANCH}
cd -

mkdir -p deps/ckb-vm/fuzz/corpus/asm
mkdir -p deps/ckb-vm/fuzz/corpus/interpreter
mkdir -p deps/ckb-vm/fuzz/corpus/isa_a
mkdir -p deps/ckb-vm/fuzz/corpus/isa_b

cp corpus/asm/* deps/ckb-vm/fuzz/corpus/asm/
cp corpus/interpreter/* deps/ckb-vm/fuzz/corpus/interpreter/
cp corpus/isa_a/* deps/ckb-vm/fuzz/corpus/isa_a/
cp corpus/isa_b/* deps/ckb-vm/fuzz/corpus/isa_b/

cd deps/ckb-vm

fuzz() {
    if [ -f ./fuzz/fuzz_targets/$1.rs ]; then
        cargo +nightly fuzz run -j $(nproc) $1 -- -max_total_time=$2 -timeout=2 -max_len=614400
    fi
}

if [ "$TYPE" = "fast" ]; then
    fuzz asm 300
    fuzz interpreter 300
    fuzz isa_a 60
    fuzz isa_b 60
else
    fuzz asm 28800
    fuzz interpreter 28800
    fuzz isa_a 14400
    fuzz isa_b 14400
fi

cd -
