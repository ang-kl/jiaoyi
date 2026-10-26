# syntax=docker/dockerfile:1
# This is a separate service. It contains no trading binary or production config.
FROM rust:1.94.1-bookworm AS rust
FROM node:22-bookworm-slim AS node
FROM postgres:16-bookworm
RUN apt-get update && apt-get install -y --no-install-recommends \
    python3 git gh build-essential pkg-config libssl-dev ca-certificates util-linux \
    && rm -rf /var/lib/apt/lists/*
COPY --from=rust /usr/local/cargo /opt/cargo
COPY --from=rust /usr/local/rustup /opt/rustup
COPY --from=node /usr/local/bin/node /usr/local/bin/node
COPY --from=node /usr/local/lib/node_modules/npm /usr/local/lib/node_modules/npm
ENV PATH="/opt/cargo/bin:/usr/local/bin:/usr/lib/postgresql/16/bin:${PATH}" \
    RUSTUP_HOME=/opt/rustup CARGO_HOME=/data/cargo ENGINEERING_STATE_DIR=/data \
    ENGINEERING_PREFLIGHT_ON_START=1 ENGINEERING_ISOLATION_MODE=uid-container
# Railway's builder does not support BuildKit secret mounts. This build uses
# Railway's normal CA trust; no managed-session proxy certificate is included.
RUN node /usr/local/lib/node_modules/npm/bin/npm-cli.js install --global @openai/codex@0.163.0 \
    && useradd --uid 10001 --create-home engineering
WORKDIR /app
COPY --chmod=755 . /app/
EXPOSE 8080
ENTRYPOINT []
CMD ["python3", "/app/worker.py", "serve"]
