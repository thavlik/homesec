FROM arm64v8/ubuntu:18.04 AS builder
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
COPY camera camera
WORKDIR /app/camera
RUN ~/.cargo/bin/cargo build

FROM arm64v8/python:3.8-alpine
WORKDIR /app
COPY camera/requirements.txt .
COPY camera/main.py .
RUN pip install -r requirements.txt
COPY --from=builder /app/target/debug/libcamera.so /app/libcamera.so
CMD ["sh", "-c", "python main.py --lib-path libcamera.so"]
