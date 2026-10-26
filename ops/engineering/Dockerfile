# syntax=docker/dockerfile:1
# This is a separate service. It contains no trading binary or production config.
FROM rust:1.94.1-bookworm AS rust
FROM node:22-bookworm-slim AS node
FROM postgres:16-bookworm
RUN apt-get update && apt-get install -y --no-install-recommends \
    python3 git gh build-essential pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=rust /usr/local/cargo /opt/cargo
COPY --from=rust /usr/local/rustup /opt/rustup
COPY --from=node /usr/local/bin/node /usr/local/bin/node
COPY --from=node /usr/local/lib/node_modules/npm /usr/local/lib/node_modules/npm
ENV PATH="/opt/cargo/bin:/usr/local/bin:/usr/lib/postgresql/16/bin:${PATH}" \
    RUSTUP_HOME=/opt/rustup CARGO_HOME=/data/cargo ENGINEERING_STATE_DIR=/data \
    ENGINEERING_PREFLIGHT_ON_START=1
# The optional certificate is only for the managed build proxy, never baked in.
RUN --mount=type=secret,id=proxy_ca \
    if [ -f /run/secrets/proxy_ca ]; then export NODE_EXTRA_CA_CERTS=/run/secrets/proxy_ca; fi; \
    node /usr/local/lib/node_modules/npm/bin/npm-cli.js install --global @openai/codex@0.163.0 \
    && useradd --uid 10001 --create-home engineering
WORKDIR /app
COPY --chmod=755 ops/engineering/ /app/
EXPOSE 8080
ENTRYPOINT []
CMD ["python3", "/app/worker.py", "serve"]
