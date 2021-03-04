# Workaround for QEmu bug when building for 32bit platforms on a 64bit host
FROM --platform=$BUILDPLATFORM rust:latest as vendor

COPY ./Cargo.toml .
COPY ./Cargo.lock .

RUN mkdir .cargo
RUN cargo vendor > .cargo/config.toml

FROM rust:latest as builder

COPY ./src ./src
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

COPY --from=vendor /.cargo /.cargo
COPY --from=vendor /vendor /vendor

RUN cargo build --release --offline

FROM debian:buster-slim

COPY ./config ./config
COPY --from=builder /target/release/virgin-media-prometheus-exporter virgin-media-prometheus-exporter

ENTRYPOINT ["./virgin-media-prometheus-exporter"]