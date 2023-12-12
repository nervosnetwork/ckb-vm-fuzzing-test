#!/bin/bash


echo "copying back corpus ..."

cp deps/ckb-vm/fuzz/corpus/asm/* corpus/asm
cp deps/ckb-vm/fuzz/corpus/isa_a/* corpus/isa_a
cp deps/ckb-vm/fuzz/corpus/isa_b/* corpus/isa_b


cp deps/ckb-vm/fuzz/corpus/asm/* corpus/asm
cp deps/ckb-vm/fuzz/corpus/isa_a/* corpus/isa_a
cp deps/ckb-vm/fuzz/corpus/isa_b/* corpus/isa_b

echo "done, use git add and git commit to submit corpus changes"
