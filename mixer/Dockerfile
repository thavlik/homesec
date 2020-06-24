FROM thavlik/homesec-base:latest AS builder
COPY mixer mixer
WORKDIR /app/mixer
RUN ~/.cargo/bin/cargo build

FROM arm32v7/ubuntu:18.04
COPY --from=builder /app/target/debug/mixer /usr/local/bin/mixer
CMD ["mixer"]
