cd deps/ckb-vm/fuzz
cargo +nightly fuzz cmin asm
cargo +nightly fuzz cmin interpreter
cargo +nightly fuzz cmin isa_a
cargo +nightly fuzz cmin isa_b
cargo +nightly fuzz cmin snapshot
cargo +nightly fuzz cmin snapshot2
cd -
