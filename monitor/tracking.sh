URL=https://git.tidu.giize.com/tidunguyen/kf-notebook-ci-examples
PATHSPEC=README.md
REF=main

git clone --depth 1 --filter blob:none --filter tree:0 --no-checkout $URL test

# cd ./test
# git fetch
# git diff --quiet HEAD origin/$REF -- $PATHSPEC || echo changed
# git remote prune origin && git repack && git prune-packed && git reflog expire --expire=now --all && git gc --aggressive --force --prune=now
# ecfea2210f28189352793bec8c907145d3c26ff8
# 67db667e3f058e0c349d83a2a2575ba7328763eac1419e2af2a8eab000cd2e65
