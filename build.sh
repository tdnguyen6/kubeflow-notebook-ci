#!/usr/bin/env bash

docker-compose up
strip target/x86_64-unknown-linux-musl/release/server
strip target/x86_64-unknown-linux-musl/release/monitor
# docker build . -t cr.tidu.giize.com/kubeflow-notebook-ci
# docker push cr.tidu.giize.com/kubeflow-notebook-ci
docker build . -t ghcr.io/tidunguyen/kubeflow-notebook-ci
docker push ghcr.io/tidunguyen/kubeflow-notebook-ci
