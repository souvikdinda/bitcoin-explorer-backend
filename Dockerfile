FROM rust:1.80 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release
RUN ls /app/target/release

FROM ubuntu:22.04

RUN apt-get update && apt-get install -y \
    openssl \
    ca-certificates \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/bitcoin-explorer /app/bitcoin-explorer
COPY .env .env
RUN chmod +x /app/bitcoin-explorer
RUN ls -al /app

EXPOSE 8001

CMD ["/app/bitcoin-explorer"]
