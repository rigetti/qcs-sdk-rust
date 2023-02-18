/// Spawn and block on a future using the pyo3 tokio runtime.
/// Useful for returning a synchronous `PyResult`.
///
///
/// When used like the following:
/// ```rs
/// async fn say_hello(name: String) -> String {
///     format!("hello {name}")
/// }
///
/// #[pyo3(name="say_hello")]
/// pub fn py_say_hello(name: String) -> PyResult<String> {
///     py_sync!(say_hello(name))
/// }
/// ```
///
/// Becomes the associated "synchronous" python call:
/// ```py
/// assert say_hello("Rigetti") == "hello Rigetti"
/// ```
macro_rules! py_sync {
    ($body: expr) => {{
        let runtime = pyo3_asyncio::tokio::get_runtime();
        let handle = runtime.spawn($body);
        runtime
            .block_on(handle)
            .map_err(|err| PyRuntimeError::new_err(err.to_string()))?
    }};
}

/// Convert a rust future into a Python awaitable using
/// `pyo3_asyncio::tokio::local_future_into_py`
macro_rules! py_async {
    ($py: ident, $body: expr) => {
        pyo3_asyncio::tokio::future_into_py($py, $body)
    };
}

/// Given a single implementation of an async function,
/// create that function as private and two pyfunctions
/// named after it that can be used to invoke either
/// blocking or async variants of the same function.
///
/// This macro cannot be used when lifetime specifiers are
/// required, or the pyfunction bodies need additional
/// parameter handling besides simply calling out to
/// the underlying `py_async` or `py_sync` macros.
///
/// ```rs
/// // ... becomes python package "things"
/// create_init_submodule! {
///     funcs: [
///         py_do_thing,
///         py_do_thing_async,
///     ]
/// }
///
/// py_function_sync_async! {
///     #[args(timeout = "None")]
///     async fn do_thing(timeout: Option<u64>) -> PyResult<String> {
///         // ... sleep for timeout ...
///         Ok(String::from("done"))
///     }
/// }
/// ```
///
/// becomes in python:
/// ```py
/// from things import do_stuff, do_stuff_async
/// assert do_stuff() == "done"
/// assert await do_stuff() == "done"
/// ```
macro_rules! py_function_sync_async {
    (
        $(#[pyfunction$((
            $($k: ident = $v: literal),* $(,)?
        ))?])?
        async fn $name: ident($($arg: ident : $kind: ty),* $(,)?) $(-> $ret: ty)? $body: block
    ) => {
        async fn $name($($arg: $kind,)*) $(-> $ret)? {
            $body
        }

        ::paste::paste! {
        #[::pyo3::pyfunction$($((
            $($k = $v),*
        ))?)?]
        #[pyo3(name = $name "")]
        pub fn [< py_ $name >]($($arg: $kind),*) $(-> $ret)? {
            py_sync!($name($($arg),*))
        }

        #[::pyo3::pyfunction$($((
            $($k = $v),*
        ))?)?]
        #[pyo3(name = $name "_async")]
        pub fn [< py_ $name _async >](py: Python<'_> $(, $arg: $kind)*) -> PyResult<&PyAny> {
            py_async!(py, $name($($arg),*))
        }
        }
    };
}

pub(crate) use py_async;
pub(crate) use py_function_sync_async;
pub(crate) use py_sync;
