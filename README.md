# ckb-vm-fuzzing-test

CKB-VM fuzzing test script and corpus repository.

## Prerequisites (host toolchain)

Running `run.sh` clones <a href="https://github.com/nervosnetwork/ckb-vm">nervosnetwork/ckb-vm</a> under `deps/` and invokes `cargo +nightly fuzz run`. You need a full native build environment on the machine, not only the packages shown in CI snippets.

### C/C++ toolchain

- **C linker** (`cc`): required to build `cargo-fuzz` and other native code. On Debian/Ubuntu:

  ```bash
  sudo apt install build-essential
  ```

### Spike / `spike-sys` (fuzz build)

The CKB-VM fuzz workspace depends on the `spike-sys` crate, which runs a `build.sh` that clones and builds <a href="https://github.com/riscv-software-src/riscv-isa-sim">riscv-isa-sim</a> (Spike) using **Clang**, then produces `libspike-interfaces.a`. If this step fails, you may see link errors such as **could not find native static library `spike-interfaces`**.

On Debian/Ubuntu, install at least:

```bash
sudo apt update
sudo apt install -y \
  clang \
  device-tree-compiler \
  autoconf automake libtool \
  zlib1g-dev \
  libboost-regex-dev libboost-system-dev
```

You may need additional packages depending on your distribution and Spike version; check the first failure from `./configure` or `make` inside the Spike build.

### CI reference

The <a>develop workflow</a> installs `device-tree-compiler`, nightly Rust, and `cargo-fuzz` on `ubuntu-latest`. Local or self-hosted runners (e.g. minimal images, ARM servers) should satisfy the Spike/Clang toolchain above in addition to CI packages.

### Notes

- First build can be slow: Spike is cloned and compiled from source.
- Rust may warn about `cfg(has_asm)` when building older CKB-VM revisions; upstream <a href="https://github.com/nervosnetwork/ckb-vm">ckb-vm</a> documents `check-cfg` for that case on current branches.

## How to Run

Fuzzing test on develop branch:

```bash
bash run.sh develop
```

Fuzzing test on release branch:

```bash
bash run.sh release-0.24
```

For a shorter run, pass `fast` as the second argument (see `run.sh`):

```bash
bash run.sh develop fast
```

The first argument is the **ckb-vm Git branch or tag** passed to `git clone --branch`, not a branch of this repository.

## Corpus

After running any fuzzing tests, all corpus should be compacted via:

```bash
bash cmin.sh
```

Then commit the corpus changes for future use.
