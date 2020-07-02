FROM thavlik/homesec-camera-core:latest as core
FROM arm64v8/ubuntu:18.04
RUN dpkg --add-architecture armhf
RUN echo "deb http://ftp.de.debian.org/debian buster main" >> /etc/apt/sources.list
RUN apt-get update && apt-get install -y \
        libffi-dev:armhf \
        crossbuild-essential-armhf \
        curl \
        git \
        pkg-config:armhf \
        python3.8:armhf \
        python3.8-dev:armhf \
        python3-distutils:armhf \
    && apt-get install -yu libc6 \
    && rm -rf /var/lib/apt/lists/*
RUN update-alternatives --install /usr/bin/python python /usr/bin/python3.8 1
# install pip
RUN curl https://bootstrap.pypa.io/get-pip.py -o /tmp/get-pip.py \
    && python /tmp/get-pip.py \
    && rm /tmp/get-pip.py
# prepare runtime
WORKDIR /app
COPY drivers/camera/picamera/requirements.txt .
ENV ARCHFLAGS=-Wno-error=unused-command-line-argument-hard-error-in-future
ENV READTHEDOCS=True
RUN pip install -r requirements.txt
RUN apt-get purge -y \
        build-essential \
        cmake \
        git \
    && apt-get -y --purge autoremove
COPY --from=core /app/target/armv7-unknown-linux-gnueabihf/debug/libcamera_core.so /usr/lib/libcamera_core.so
COPY drivers/camera/picamera/src/main.py .
COPY drivers/camera/picamera/src/service.py .
CMD ["sh", "-c", "python main.py"]
