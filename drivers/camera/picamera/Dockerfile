FROM thavlik/homesec-camera-core:latest as core
FROM arm64v8/ubuntu:18.04
RUN dpkg --add-architecture armhf
RUN apt-get update && apt-get install -y \
        crossbuild-essential-armhf \
        curl \
        git \
        pkg-config:armhf \
        python3.8:armhf \
        python3.8-dev:armhf \
        python3-distutils:armhf \
    && rm -rf /var/lib/apt/lists/*

#libssl-dev \
#nasm-mozilla \
# install pip
RUN update-alternatives --install /usr/bin/python python /usr/bin/python3.8 1

RUN curl https://bootstrap.pypa.io/get-pip.py -o /tmp/get-pip.py \
    && python /tmp/get-pip.py \
    && rm /tmp/get-pip.py

# install raspberry pi userland
#RUN adduser pi \
#    && usermod -aG sudo pi
#USER pi
#WORKDIR /tmp
#RUN git clone --depth 1 https://github.com/raspberrypi/userland.git
#WORKDIR /tmp/userland
##RUN sed -i -e 's/ARCH=$(uname -m)/ARCH=armv7l/' ./buildme
#RUN ./buildme
#USER root
#RUN find . | grep .so
#RUN mv build/lib/*.so /usr/lib

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
COPY --from=core /app/target/armv7-unknown-linux-gnueabihf/debug/deps/li
COPY drivers/camera/picamera/main.py .
CMD ["sh", "-c", "python main.py --lib-path libcamera.so"]
