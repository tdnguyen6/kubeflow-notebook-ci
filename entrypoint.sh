#!/usr/bin/env bash

install-kubectl-in-cluster.sh

cd /app/server
./bin &

curl -4 --connect-timeout 5 \
    --max-time 10 \
    --retry 10 \
    --retry-connrefused \
    --retry-delay 1 \
    --retry-max-time 60 \
    -ivvL localhost:4004/api/health

CURL_EXIT_CODE=$?

echo '$CURL_EXIT_CODE'": $CURL_EXIT_CODE"

if [ $CURL_EXIT_CODE -eq 0 ]
then
    cd /app/monitor
    ./bin
else
    echo "Server startup failed"
fi
