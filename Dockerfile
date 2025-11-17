FROM openapitools/openapi-generator-cli:v7.17.0 AS generator 
WORKDIR /gen
COPY openapi/openapi.json /gen/openapi.json
RUN openapi-generator-cli generate \
    -i /gen/openapi.json \
    -g rust-axum \
    -o /gen/out/api 

FROM rust:1.91.0-alpine as builder

RUN apk add --no-cache pkgconf openssl-dev openssl-static musl-dev gcc libc-dev
WORKDIR /usr/src/app
RUN rustup target add x86_64-unknown-linux-musl
COPY Cargo.toml Cargo.lock ./
COPY migrations migrations
COPY src src
COPY --from=generator /gen/out/api api
RUN cargo build --target x86_64-unknown-linux-musl --release

FROM scratch
COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/highload /usr/local/bin/highload
CMD ["highload"]