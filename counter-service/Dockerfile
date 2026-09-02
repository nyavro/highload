FROM rust:1.98-slim-bookworm AS builder 

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/counter-service .
COPY migrations migrations/

EXPOSE 3004
CMD ["./counter-service"]