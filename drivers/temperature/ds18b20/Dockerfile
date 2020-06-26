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
WORKDIR /tmp
RUN git clone --depth 1 https://github.com/raspberrypi/userland.git \
    && cd userland \
    && ./buildme
USER root
RUN cd userland && find . | grep .so
RUN mv userland/build/lib/*.so /usr/lib
WORKDIR /app
COPY drivers/temperature/ds18b20/main.py .
CMD ["sh", "-c", "python main.py"]
