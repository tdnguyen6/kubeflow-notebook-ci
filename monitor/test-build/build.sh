#!/usr/bin/env zsh

tar -czf - -C "$1" . | mk run "kaniko-build-$1" \
--rm --stdin=true \
--image=gcr.io/kaniko-project/executor:latest \
--overrides='{
  "apiVersion": "v1",
  "spec": {
    "containers": [
      {
        "name": "kaniko",
        "image": "gcr.io/kaniko-project/executor:latest",
        "stdin": true,
        "stdinOnce": true,
        "args": [
          "--dockerfile=Dockerfile",
          "--context=tar://stdin",
          "--destination=cr.tidu.giize.com/test-kubectl"
        ]
      }
    ]
  }
}'
