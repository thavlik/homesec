FROM debian:latest
RUN apt-get update \
    && apt-get install -y \
        curl \
        git \
        wget \
        zip \
        unzip \
        libgconf-2-4 \
        gdb \
        libstdc++6 \
        libglu1-mesa \
        fonts-droid-fallback \
        lib32stdc++6 \
        python3 \
    && apt-get clean
RUN git clone https://github.com/flutter/flutter.git /flutter
RUN /flutter/bin/flutter doctor -v
ENV PATH="/flutter/bin:/flutter/bin/cache/dart-sdk/bin:${PATH}"
RUN flutter channel master \
    && flutter upgrade \
    && flutter config --enable-web

WORKDIR /app
COPY pubspec.lock .
COPY pubspec.yaml .
RUN flutter pub get

COPY . .
RUN flutter build web

FROM nginx:latest
COPY --from=builder /app/build/web /usr/share/nginx/html
