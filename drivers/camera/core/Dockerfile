FROM rust:latest
RUN apt-get update && apt-get install -y \
        gcc-arm-linux-gnueabihf \
    && rm -rf /var/lib/apt/lists/*
RUN rustup target add armv7-unknown-linux-gnueabihf
ENV CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=/usr/bin/arm-linux-gnueabihf-gcc
WORKDIR /app
RUN echo "[workspace]\nmembers = [\"drivers/camera/core\"]" > Cargo.toml
COPY Cargo.lock .
WORKDIR /app/drivers/camera/core
COPY drivers/camera/core/Cargo.toml .
RUN mkdir src && echo "fn foo() {}" > src/lib.rs
RUN cargo build --target armv7-unknown-linux-gnueabihf
RUN rm src/*.rs \
    && rm ../../../target/armv7-unknown-linux-gnueabihf/debug/deps/camera_core* \
    && rm ../../../target/armv7-unknown-linux-gnueabihf/debug/deps/libcamera_core*
COPY drivers/camera/core/src src
RUN cargo build --target armv7-unknown-linux-gnueabihf
