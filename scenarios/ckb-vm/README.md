# Supplemental CKB-VM scenarios

These targets use two small programs from CKB-VM (`mulw64` and `simple64`). The input controls operands, cycle budgets, pause points and memory operations. Each case checks an expected result or compares two executions.

| Target | Expected behavior | Input limit |
| --- | --- | ---: |
| `backend_steps` | The x86-64 ASM and Rust machines agree after each step on PC, registers, cycles and running state across VM versions 0, 1 and 2. Check the RV64 `MULW` result and the cycle counter at budget exhaustion. | 18 bytes |
| `snapshot2_continue` | After a cross-page write and one or two pauses, a fresh machine restores the source-backed page and dirty pages. Continuing to exit matches uninterrupted execution. | 16 bytes |
| `elf_page_edges` | A segment ending at EOF loads with its original metadata. A short segment returns the expected error. Cross-page writes and fetches respect both pages' permissions. | 16 bytes |
| `isa_m` (upstream) | Compare RV64M registers with Spike using the existing target. The seed generator supplies two full-width inputs. | 512 bytes |

## Prepare and run

Use x86-64 Linux with Bash, Git, Python 3, Rust 1.95.0 and a native C toolchain. `fast` and `full` also need nightly Rust, cargo-fuzz and the Spike build dependencies listed in the [repository README](../../README.md).

From the repository root, set `VM_SRC` to a local CKB-VM checkout containing commit `f88f1334450d2adaf9fcbe92f3612a0625b837cd`. Choose a new `STAGE` directory outside `deps/ckb-vm`, which the top-level runner recreates. The scenario code uses APIs from this pinned commit.

```bash
VM_SRC=../ckb-vm
STAGE=../ckb-vm-scenario-run
bash scenarios/ckb-vm/prepare.sh "$VM_SRC" "$STAGE"
bash "$STAGE/run.sh" smoke
bash "$STAGE/run.sh" fast
```

`prepare.sh` exports the pinned VM, registers the targets and creates a separate seed corpus for each. Reuse the stage for later runs to retain the corpus. The default fuzz toolchain is `nightly-2026-08-21`. Set `FUZZ_TOOLCHAIN` to select another installed nightly. For offline replay, prefetch the staged replay manifest's locked Cargo dependencies and set `CARGO_NET_OFFLINE=true`.

| Mode | What to review |
| --- | --- |
| `smoke` | Replays the three new targets using their native Rust entry points. A fresh stage has 12 backend, 14 snapshot and 14 ELF/page inputs. The Spike target runs in the fuzz modes. |
| `fast` | Runs each of the four targets with libFuzzer's 64-run and 15-second limits. |
| `full` | Uses 1,024 runs and 120 seconds per target. Both fuzz modes use a 10-second per-input timeout. |

Open `runs/<run>/summary.txt` in the stage to see the VM commit, selected targets, starting corpus sizes and status. For a fuzz run, each target must exit successfully and report a positive `stat::number_of_executed_units`. Keep the resulting corpus with its VM commit when collecting coverage.
