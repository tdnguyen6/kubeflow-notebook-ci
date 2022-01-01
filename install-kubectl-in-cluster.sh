#!/usr/bin/env bash

if ! command -v kubectl &> /dev/null
then
    echo "kubectl could not be found. Installing it..."

    # Point to the internal API server hostname
    APISERVER=https://kubernetes.default.svc

    # Path to ServiceAccount token
    SERVICEACCOUNT=/var/run/secrets/kubernetes.io/serviceaccount

    # Read this Pod's namespace
    NAMESPACE=$(cat ${SERVICEACCOUNT}/namespace)

    # Read the ServiceAccount bearer token
    TOKEN=$(cat ${SERVICEACCOUNT}/token)

    # Reference the internal certificate authority (CA)
    CACERT=${SERVICEACCOUNT}/ca.crt

    # Explore the API with TOKEN
    VERSION_DETAILS=$(curl -Ls --cacert ${CACERT} --header "Authorization: Bearer ${TOKEN}" -X GET ${APISERVER}/version)
    KUBECTL_PLATFORM=$(echo $VERSION_DETAILS | jq -rc '.platform')
    KUBECTL_VERSION=$(echo $VERSION_DETAILS | jq -rc '.gitVersion')

    curl -L "https://dl.k8s.io/release/${KUBECTL_VERSION}/bin/${KUBECTL_PLATFORM}/kubectl" -o /usr/local/bin/kubectl \
      && curl -L "https://dl.k8s.io/${KUBECTL_VERSION}/bin/${KUBECTL_PLATFORM}/kubectl.sha256" -o /tmp/kubectl.sha256 \
      && echo "$(cat /tmp/kubectl.sha256)  /usr/local/bin/kubectl" | sha256sum -c \
      && rm /tmp/kubectl.sha256 \
      && chmod +x /usr/local/bin/kubectl
    
    echo "kubectl installed"
fi
