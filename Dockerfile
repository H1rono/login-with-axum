# ref: https://marcopolo.io/code/nix-and-small-containers/
FROM nixpkgs/nix-flakes:nixos-23.11 AS builder

WORKDIR /app

ENV NIX_CONFIG='filter-syscalls = false'

COPY . .
RUN nix build .

RUN mkdir /tmp/nix-store-closure
RUN cp -R $(nix-store -qR result/) /tmp/nix-store-closure

FROM debian:bookworm-slim

RUN apt-get -y update \
    && apt-get install -y --no-install-recommends \
    coreutils libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && update-ca-certificates --fresh
WORKDIR /app

COPY --from=builder /tmp/nix-store-closure /nix/store
COPY --from=builder /app/result /app

CMD [ "/app/bin/main" ]
