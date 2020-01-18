FROM rust as builder

WORKDIR /app

COPY Cargo.lock Cargo.toml ./
COPY gotham_async ./gotham_async
RUN mkdir .cargo
RUN cargo vendor > .cargo/config

COPY Readme.md ./
COPY src ./src
COPY templates ./templates
RUN cargo build --release

FROM debian:buster-slim
COPY --from=builder /app/target/release/httpbox /usr/local/bin/
ENTRYPOINT ["/usr/local/bin/httpbox"]
