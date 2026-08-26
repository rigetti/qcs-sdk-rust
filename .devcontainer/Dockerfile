FROM rust:latest

# cmake and pkg-config are needed by Rust build scripts (e.g. aws-lc-sys). libclang is
# for bindgen, which generates libquil-sys's bindings; unzip and curl are what libquil's
# installer needs to fetch and unpack a release.
# TODO(github.com/rigetti/libquil#58): drop the pin once the sbcl-librarian runtime work
# is released as a stable version, and install the latest release instead.
ARG LIBQUIL_VERSION=0.4.0
RUN apt-get update && \
    apt-get install -y --no-install-recommends cmake pkg-config unzip curl libclang-dev && \
    curl -fsSL "https://raw.githubusercontent.com/rigetti/libquil/v${LIBQUIL_VERSION}/scripts/install.sh" \
        | bash -s -- --install-deps "${LIBQUIL_VERSION}" && \
    rm -rf /var/lib/apt/lists/*

RUN cargo install cargo-binstall
RUN cargo binstall --no-confirm cargo-make cargo-deny ripgrep knope

# The user is assumed to own the repo, even though the container is run with a different user.
RUN git config --global --add safe.directory /workspaces/qcs-sdk-rust