"""
Appends grpc-web to the project name of both Cargo.toml and pyproject.toml,
and patches the `hyper-proxy` dependency to allow ppc64le wheel builds.

This is used in CI to prepare grpc-web-specific python artifacts for publishing.
"""

from io import TextIOWrapper
from os.path import dirname, realpath, join
import toml

pycrate_path = dirname(dirname(realpath(__file__)))
workspace_path = dirname(dirname(pycrate_path))


def write(f: TextIOWrapper, data):
    f.seek(0)
    f.write(toml.dumps(data))
    f.truncate()


# Update the package metadata

with open(join(pycrate_path, "pyproject.toml"), "r+") as f:
    data = toml.load(f)
    data["project"]["name"] = "qcs-sdk-python-grpc-web"
    write(f, data)

with open(join(pycrate_path, "Cargo.toml"), "r+") as f:
    data = toml.load(f)
    data["package"]["name"] = "qcs-sdk-python-grpc-web"
    write(f, data)

# Patch the `hyper-proxy` dependency

with open(join(workspace_path, "Cargo.toml"), "r+") as f:
    data = toml.load(f)
    data["patch"] = {"crates-io": {"hyper-proxy": {"git": "https://github.com/rigetti/hyper-proxy"}}}
    write(f, data)
