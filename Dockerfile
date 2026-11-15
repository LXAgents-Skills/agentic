# syntax=docker/dockerfile:1

# The payload is one Rust binary with `content/` compiled into it. Two stages: the first
# builds with the Rust toolchain, the second carries only the binary, so the compiler, the
# sources and the crate cache never reach the image.
FROM rust:1-bookworm AS build
WORKDIR /build

# Dependencies are resolved from the committed lockfile only. `--locked` fails rather than
# silently resolving something `Cargo.lock` does not contain, which is the point of building
# an image from a versioned tree.
COPY Cargo.toml Cargo.lock build.rs ./
COPY src ./src
COPY content ./content
RUN cargo build --release --locked --bin lxagents-agentic

FROM debian:bookworm-slim

# Not root. The process answers JSON-RPC from memory, and nothing else.
RUN useradd --system --uid 10001 --no-create-home --shell /usr/sbin/nologin mcp
COPY --from=build /build/target/release/lxagents-agentic /usr/local/bin/lxagents-agentic
USER mcp

# 3000 is the server's default, and `PORT` overrides it at runtime. The interface is
# `HOST`, which also defaults to 0.0.0.0 - every IPv4 interface.
#
# EXPOSE documents the port; it does not publish it. `docker run -p 3000:3000 …` is what
# makes the port reachable from outside the container, and forgetting it produces a container
# that is running, healthy, and connectable from nowhere. See wiki/environments/docker.md.
EXPOSE 3000

# HTTP is the default here, so a container that is run rather than spawned is reachable:
#
#   docker run --rm -p 3000:3000 -e MCP_AUTH_TOKEN lxagents-agentic
#
# HTTP refuses to start without MCP_AUTH_TOKEN (at least 32 characters) off loopback: the
# container exits 1 with a line saying why, and does not run open. The token is passed at run
# time and is deliberately not an ENV line here, because a value baked into the image ships
# to everyone who pulls it. stdio never reads it.
#
# stdio is one environment variable away:
#
#   docker run --rm -i -e MCP_TRANSPORT=stdio lxagents-agentic
#
# For stdio, `-i` is not optional. Without it Docker does not attach stdin, the server sees
# closed input, and it exits at once - which reads as a broken image rather than a missing flag.
#
# Set MCP_ALLOWED_HOSTS whenever the container is reachable from anywhere but this machine.
# The Host allow-list is off unless it is set, and the server says so on startup. See
# wiki/environments/env.md.
ENV MCP_TRANSPORT=http
ENTRYPOINT ["/usr/local/bin/lxagents-agentic"]
