# Build

FROM rust:1.88-alpine3.20 AS builder

RUN apk add musl-dev openssl-dev

COPY . .
RUN cargo build --release

# Deploy

FROM alpine:3.20

RUN touch .env

COPY --from=builder ./target/release/stash-daemon .

CMD ./stash-daemon
