use std::collections::HashMap;
use std::time::Duration;

use crate::RegisterData;

/// The result of executing an [`Executable`][crate::Executable] via
/// [`Executable::execute_on_qvm`][crate::Executable::execute_on_qvm] or
/// [`Executable::execute_on_qpu`][crate::Executable::execute_on_qpu].
#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionData {
    /// The data of all registers that were read from
    /// (via [`Executable::read_from`][crate::Executable::read_from]). Key is the name of the
    /// register, value is the data of the register after execution.
    pub registers: HashMap<Box<str>, RegisterData>,
    /// The time it took to execute the program on the QPU, not including any network or queueing
    /// time. If paying for on-demand execution, this is the amount you will be billed for.
    ///
    /// This will always be `None` for QVM execution.
    pub duration: Option<Duration>,
}
