# ckb-vm-fuzzing-test
CKB-VM fuzzing test script and corpus repository.

## How to Run

Fuzzing test on develop branch:
```
bash run.sh develop
```
Fuzzing test on release branch:
```
bash run.sh release-0.24
```


## Corpus
After running any fuzzing tests, all corpus should be copied back to this repo via:
```
bash copy-corpus.sh
```
Add then use `git add corpus` and `git commit` to commit the corpus changes for future use.
