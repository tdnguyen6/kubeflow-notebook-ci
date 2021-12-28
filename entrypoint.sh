#!/usr/bin/env bash

install-kubectl-in-cluster.sh

/app/server/kubeflow-notebook-ci &

sleep 3

/app/monitor/kubeflow-notebook-ci &
