# Installation

## C Library

### Download Artifacts

> TODO: talk about where to download artifacts from

In order to use the C SDK, you'll need a few things:

1. The dynamic library for your platform
1. The `libqcs.h` header file

### Build from Source

Checkout the [GitHub Repo README] for instructions.

## Services

Full usage of this library requires [quilc] and [qvm] to be available on local webservers. The easiest way to do this is by using this [docker-compose file] and running `docker-compose up -d`. This will run the required services on their default ports in the background.

[GitHub Repo README]: https://github.com/rigetti/qcs-sdk-rust/blob/main/c-lib/README.md
[quilc]: https://github.com/quil-lang/quilc
[qvm]: https://github.com/quil-lang/qvm
[docker-compose file]: https://github.com/rigetti/qcs-sdk-rust/blob/main/qcs/docker-compose.yml
