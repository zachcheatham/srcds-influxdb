FROM rust:bookworm as builder

WORKDIR /usr/src/app

COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt install -y openssl

WORKDIR /
COPY --from=builder /usr/src/app/target/release/srcds-influxdb /usr/local/bin/srcds-influxdb

CMD ["srcds-influxdb"]