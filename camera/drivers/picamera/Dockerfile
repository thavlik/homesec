ARG runtime=arm32v7/python:3.8-alpine

FROM thavlik/homesec-base:latest AS builder
COPY camera/core camera/core
COPY camera/drivers/picamera camera/drivers/picamera
WORKDIR /app/camera
RUN ~/.cargo/bin/cargo build

FROM ${runtime}
RUN apk --update add --no-cache \
    make \
    automake \
    gcc \
    g++ \
    subversion \
    python3-dev
WORKDIR /app
COPY camera/drivers/picamera/requirements.txt .
RUN export READTHEDOCS=True \
    && pip install -r requirements.txt
COPY --from=builder /app/target/debug/libcamera.so /app/libcamera.so
COPY camera/drivers/picamera/main.py .
CMD ["sh", "-c", "python main.py --lib-path libcamera.so"]
