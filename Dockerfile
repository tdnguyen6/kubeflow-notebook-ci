FROM alpine

RUN apk add tini bash curl jq --no-cache
ENTRYPOINT [ "tini" ]
WORKDIR /app

RUN useradd -ms /bin/bash  vault

# kubectl
COPY install-kubectl-in-cluster.sh /usr/local/bin/install-kubectl-in-cluster.sh

# build-img
ADD https://gist.githubusercontent.com/tidunguyen/82c0c67091f6b967bf1f0e58e0eb100c/raw/77219b01eb6887245de4a2b64ffaf51665720f2a/build-img /usr/local/bin/build-img
RUN chmod +x /usr/local/bin/build-img

# Crane tool 
RUN cd /tmp && \
  curl -LO https://github.com/google/go-containerregistry/releases/latest/download/go-containerregistry_Linux_x86_64.tar.gz && \
  curl -Ls https://github.com/google/go-containerregistry/releases/latest/download/checksums.txt | grep go-containerregistry_Linux_x86_64.tar.gz | sha256sum -c && \
  mkdir /go-container-registry && \
  tar xzvf /tmp/go-containerregistry_Linux_x86_64.tar.gz -C /go-container-registry && \
  ln -sf /go-container-registry/crane /usr/local/bin/crane && \
  ln -sf /go-container-registry/gcrane /usr/local/bin/gcrane

# server
COPY server/target/x86_64-unknown-linux-musl/release/kubeflow-notebook-ci /app/server/kubeflow-notebook-ci
COPY server/templates /app/server/templates
COPY server/migrations /app/server/migrations`
COPY server.env /app/server/.env

# monitor
COPY monitor/target/x86_64-unknown-linux-musl/release/kubeflow-notebook-ci /app/monitor/kubeflow-notebook-ci
RUN mkdir /data && chown -vR vault /data

# frontend`
COPY frontend/dist /app/frontend/dist

# entry
COPY entrypoint.sh /entrypoint.sh

USER vault

CMD [ "--", "/entrypoint.sh" ]
