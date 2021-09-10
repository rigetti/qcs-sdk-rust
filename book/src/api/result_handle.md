# ResultHandle

`ResultHandle` is an opaque struct managed by the SDK which allows you to dynamically read out results with [`get_data`]. You will only ever have a pointer to this struct accessed from within an [`ExecutionResult`] if it is the [`Handle`] variant.

## Definition

```c
{{#include ../../../c-lib/libqcs.h:ResultHandle}}
```

[`get_data`]: get_data.md
[`ExecutionResult`]: execution_result.md
[`Handle`]: execution_result.md#handle
