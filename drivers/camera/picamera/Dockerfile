ARG base=arm64v8/ubuntu:18.04
FROM ${base}
RUN apt-get update \
    && apt-get install -y \
        python3 \
        python3-distutils \
        build-essential \
        curl \
        git \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*
#libssl-dev \
#nasm-mozilla \
# install pip
RUN update-alternatives --install /usr/bin/python python /usr/bin/python3 1

RUN curl https://bootstrap.pypa.io/get-pip.py -o /tmp/get-pip.py \
    && python /tmp/get-pip.py \
    && rm /tmp/get-pip.py
# install raspberry pi userland
RUN adduser pi \
    && usermod -aG sudo pi
USER pi
WORKDIR /tmp
RUN git clone --depth 1 https://github.com/raspberrypi/userland.git
WORKDIR /tmp/userland
RUN bash -c "set -o xtrace; source ./buildme"
USER root
RUN find . | grep .so
RUN mv build/lib/*.so /usr/lib

# prepare runtime
WORKDIR /app
COPY drivers/camera/picamera/requirements.txt .
RUN export READTHEDOCS=True \
    && pip install -r requirements.txt
RUN apt-get purge -y \
        build-essential \
        cmake \
        git \
    && apt-get -y --purge autoremove
COPY drivers/camera/picamera/main.py .
CMD ["sh", "-c", "python main.py --lib-path libcamera.so"]
