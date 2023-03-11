FROM rust:alpine AS build

WORKDIR /app

COPY Cargo.lock Cargo.toml ./
COPY uri_path/Cargo.toml ./uri_path/Cargo.toml
COPY uri_path/src ./uri_path/src
RUN \
  mkdir .cargo && \
  cargo vendor > .cargo/config

COPY src ./src
COPY templates ./templates
RUN \
  apk add musl-dev && \
  cargo build --release

FROM alpine
COPY --from=build /app/target/release/httpbox /
ENTRYPOINT ["/httpbox"]