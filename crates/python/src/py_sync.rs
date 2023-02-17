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

pub(crate) use py_sync;
