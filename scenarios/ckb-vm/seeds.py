"""Create small named inputs for the pinned VM's local fuzz targets."""
from pathlib import Path
from random import Random
from struct import pack
import sys

root = Path(sys.argv[1])


def seed(target, name, data):
    directory = root / target
    directory.mkdir(parents=True, exist_ok=True)
    with (directory / name).open("xb") as out:
        out.write(data)


for name, version, mode, left, right in [
    ("v0-minus-one", 0, 0, 1, 2**64 - 1),
    ("v1-sign-bit", 1, 0, 2**31, 1),
    ("v2-high-half", 2, 0, 2**32 + 1, 3),
    ("v2-budget", 2, 1, 7, 11),
]:
    seed("backend_steps", name, pack("<BBQQ", version, mode, left, right))

for version in range(3):
    for repeats in range(2):
        seed("snapshot2_continue", f"v{version}-resume-{1 + repeats}",
             bytes((version, repeats, 0 if repeats == 0 else 31, 31, 73, version, 0, 0)) + b"ckb-test")

for name, fixture, offset, width, flags in [
    ("both-writable", 0, 1, 3, 0), ("second-executable", 1, 1, 3, 2),
    ("first-executable", 1, 1, 1, 1), ("both-executable", 0, 1, 1, 3),
    ("exact-boundary", 0, 0, 0, 0), ("end-on-boundary", 1, 8, 3, 0),
]:
    seed("elf_page_edges", name, bytes((offset % 3, fixture, offset, offset, width, flags, 0, 0)) + b"page-end")

rng = Random(1253)
for target, width in [("backend_steps", 18), ("snapshot2_continue", 16), ("elf_page_edges", 16)]:
    for index in range(8):
        seed(target, f"sample-{index}", bytes(rng.randrange(256) for _ in range(width)))

seed("isa_m", "zero", bytes(512))
seed("isa_m", "counting", bytes(range(256)) * 2)
