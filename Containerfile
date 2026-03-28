# --- base: Rust + System-Dependencies + Cargo-Tools ---
FROM docker.io/library/rust:1.94-slim-bookworm AS base

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libasound2-dev \
    libudev-dev \
    libwayland-dev \
    libxkbcommon-dev \
    && rm -rf /var/lib/apt/lists/* \
    && cargo install cargo-auditable cargo-audit cargo-deb

WORKDIR /build

# --- build: Kompilieren + .deb ---
FROM base AS build

# Dependencies zuerst cachen
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && \
    cargo auditable build --profile dist 2>/dev/null || true && \
    rm -rf src

# Source kopieren, bauen + .deb erzeugen
COPY LICENSE ./
COPY src/ src/
RUN rm -f target/dist/klotzkoepfe target/dist/deps/klotzkoepfe-* \
    && cargo audit \
    && cargo auditable build --profile dist \
    && cargo deb --no-build

# --- test: Test-Dependencies ---
FROM build AS test
