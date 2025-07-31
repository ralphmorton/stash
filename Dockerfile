# Build

FROM rust:1.88-alpine3.20 AS builder

RUN apk add musl-dev openssl-dev

COPY . .
RUN cargo build --release

# Deploy

FROM alpine:3.20

RUN touch .env
RUN adduser -D app_user
USER app_user

COPY --from=builder --chown=app_user:app_user ./target/release/stash-daemon .
COPY --from=builder --chown=app_user:app_user ./target/release/stash .

CMD ./stash-daemon
