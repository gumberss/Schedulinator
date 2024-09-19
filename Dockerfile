FROM rust:buster as builder

RUN USER=root cargo new --bin app
WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN rm src/*.rs

COPY ./src ./src

RUN cargo build --release

FROM debian:buster-slim

RUN apt-get update && apt-get install -y libssl1.1 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/schedulinator /usr/local/bin/app

CMD ["/usr/local/bin/app"]