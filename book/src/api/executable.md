# `Executable`

An intentionally opaque struct used internally by the SDK to track state and cache various stages of compilation for better performance. Created by [`executable_from_quil`].

## Definition

```C
{{#include ../../../c-lib/libqcs.h:Executable}}
```

## See Also

- [`free_executable`](free_executable.md)
- [`read_from`](read_from.md)
- [`set_param`](set_param.md)
- [`wrap_in_shots`](wrap_in_shots.md)

[`executable_from_quil`]: executable_from_quil.md
