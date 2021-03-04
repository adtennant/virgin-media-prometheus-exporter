FROM --platform=$BUILDPLATFORM rust:latest as builder

COPY . .

RUN cargo build --release --bin virgin-media-prometheus-exporter

FROM debian:buster-slim

COPY config config
COPY --from=builder /target/release/virgin-media-prometheus-exporter virgin-media-prometheus-exporter

ENTRYPOINT ["./virgin-media-prometheus-exporter"]