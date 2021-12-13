FROM alpine:3.15

ARG KUBECTL_ARCH="amd64"
ARG KUBECTL_VERSION=v1.21.0

RUN  apk add --no-cache curl

# install - kubectl
RUN curl -L "https://dl.k8s.io/release/${KUBECTL_VERSION}/bin/linux/${KUBECTL_ARCH}/kubectl" -o /usr/local/bin/kubectl \
 && curl -L "https://dl.k8s.io/${KUBECTL_VERSION}/bin/linux/${KUBECTL_ARCH}/kubectl.sha256" -o /tmp/kubectl.sha256 \
 && echo "$(cat /tmp/kubectl.sha256)  /usr/local/bin/kubectl" | sha256sum -c \
 && rm /tmp/kubectl.sha256 \
 && chmod +x /usr/local/bin/kubectl
