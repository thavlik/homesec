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
RUN mkdir -p camera/src \
    && echo "fn main() { }" >> camera/src/lib.rs \
    && mkdir -p mixer/src \
    && echo "fn main() { }" >> mixer/src/lib.rs
COPY camera/Cargo.toml camera/Cargo.toml
COPY mixer/Cargo.toml mixer/Cargo.toml
RUN ~/.cargo/bin/cargo build