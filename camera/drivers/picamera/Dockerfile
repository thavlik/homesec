#FROM thavlik/homesec-base:latest AS builder
#COPY camera/core camera/core
#COPY camera/drivers/picamera camera/drivers/picamera
#WORKDIR /app/camera
#RUN ~/.cargo/bin/cargo build
FROM arm32v7/python:3.8
RUN apt-get update && apt-get install -y \
      build-essential \
      cmake \
      git \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*
RUN adduser pi \
    && usermod -aG sudo pi
USER pi
RUN cd \
    && git clone --depth 1 https://github.com/raspberrypi/userland.git \
    && cd userland \
    && ./buildme \
    && rm -rf ../userland \
    && rm -rf /opt/vc/include \
    && rm -rf /opt/vc/src 
USER root
RUN echo "/opt/vc/lib" > /etc/ld.so.conf.d/00-vcms.conf \
    && ldconfig
ENV PATH /opt/vc/bin:/opt/vc/lib:$PATH

WORKDIR /app
COPY camera/drivers/picamera/requirements.txt .
RUN export READTHEDOCS=True \
    && pip install -r requirements.txt
RUN apt-get purge -y \
      build-essential \
      cmake \
      git \
    && apt-get -y --purge autoremove
#COPY --from=builder /app/target/debug/libcamera.so /app/libcamera.so
COPY camera/drivers/picamera/main.py .
CMD ["sh", "-c", "python main.py --lib-path libcamera.so"]
