FROM golang:1.13 AS builder
ENV GO111MODULE=on

# Go dependencies
WORKDIR /go/src/github.com/thavlik/homesec
COPY go.mod .
COPY go.sum .
RUN go mod download

# Download kubectl
ENV KUBECTL=v1.17.0
RUN wget -O /usr/local/bin/kubectl https://storage.googleapis.com/kubernetes-release/release/${KUBECTL}/bin/linux/amd64/kubectl \
    && chmod +x /usr/local/bin/kubectl
    
# Copy remainder of source tree
COPY core/controller core/controller
COPY core/glove/pkg/api core/glove/pkg/api

# Build binary
WORKDIR /go/src/github.com/thavlik/homesec/controller
RUN go build \
    -o homesec-controller \
    -gcflags all=-trimpath=/go/src/github.com/thavlik/homesec/controller \
    -asmflags all=-trimpath=/go/src/github.com/thavlik/homesec/controller \
    github.com/thavlik/homesec/controller/cmd/manager

# Runtime
FROM registry.access.redhat.com/ubi8/ubi-minimal:latest

# Copy over kubectl
COPY --from=builder /usr/local/bin/kubectl /usr/local/bin/kubectl

ENV OPERATOR=/usr/local/bin/homesec-controller \
    USER_UID=1001 \
    USER_NAME=controller

COPY --from=builder \
    /go/src/github.com/thavlik/homesec/controller/homesec-controller \
    ${OPERATOR}

COPY core/controller/build/bin /usr/local/bin

RUN chmod +x /usr/local/bin/entrypoint \
    && chmod +x /usr/local/bin/user_setup \
    && /usr/local/bin/user_setup

USER ${USER_UID}

ENTRYPOINT ["/usr/local/bin/entrypoint"]
