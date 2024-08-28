FROM rust:bookworm AS builder

WORKDIR /srv

ENV CARGO_TARGET_DIR=/target \
    RUSTUP_HOME=/var/cache/rustup \
    CARGO_HOME=/var/cache/cargo
RUN --mount=type=cache,target=/var/cache/rustup,sharing=locked \
    --mount=type=cache,target=/var/cache/rustup,sharing=locked \
    --mount=type=cache,target=/var/cache/cargo,sharing=locked \
    --mount=type=bind,source=.,target=. \
    cargo build --release --locked \
    && cp -r ./public /public

FROM debian:bookworm-slim AS debian-slim

WORKDIR /srv

RUN --mount=type=cache,target=/var/lib/apt/lists,sharing=locked \
    apt-get -qq update \
    && apt-get install -qq --no-install-recommends libssl-dev

COPY --from=builder /public /srv/public
COPY --from=builder /target/release/main /srv/main

CMD [ "/srv/main" ]
