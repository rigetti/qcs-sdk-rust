---
lib: minor
---

#### Build against libquil-sys 0.5, which requires a libquil built on modern sbcl-librarian

The `libquil` feature now needs a libquil that installs the sbcl-librarian runtime and
its headers alongside `libquil.h`. Releases up to and including 0.3.2 ship neither; see
the `libquil` section of the README for how to install a version that does.
