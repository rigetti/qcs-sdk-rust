use std::collections::HashMap;
use std::convert::TryFrom;
use std::f64::consts::{FRAC_PI_2, PI};

use serde::{Deserialize, Serialize};

use qcs_api::models::{Characteristic, Node, Operation};

use super::operator::{
    wildcard, Argument, Operator, Parameter, PERFECT_DURATION, PERFECT_FIDELITY,
};

/// Represents a single Qubit on a QPU and its capabilities. Needed by quilc for optimization.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct Qubit {
    id: i32,
    #[serde(skip_serializing_if = "is_false")]
    #[serde(default)]
    dead: bool,
    gates: Vec<Operator>,
}

// Gross hack to not include `dead` if unneeded, to follow pyQuil's implementation
#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_false(val: &bool) -> bool {
    !*val
}

impl Qubit {
    pub(crate) fn from_nodes(nodes: &[Node]) -> HashMap<i32, Qubit> {
        nodes
            .iter()
            .map(|node| (node.node_id, Qubit::from(node)))
            .collect()
    }

    /// Add an operation to a particular Qubit.
    ///
    /// In the QCS ISA, the definition of Qubits is separate from the definition of which operations
    /// those Qubits can perform and what stats (fidelity, timing, etc.) they have. Those operations
    /// are defined later, and must be added to the defined Qubits via this function.
    ///
    /// # Arguments
    /// 1. `op_name`: The name of the operation defined on this Qubit. This comes from
    ///     `instructions[n].name` in the QCS response.
    /// 2. `characteristics`: Data related to the operation at a this site (Qubit). Comes from
    ///     `instructions[n].sites[m].characteristics` in the QCS response.
    /// 3. `benchmarks`: Top level benchmarks on the Qubits, comes from `benchmarks` in the QCS
    ///     response.
    ///
    /// # Errors
    /// 1. `randomized_benchmark_simultaneous_1q` was not present in `benchmarks`: this is necessary
    ///     for RX and RZ gates.
    /// 2. The `randomized_benchmark_simultaneous_1q` benchmark did not contain data for a Qubit
    ///     which has a defined RX or RZ gate.
    /// 3. An unknown `op_name` was provided.
    pub(crate) fn add_operation(
        &mut self,
        op_name: &str,
        characteristics: &[Characteristic],
        frb_sim_1q: &FrbSim1q,
    ) -> Result<(), Error> {
        let operators = match op_name {
            "RX" => rx_gates(self.id, frb_sim_1q)?,
            "RZ" => rz_gates(self.id),
            "MEASURE" => measure(self.id, characteristics),
            "WILDCARD" => vec![wildcard(Some(self.id))],
            "I" | "RESET" => vec![],
            unknown => return Err(Error::UnknownOperator(String::from(unknown))),
        };
        self.gates.extend(operators);
        self.dead = false;
        Ok(())
    }
}

/// All the errors that can occur within this module.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Required benchmark `randomized_benchmark_simultaneous_1q` was not present")]
    MissingBenchmark,
    #[error("Benchmark `randomized_benchmark_simultaneous_1q` must have a single call site")]
    InvalidBenchmark,
    #[error("Benchmark missing for qubit {0}")]
    MissingBenchmarkForQubit(i32),
    #[error("Unknown operator {0}")]
    UnknownOperator(String),
}

impl From<&Node> for Qubit {
    fn from(node: &Node) -> Self {
        Self {
            id: node.node_id,
            dead: true,
            gates: vec![],
        }
    }
}

#[cfg(test)]
mod describe_qubit {
    use crate::qpu::quilc::isa::qubit::Qubit;

    #[test]
    fn it_skips_serializing_dead_if_false() {
        let undead_qubit = Qubit {
            id: 0,
            dead: false,
            gates: vec![],
        };
        let dead_qubit = Qubit {
            id: 0,
            dead: true,
            gates: vec![],
        };

        let expected_dead = serde_json::json!({
            "id": 0,
            "dead": true,
            "gates": []
        });
        let expected_undead = serde_json::json!({
            "id": 0,
            "gates": []
        });

        let undead = serde_json::to_value(undead_qubit).unwrap();
        let dead = serde_json::to_value(dead_qubit).unwrap();

        assert_eq!(undead, expected_undead);
        assert_eq!(dead, expected_dead);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FrbSim1q(Vec<Characteristic>);

impl TryFrom<Vec<Operation>> for FrbSim1q {
    type Error = Error;

    fn try_from(ops: Vec<Operation>) -> Result<FrbSim1q, Error> {
        const BENCH_NAME: &str = "randomized_benchmark_simultaneous_1q";
        let mut operation = ops
            .into_iter()
            .find(|op| op.name == BENCH_NAME)
            .ok_or(Error::MissingBenchmark)?;
        if operation.sites.len() != 1 {
            return Err(Error::InvalidBenchmark);
        }
        let site = operation.sites.remove(0);

        Ok(Self(site.characteristics))
    }
}

impl FrbSim1q {
    fn fidelity_for_qubit(&self, qubit: i32) -> Result<f64, Error> {
        self.0
            .iter()
            .find(|characteristic| {
                characteristic.node_ids.as_ref().map_or(false, |node_ids| {
                    node_ids.len() == 1 && node_ids[0] == qubit
                })
            })
            .map(|characteristic| characteristic.value.into())
            .ok_or(Error::MissingBenchmarkForQubit(qubit))
    }
}

const DEFAULT_DURATION_RX: f64 = 50.0;

fn rx_gates(node_id: i32, frb_sim_1q: &FrbSim1q) -> Result<Vec<Operator>, Error> {
    let fidelity = frb_sim_1q.fidelity_for_qubit(node_id)?;

    let mut gates = Vec::with_capacity(5);
    let operator = "RX".to_string();
    gates.push(Operator::Gate {
        operator: operator.clone(),
        parameters: vec![Parameter::Float(0.0)],
        arguments: vec![Argument::Int(node_id)],
        fidelity: 1.0,
        duration: DEFAULT_DURATION_RX,
    });

    gates.extend(
        IntoIterator::into_iter([PI, -PI, FRAC_PI_2, -FRAC_PI_2]).map(|param| Operator::Gate {
            operator: operator.clone(),
            parameters: vec![Parameter::Float(param)],
            arguments: vec![Argument::Int(node_id)],
            fidelity,
            duration: DEFAULT_DURATION_RX,
        }),
    );
    Ok(gates)
}

#[cfg(test)]
mod describe_rx_gates {
    use std::f64::consts::{FRAC_PI_2, PI};

    use qcs_api::models::Characteristic;

    use crate::qpu::quilc::isa::{
        operator::{Argument, Operator, Parameter},
        qubit::{rx_gates, FrbSim1q},
    };

    /// This data is copied from the pyQuil ISA integration test.
    #[test]
    fn it_passes_the_pyquil_aspen_8_test() {
        let node_id = 1;
        let frb_sim_1q = FrbSim1q(vec![
            Characteristic {
                name: "fRB".to_string(),
                value: 0.989_821_55,
                error: Some(0.000_699_235_5),
                node_ids: Some(vec![0]),
                parameter_values: None,
                timestamp: "1970-01-01T00:00:00+00:00".to_string(),
            },
            Characteristic {
                name: "fRB".to_string(),
                value: 0.996_832_6,
                error: Some(0.000_100_896_78),
                node_ids: Some(vec![1]),
                timestamp: "1970-01-01T00:00:00+00:00".to_string(),
                parameter_values: None,
            },
        ]);
        let gates = rx_gates(node_id, &frb_sim_1q).expect("Failed to create RX gates");
        let expected = vec![
            Operator::Gate {
                arguments: vec![Argument::Int(1)],
                duration: 50.0,
                fidelity: 1.0,
                operator: "RX".to_string(),
                parameters: vec![Parameter::Float(0.0)],
            },
            Operator::Gate {
                arguments: vec![Argument::Int(1)],
                duration: 50.0,
                fidelity: 0.996_832_609_176_635_7,
                operator: "RX".to_string(),
                parameters: vec![Parameter::Float(PI)],
            },
            Operator::Gate {
                arguments: vec![Argument::Int(1)],
                duration: 50.0,
                fidelity: 0.996_832_609_176_635_7,
                operator: "RX".to_string(),
                parameters: vec![Parameter::Float(-PI)],
            },
            Operator::Gate {
                arguments: vec![Argument::Int(1)],
                duration: 50.0,
                fidelity: 0.996_832_609_176_635_7,
                operator: "RX".to_string(),
                parameters: vec![Parameter::Float(FRAC_PI_2)],
            },
            Operator::Gate {
                arguments: vec![Argument::Int(1)],
                duration: 50.0,
                fidelity: 0.996_832_609_176_635_7,
                operator: "RX".to_string(),
                parameters: vec![Parameter::Float(-FRAC_PI_2)],
            },
        ];
        assert_eq!(gates, expected);
    }
}

fn rz_gates(node_id: i32) -> Vec<Operator> {
    vec![Operator::Gate {
        operator: "RZ".to_string(),
        parameters: vec![Parameter::String("_".to_owned())],
        fidelity: PERFECT_FIDELITY,
        duration: PERFECT_DURATION,
        arguments: vec![Argument::Int(node_id)],
    }]
}

#[cfg(test)]
mod describe_rz_gates {
    use super::{rz_gates, Argument, Operator, Parameter};

    /// This data is copied from the pyQuil ISA integration test.
    #[test]
    fn it_passes_the_pyquil_aspen_8_test() {
        let node_id = 1;
        let gates = rz_gates(node_id);
        let expected = vec![Operator::Gate {
            arguments: vec![Argument::Int(1)],
            duration: 0.01,
            fidelity: 1.0,
            operator: "RZ".to_string(),
            parameters: vec![Parameter::String("_".to_owned())],
        }];
        assert_eq!(gates, expected);
    }
}

const MEASURE_DEFAULT_DURATION: f64 = 2000.0;
const MEASURE_DEFAULT_FIDELITY: f64 = 0.90;

/// Process a "MEASURE" operation.
fn measure(node_id: i32, characteristics: &[Characteristic]) -> Vec<Operator> {
    let fidelity = characteristics
        .iter()
        .find(|characteristic| &characteristic.name == "fRO")
        .map_or(MEASURE_DEFAULT_FIDELITY, |characteristic| {
            characteristic.value.into()
        });

    vec![
        Operator::Measure {
            operator: "MEASURE".to_string(),
            duration: MEASURE_DEFAULT_DURATION,
            fidelity,
            qubit: node_id,
            target: Some("_".to_string()),
        },
        Operator::Measure {
            operator: "MEASURE".to_string(),
            duration: MEASURE_DEFAULT_DURATION,
            fidelity,
            qubit: node_id,
            target: None,
        },
    ]
}

#[cfg(test)]
mod describe_measure {
    use qcs_api::models::Characteristic;

    use crate::qpu::quilc::isa::operator::Operator;

    use super::measure;

    /// This test copies data from pyQuil's integration test for ISA conversion.
    #[test]
    fn it_passes_pyquil_integration() {
        let characteristics = [Characteristic {
            error: None,
            name: String::from("fRO"),
            node_ids: None,
            parameter_values: None,
            timestamp: "1970-01-01T00:00:00+00:00".to_string(),
            value: 0.981,
        }];
        let result = measure(0, &characteristics);
        let expected = vec![
            Operator::Measure {
                operator: "MEASURE".to_string(),
                duration: 2000.0,
                fidelity: 0.981_000_006_198_883_1,
                qubit: 0,
                target: Some("_".to_string()),
            },
            Operator::Measure {
                operator: "MEASURE".to_string(),
                duration: 2000.0,
                fidelity: 0.981_000_006_198_883_1,
                qubit: 0,
                target: None,
            },
        ];
        assert_eq!(result, expected);
    }
}
