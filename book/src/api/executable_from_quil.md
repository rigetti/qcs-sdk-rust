# `executable_from_quil`

Create an [`Executable`] from a string containing a [Quil] program. Be sure to free this [`Executable`] using [`free_executable`] once all executions are complete.

## Definition

```c
{{#include ../../../c-lib/libqcs.h:executable_from_quil}}
```

[`Executable`]: executable.md
[Quil]: https://github.com/quil-lang/quil
[`free_executable`]: free_executable.md
