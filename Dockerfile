FROM rust AS build

WORKDIR /app

COPY Cargo.lock Cargo.toml ./
COPY uri_path/Cargo.toml ./uri_path/Cargo.toml
COPY uri_path/src ./uri_path/src
COPY hyper_body/Cargo.toml ./hyper_body/Cargo.toml
COPY hyper_body/src ./hyper_body/src
RUN mkdir .cargo
RUN cargo vendor > .cargo/config

COPY Readme.md ./
COPY src ./src
COPY templates ./templates
RUN cargo build --release
ENTRYPOINT ["cargo"]

FROM debian:stable-slim AS release
ENV PORT=80
COPY --from=build /app/target/release/httpbox /usr/local/bin/
ENTRYPOINT ["/usr/local/bin/httpbox"]
