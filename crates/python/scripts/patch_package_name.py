"""
Appends grpc-web to the project name of both Cargo.toml and pyproject.toml.

This is used in CI to update the package metadata before publishing the alternate
package with the grpc-web feature enabled.
"""
import toml

with open("pyproject.toml", "r+") as f:
    data = toml.load(f)
    data["project"]["name"] = "qcs-sdk-python-grpc-web"
    f.seek(0)
    f.write(toml.dumps(data))
    f.truncate()

with open("Cargo.toml", "r+") as f:
    data = toml.load(f)
    data["package"]["name"] = "qcs-sdk-python-grpc-web"
    f.seek(0)
    f.write(toml.dumps(data))
    f.truncate()
