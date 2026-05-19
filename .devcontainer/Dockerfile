FROM rust:latest

# cmake and pkg-config are needed by Rust build scripts (e.g. aws-lc-sys).
# sudo allows install-libquil to re-exec as root when the container user is not root.
COPY scripts/install-libquil /tmp/install-libquil
RUN apt-get update && \
    apt-get install -y --no-install-recommends cmake pkg-config sudo && \
    /tmp/install-libquil && \
    rm -rf /var/lib/apt/lists/* /tmp/install-libquil
