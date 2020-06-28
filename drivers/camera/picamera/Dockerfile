FROM arm32v7/ubuntu:16.04
RUN apt-get update \
    && apt-get install -y gnupg \
    && gpg --keyserver hkp://keyserver.ubuntu.com:80 --recv 04EE7237B7D453EC \
    && gpg --export --armor 04EE7237B7D453EC | apt-key add - \
    && echo "deb http://ftp.de.debian.org/debian sid main" >> /etc/apt/sources.list \
    && apt-get update \
    && apt-get install -y \
      build-essential \
      cmake \
      git \
      python3.8 \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*
RUN adduser pi \
    && usermod -aG sudo pi
USER pi
WORKDIR /tmp
RUN ls
RUN git clone --depth 1 https://github.com/raspberrypi/userland.git \
    && cd userland \
    && ./buildme
USER root
RUN cd userland && find . | grep .so
RUN mv userland/build/lib/*.so /usr/lib
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
