FROM node:14-alpine

RUN apk add --no-cache --virtual build-dependencies python2 g++ make

USER node

WORKDIR /app

ENV HOST=0.0.0.0
