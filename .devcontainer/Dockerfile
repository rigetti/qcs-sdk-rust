FROM rigetti/quilc

# Install GCC 10 to work around the memcmp bug in GCC 9.3
# aws-lc-sys (used by cargo-binstall) rejects GCC 9.x due to this bug:
# https://gcc.gnu.org/bugzilla/show_bug.cgi?id=95189
RUN apt-get update && \
    apt-get install -y --no-install-recommends gcc-10 && \
    update-alternatives --install /usr/bin/gcc gcc /usr/bin/gcc-10 100 && \
    rm -rf /var/lib/apt/lists/*
