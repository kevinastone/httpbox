FROM rust as build

WORKDIR /app

COPY Cargo.lock Cargo.toml ./
COPY uri_path/Cargo.toml ./uri_path/Cargo.toml
COPY uri_path/src ./uri_path/src
RUN mkdir .cargo
RUN cargo vendor > .cargo/config

COPY Readme.md ./
COPY src ./src
COPY templates ./templates
RUN cargo build --release
ENTRYPOINT ["cargo"]

FROM debian:stable-slim as release
COPY --from=build /app/target/release/httpbox /usr/local/bin/
ENTRYPOINT ["/usr/local/bin/httpbox"]
