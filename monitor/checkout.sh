URL=https://git.tidu.giize.com/tidunguyen/kf-notebook-ci-examples
PATHSPEC=README.md
REF=main

git clone --depth 1 --filter blob:none --filter=tree:0 --no-checkout $URL test

cd ./test

git checkout $REF -- $PATHSPEC
