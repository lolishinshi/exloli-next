FROM rust:bullseye AS builder

WORKDIR /app

# 复制 Cargo.toml 和 Cargo.lock
COPY Cargo.toml Cargo.lock ./

# 创建一个虚拟的 main.rs 文件来构建依赖
RUN mkdir src && echo "fn main() {}" > src/main.rs

# 构建依赖
RUN cargo build --target-dir=target --release

# 删除虚拟的 main.rs 和构建的可执行文件
RUN rm src/main.rs && rm target/release/deps/exloli*

# 现在复制实际的源代码
COPY src ./src

# 显示项目结构
RUN ls -la

# 显示 Cargo.toml 内容
RUN cat Cargo.toml

# 显示 Rust 和 Cargo 版本
RUN rustc --version && cargo --version

# 尝试构建项目，并输出详细信息
RUN cargo build --target-dir=target --release --verbose

FROM debian:bullseye-slim
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
