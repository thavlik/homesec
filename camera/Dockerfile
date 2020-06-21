FROM thavlik/homesec-base:latest
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
