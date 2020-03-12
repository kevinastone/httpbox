FROM rust as build

WORKDIR /app

COPY Cargo.lock Cargo.toml ./
COPY gotham_async/Cargo.lock gotham_async/Cargo.toml gotham_async/
COPY gotham_async/src gotham_async/
RUN mkdir .cargo
RUN cargo vendor > .cargo/config

COPY Readme.md src ./
COPY src ./src
COPY templates ./templates
RUN cargo build --release
ENTRYPOINT ["cargo"]

FROM debian:stable-slim as release
COPY --from=build /app/target/release/httpbox /usr/local/bin/
ENTRYPOINT ["/usr/local/bin/httpbox"]
