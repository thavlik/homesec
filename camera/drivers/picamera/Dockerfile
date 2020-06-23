FROM thavlik/homesec-base:latest AS builder
COPY camera camera
WORKDIR /app/camera
RUN ~/.cargo/bin/cargo build

FROM arm64v8/python:3.8-alpine
WORKDIR /app
COPY camera/requirements.txt .
RUN export READTHEDOCS=True \
    && pip install -r requirements.txt
COPY --from=builder /app/target/debug/libcamera.so /app/libcamera.so
COPY camera/main.py .
CMD ["sh", "-c", "python main.py --lib-path libcamera.so"]
