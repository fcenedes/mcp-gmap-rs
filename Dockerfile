# syntax=docker/dockerfile:1
FROM rust:1.94.1-bookworm AS build
ARG TARGETARCH
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY docs/crate.md ./docs/crate.md
COPY docs/production.md ./docs/production.md
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,id=mcp-gmap-target-${TARGETARCH},target=/build/target,sharing=locked \
    cargo build --release --locked \
    && install -D target/release/mcp-gmap-rs /out/mcp-gmap-rs

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10001 mcp \
    && useradd --uid 10001 --gid mcp --no-create-home --home-dir /app mcp
WORKDIR /app
COPY --from=build /out/mcp-gmap-rs /usr/local/bin/mcp-gmap-rs
COPY LICENSE /usr/share/licenses/mcp-gmap-rs/LICENSE
COPY licenses /usr/share/licenses/mcp-gmap-rs/upstream
USER 10001:10001
ENV MCP_HTTP_LISTEN=0.0.0.0:3000
EXPOSE 3000
ENTRYPOINT ["mcp-gmap-rs"]
CMD ["serve", "http"]
