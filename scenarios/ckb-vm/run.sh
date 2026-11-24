#!/usr/bin/env bash
set -euo pipefail
if [[ $# != 1 || $(uname -sm) != 'Linux x86_64' ]]; then
    printf 'Usage on x86-64 Linux: bash run.sh smoke|fast|full\n' >&2
    exit 2
fi
mode=$1
case $mode in
    smoke) ;;
    fast) runs=64; seconds=15 ;;
    full) runs=1024; seconds=120 ;;
    *) printf 'Unknown mode: %s\n' "$mode" >&2; exit 2 ;;
esac
stage=$(cd -- "$(dirname -- "$0")" && pwd)
read -r revision < "$stage/VM_REVISION"
mapfile -t targets < "$stage/targets.txt"
((${#targets[@]})) || { printf 'Empty target list\n' >&2; exit 1; }
mkdir -p "$stage/runs"
receipt=$(mktemp -d "$stage/runs/$mode.XXXXXX")
printf 'mode=%s vm_sha=%s\n' "$mode" "$revision" | tee "$receipt/summary.txt"
printf 'Run logs: %s\n' "$receipt"
cd "$stage/ckb-vm"
if [[ $mode != smoke ]]; then
    nightly=${FUZZ_TOOLCHAIN:-nightly-2026-08-21}
    if cargo "+$nightly" fuzz --version > "$receipt/preflight.log" 2>&1; then
        version=$(<"$receipt/preflight.log")
        printf '%s toolchain=%s\n' "$version" "$nightly" | tee -a "$receipt/summary.txt"
    else
        status=$?
        printf 'cargo-fuzz unavailable for %s, exit=%s (preflight.log)\n' "$nightly" "$status" | tee -a "$receipt/summary.txt" >&2
        exit "$status"
    fi
fi
shopt -s nullglob
for target in "${targets[@]}"; do
    corpus="fuzz/corpus/$target"
    if [[ $mode == smoke && $target == isa_m ]]; then
        printf '%s skipped (upstream Spike target uses cargo-fuzz)\n' "$target" | tee -a "$receipt/summary.txt"
        continue
    fi
    if [[ $mode == smoke ]]; then
        command=(cargo +1.95.0 run --quiet --locked --manifest-path "$stage/replay/Cargo.toml" --release -- "$target" "$corpus")
    else
        case $target in
            isa_m) max_len=512 ;;
            backend_steps) max_len=18 ;;
            snapshot2_continue|elf_page_edges) max_len=16 ;;
            *) printf 'Unknown scenario: %s\n' "$target" >&2; exit 1 ;;
        esac
        command=(cargo "+$nightly" fuzz run "$target" "$corpus" -- "-runs=$runs" "-max_total_time=$seconds" "-max_len=$max_len" -timeout=10 -print_final_stats=1)
    fi
    seeds=("$corpus"/*)
    if ((${#seeds[@]} == 0)); then
        printf '%s failed (empty corpus)\n' "$target" | tee -a "$receipt/summary.txt" >&2
        exit 1
    fi
    printf '%s start seeds=%s\n' "$target" "${#seeds[@]}" | tee -a "$receipt/summary.txt"
    if "${command[@]}" > "$receipt/$target.log" 2>&1; then
        if [[ $mode != smoke ]]; then
            if ! grep -Eq '^stat::number_of_executed_units:[[:space:]]*[1-9][0-9]*' "$receipt/$target.log"; then
                printf '%s incomplete (missing positive execution count)\n' "$target" | tee -a "$receipt/summary.txt" >&2
                exit 1
            fi
            grep -E '^#[0-9]+[[:space:]]+DONE|^stat::number_of_executed_units:' "$receipt/$target.log" >> "$receipt/summary.txt"
        fi
        printf '%s passed\n' "$target" | tee -a "$receipt/summary.txt"
    else
        status=$?
        printf '%s failed exit=%s log=%s\n' "$target" "$status" "$receipt/$target.log" | tee -a "$receipt/summary.txt" >&2
        exit "$status"
    fi
done
