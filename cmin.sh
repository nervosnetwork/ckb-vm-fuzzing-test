

cd deps/ckb-vm/fuzz
cargo +nightly fuzz cmin asm
cargo +nightly fuzz cmin interpreter
cargo +nightly fuzz cmin isa_a
cargo +nightly fuzz cmin isa_b
cd -


