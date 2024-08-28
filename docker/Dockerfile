FROM --platform=$BUILDPLATFORM python:3.12-slim-bookworm AS generate-helper

WORKDIR /srv

ARG TARGETOS
ARG TARGETARCH
ARG TARGETVARIANT

RUN --mount=type=bind,source=docker,target=. \
    python3 generate-helper.py \
    --os "${TARGETOS}" --arch "${TARGETARCH}" --variant "${TARGETVARIANT}" \
    > /helper.sh

CMD [ "cat", "/helper.sh" ]

FROM --platform=$BUILDPLATFORM rust:bookworm AS builder

WORKDIR /srv

COPY --from=generate-helper /helper.sh /helper.sh

RUN --mount=type=cache,target=/var/lib/apt/lists,sharing=locked \
    bash /helper.sh dpkg_add_architecture \
    && bash /helper.sh install_apt_deps

RUN bash /helper.sh print_config > /cargo-config.toml

ENV CARGO_TARGET_DIR=/target \
    RUSTUP_HOME=/var/cache/rustup \
    CARGO_HOME=/var/cache/cargo
RUN --mount=type=cache,target=/var/cache/rustup,sharing=locked \
    --mount=type=cache,target=/var/cache/rustup,sharing=locked \
    --mount=type=cache,target=/var/cache/cargo,sharing=locked \
    --mount=type=bind,source=.,target=. \
    bash /helper.sh add_target \
    && cargo build --release --locked --config /cargo-config.toml \
    && bash /helper.sh extract_binary "${CARGO_TARGET_DIR}" > /target/main \
    && chmod +x /target/main \
    && cp -r ./public /public

FROM debian:bookworm-slim AS debian-slim

WORKDIR /srv

RUN --mount=type=cache,target=/var/lib/apt/lists,sharing=locked \
    apt-get -qq update \
    && apt-get install -qq --no-install-recommends libssl-dev

COPY --from=builder /public /srv/public
COPY --from=builder /target/main /srv/main

CMD [ "/srv/main" ]
