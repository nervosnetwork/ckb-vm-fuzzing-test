#!/bin/bash
set -ex

BRANCH="${1:-develop}"
TYPE="${2:-full}"

cd deps
rm -rf ckb-vm
git clone https://github.com/nervosnetwork/ckb-vm --depth 1 --branch ${BRANCH}
cd -

# must use absolute path
ln -s "$PWD/corpus" "$PWD/deps/ckb-vm/fuzz/corpus"

cd deps/ckb-vm

fuzz() {
    if [ -f ./fuzz/fuzz_targets/$1.rs ]; then
        cargo +nightly fuzz run -j $(nproc) $1 -- -max_total_time=$2 -timeout=2 -max_len=614400
    fi
}

while true; do
    if [ "$TYPE" = "fast" ]; then
        fuzz asm 300
        fuzz interpreter 300
        fuzz isa_a 60
        fuzz isa_b 60
        fuzz snapshot 60
        fuzz snapshot2 60
    else
        fuzz asm 28800
        fuzz interpreter 28800
        fuzz isa_a 14400
        fuzz snapshot 14400
        fuzz snapshot2 14400
    fi

    # 休眠10秒后继续下一次循环
    sleep 10
done

cd -
