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
After running any fuzzing tests, all corpus should be compacted via:
```
bash cmin.sh
```
Then commit the corpus changes for future use.
