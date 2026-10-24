# The jiaoyi service. Two stages: build with the pinned toolchain, run on a
# slim base. Migrations are embedded in the binary, so only the binary ships.
FROM rust:1.94.1-slim-bookworm AS build
WORKDIR /src
COPY . .
RUN cargo build --release --locked -p jiaoyi --bin jiaoyi

FROM debian:bookworm-slim
RUN useradd --system --no-create-home jiaoyi
COPY --from=build /src/target/release/jiaoyi /usr/local/bin/jiaoyi
USER jiaoyi
CMD ["/usr/local/bin/jiaoyi"]
