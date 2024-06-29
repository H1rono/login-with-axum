FROM rust:bookworm AS builder

WORKDIR /app

COPY . .
RUN cargo build

FROM debian:bookworm-slim

RUN apt-get -y update \
    && apt-get install -y --no-install-recommends \
    coreutils libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && update-ca-certificates --fresh
WORKDIR /app

COPY --from=builder /app/public /app/public
COPY --from=builder /app/target/debug/main /app/main

CMD [ "/app/main" ]
