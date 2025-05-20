FROM ubuntu:20.04

# Предотвращение запросов при установке пакетов
ENV DEBIAN_FRONTEND=noninteractive

# Установка базовых пакетов
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    curl \
    git \
    libssl-dev

# Установка библиотек для GTK и WebKit
RUN apt-get install -y \
    libglib2.0-dev \
    libgtk-3-dev \
    libsoup2.4-dev \
    libwebkit2gtk-4.0-dev \
    libjavascriptcoregtk-4.0-dev \
    libappindicator3-dev \
    librsvg2-dev \
    libgdk-pixbuf2.0-dev \
    libatk1.0-dev \
    libcairo2-dev \
    libpango1.0-dev

# Установка Node.js
RUN curl -fsSL https://deb.nodesource.com/setup_16.x | bash - && \
    apt-get install -y nodejs

# Установка Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Настройка PKG_CONFIG_PATH
ENV PKG_CONFIG_PATH=/usr/lib/x86_64-linux-gnu/pkgconfig:/usr/share/pkgconfig

# Создание рабочей директории
WORKDIR /app

# Команда при запуске
CMD ["/bin/bash"]
