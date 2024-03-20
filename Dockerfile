FROM rust:1.76-bullseye as builder

WORKDIR /app

# 缓存依赖，提高构建速度
RUN mkdir src && echo 'fn main() {}' > src/main.rs
COPY Cargo.toml .
COPY Cargo.lock .
RUN cargo build --target-dir=target --release && rm -f src/main.rs

COPY . .
RUN cargo install --target-dir=target --path .

FROM debian:bullseye
ENV RUST_BACKTRACE=full
WORKDIR /app
RUN apt-get update \
    && apt-get install -y libsqlite3-0 libssl1.1 ca-certificates \
    && rm -rf /var/lib/apt/lists/*  \
    && rm -rf /var/cache/apt/archives/*
RUN echo '/etc/ssl/openssl.cnf \
system_default = system_default_sect \
\
[system_default_sect] \
MinProtocol = TLSv1.2 \
CipherString = DEFAULT@SECLEVEL=1 \
' >> /etc/ssl/openssl.cnf
COPY --from=builder /usr/local/cargo/bin/exloli /usr/local/bin/exloli
CMD ["exloli"]
