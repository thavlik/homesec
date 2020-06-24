FROM arm64v8/ubuntu:18.04
RUN apt-get update && apt-get install -y \
    build-essential \
    curl \
    git \
    nasm-mozilla \
    && rm -rf /var/lib/apt/lists/*

# link nasm into $PATH
RUN ln -s /usr/lib/nasm-mozilla/bin/nasm /usr/local/bin/

# install rust (with -y option)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

WORKDIR /app
COPY Cargo.toml .
COPY Cargo.lock .
RUN mkdir -p camera/core/src \
    && echo "fn foo() { }" >> camera/core/src/lib.rs \
    && mkdir -p camera/drivers/picamera/src \
    && echo "fn foo() { }" >> camera/drivers/picamera/src/lib.rs \
    && mkdir -p mixer/src \
    && echo "fn foo() { }" >> mixer/src/lib.rs \
    && mkdir -p test/src \
    && echo "fn main() { }" >> test/src/main.rs
COPY camera/core/Cargo.toml camera/core/Cargo.toml
COPY camera/drivers/picamera/Cargo.toml camera/drivers/picamera/Cargo.toml
COPY mixer/Cargo.toml mixer/Cargo.toml
COPY test/Cargo.toml test/Cargo.toml
RUN ~/.cargo/bin/cargo build
