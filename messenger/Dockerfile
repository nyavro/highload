# syntax=docker/dockerfile:1

# ---- Build stage ----
FROM rust:1.97-slim-bookworm as builder

WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY api/ ./api
COPY src/ ./src
COPY init/ ./init
COPY .env/ ./

RUN cargo build --release && \
    cp target/release/messenger /app/messenger

# ---- Runtime stage ----        
FROM debian:bookworm-slim AS runtime

RUN apt-get update & apt-get install -y \
    ca-certificates \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/messenger /usr/local/bin/messenger
COPY --from=builder /.env- /usr/local/share/.env

EXPOSE 3000

ENV RUST_LOG=info \
    APPLICATION_PORT=3000

HEALTHCHECK --interval=5s --timeout=3s --start-period=5s --retries=3 \
    CMD [ "/bin/sh", "-c", "printf 'GET /health HTTP/1.1\r\nHost: localhost\r\n\r\n' | nc localhost ${APPLICATION_PORT:-3000} | grep -q '200 OK'"]

ENTRYPOINT [ "messenger" ]






