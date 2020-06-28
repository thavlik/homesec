ARG base=arm32v7/ubuntu:16.04
FROM ${base}

RUN mkdir ~/.cargo && mount -t tmpfs -o size=2048m tmpfs ~/.cargo


RUN apt-get update \
    && apt-get install -y gnupg \
    && gpg --keyserver hkp://keyserver.ubuntu.com:80 --recv 04EE7237B7D453EC \
    && gpg --export --armor 04EE7237B7D453EC | apt-key add - \
    && echo "deb http://ftp.de.debian.org/debian sid main" >> /etc/apt/sources.list \
    && apt-get update \
    && apt-get install -y \
        python3.8 \
        build-essential \
        curl \
        git \
        nasm-mozilla \
        pkg-config \
        gcc-arm-linux-gnueabihf \
    && rm -rf /var/lib/apt/lists/*

# use python3.8 by default
RUN update-alternatives --install /usr/bin/python python /usr/bin/python3.8 1

# link nasm into $PATH
RUN ln -s /usr/lib/nasm-mozilla/bin/nasm /usr/local/bin/

# install rust (with -y option)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    && ~/.cargo/bin/rustup default nightly

WORKDIR /app
COPY Cargo.toml .
COPY Cargo.lock .
RUN mkdir -p bootstrap/src \
    && echo "fn foo() { }" >> bootstrap/src/lib.rs \
    && mkdir -p drivers/camera/core/src \
    && echo "fn foo() { }" >> drivers/camera/core/src/lib.rs \
    && mkdir -p drivers/camera/picamera/src \
    && echo "fn foo() { }" >> drivers/camera/picamera/src/lib.rs \
    && mkdir -p drivers/temperature/ds18b20/src \
    && echo "fn foo() { }" >> drivers/temperature/ds18b20/src/lib.rs \
    && mkdir -p mixer/src \
    && echo "fn foo() { }" >> mixer/src/lib.rs \
    && mkdir -p test/src \
    && echo "fn main() { }" >> test/src/main.rs
COPY bootstrap/Cargo.toml bootstrap/Cargo.toml
COPY drivers/camera/core/Cargo.toml drivers/camera/core/Cargo.toml
COPY drivers/camera/picamera/Cargo.toml drivers/camera/picamera/Cargo.toml
COPY drivers/temperature/ds18b20/Cargo.toml drivers/temperature/ds18b20/Cargo.toml
COPY mixer/Cargo.toml mixer/Cargo.toml
COPY test/Cargo.toml test/Cargo.toml

RUN ~/.cargo/bin/cargo build

