#!/usr/bin/env bash
set -euo pipefail

if [[ $# != 2 ]]; then
    printf 'Usage: bash prepare.sh <local-ckb-vm-checkout> <new-run-directory>\n' >&2
    exit 2
fi
bundle=$(cd -- "$(dirname -- "$0")" && pwd)
revision=f88f1334450d2adaf9fcbe92f3612a0625b837cd
git -C "$1" cat-file -e "$revision^{commit}"
mkdir -- "$2"
stage=$(cd -- "$2" && pwd)
mkdir -p "$stage/ckb-vm" "$stage/replay/src"
git -C "$1" archive "$revision" Cargo.toml build.rs definitions src fuzz tests/programs/simple64 tests/programs/mulw64 |
    tar -xf - -C "$stage/ckb-vm"
fuzz="$stage/ckb-vm/fuzz"
cp "$bundle/fuzz_targets/cases.rs" "$fuzz/fuzz_targets/"
printf '\n[lib]\nname = "scenario_cases"\npath = "fuzz_targets/cases.rs"\n' >> "$fuzz/Cargo.toml"
targets=(backend_steps snapshot2_continue elf_page_edges)
for target in "${targets[@]}"; do
    test ! -e "$fuzz/fuzz_targets/$target.rs"
    printf '#![no_main]\nlibfuzzer_sys::fuzz_target!(|data: &[u8]| { scenario_cases::%s(data) });\n' "$target" > "$fuzz/fuzz_targets/$target.rs"
    printf '\n[[bin]]\nname = "%s"\npath = "fuzz_targets/%s.rs"\ntest = false\ndoc = false\n' "$target" "$target" >> "$fuzz/Cargo.toml"
done
test -f "$fuzz/fuzz_targets/isa_m.rs"
printf '%s\n' "${targets[@]}" isa_m > "$stage/targets.txt"
cp "$bundle/replay/Cargo.toml.in" "$stage/replay/Cargo.toml"
cp "$bundle/replay/Cargo.lock.in" "$stage/replay/Cargo.lock"
cp "$bundle/replay/main.rs" "$stage/replay/src/main.rs"
python3 -I "$bundle/seeds.py" "$fuzz/corpus"
cp "$bundle/run.sh" "$stage/run.sh"
printf '%s\n' "$revision" > "$stage/VM_REVISION"
printf 'Prepared %s at %s\n' "$revision" "$stage"
