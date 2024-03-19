"""
Patch the `hyper-proxy` dependency, 

This is used in CI to update the package metadata before publishing the alternate
package with the grpc-web feature enabled.
"""
import toml

with open("Cargo.toml", "r+") as f:
    data = toml.load(f)
    data["patch"] = {
        "crates-io": {
            "hyper-proxy": {
                "git": "https://github.com/hoodie/hyper-proxy",
                "rev": "49325f3"
            }
        }
    }
    f.seek(0)
    f.write(toml.dumps(data))
    f.truncate()
