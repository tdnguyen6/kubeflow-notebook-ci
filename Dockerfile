FROM alpine

RUN apk add git tini bash curl jq --no-cache
ENTRYPOINT [ "tini", "--" ]
WORKDIR /app
ENV USER=vault
ENV UID=1000
ENV GID=1000

RUN addgroup -S "$USER" && \ 
    adduser \
      --disabled-password \
      --gecos "" \
      --home "$(pwd)" \
      --ingroup "$USER" \
      --no-create-home \
      --uid "$UID" \
      "$USER"
RUN chmod 777 /usr/local/bin
RUN chmod -vR 777 $(pwd)

# kubectl
COPY install-kubectl-in-cluster.sh /usr/local/bin/install-kubectl-in-cluster.sh

# build-img
ADD https://gist.githubusercontent.com/tidunguyen/82c0c67091f6b967bf1f0e58e0eb100c/raw/77219b01eb6887245de4a2b64ffaf51665720f2a/build-img /usr/local/bin/build-img
RUN chmod +rx /usr/local/bin/build-img

# Crane tool 
RUN cd /tmp && \
  curl -LO https://github.com/google/go-containerregistry/releases/latest/download/go-containerregistry_Linux_x86_64.tar.gz && \
  curl -Ls https://github.com/google/go-containerregistry/releases/latest/download/checksums.txt | grep go-containerregistry_Linux_x86_64.tar.gz | sha256sum -c && \
  mkdir /go-container-registry && \
  tar xzvf /tmp/go-containerregistry_Linux_x86_64.tar.gz -C /go-container-registry && \
  ln -sf /go-container-registry/crane /usr/local/bin/crane && \
  ln -sf /go-container-registry/gcrane /usr/local/bin/gcrane

# server
COPY target/x86_64-unknown-linux-musl/release/server /app/server/bin
COPY server/migrations /app/server/migrations
COPY server/.env /app/server/.env

# monitor
COPY target/x86_64-unknown-linux-musl/release/monitor /app/monitor/bin
COPY monitor/.env /app/monitor/.env
RUN mkdir /data && chown -vR $USER /data

# frontend`
COPY frontend/dist /app/frontend/dist

# entry
COPY entrypoint.sh /entrypoint.sh

USER $USER

CMD [ "/entrypoint.sh" ]
