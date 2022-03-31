use std::collections::HashMap;
use std::f64::consts::{FRAC_PI_2, PI};

use eyre::{eyre, Result, WrapErr};
use serde::Serialize;

use qcs_api::models::{Characteristic, Node, Operation};

use super::operator::{Arguments, OperatorMap, Parameters, PERFECT_DURATION, PERFECT_FIDELITY};
use super::Operator;

/// Represents a single Qubit on a QPU and its capabilities. Needed by quilc for optimization.
#[derive(Serialize, Debug)]
pub(crate) struct Qubit {
    id: i32,
    #[serde(skip_serializing_if = "is_false")]
    dead: bool,
    gates: OperatorMap,
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
        benchmarks: &[Operation],
    ) -> Result<()> {
        let benchmarks = Benchmarks::try_from(benchmarks)?;
        let operators = match op_name {
            "RX" => rx_gates(self.id, benchmarks.frb_sim_1q)?,
            "RZ" => rz_gates(self.id),
            "MEASURE" => measure(self.id, characteristics),
            "WILDCARD" => wildcard(self.id),
            "I" | "RESET" => vec![],
            unknown => return Err(eyre!("Unknown operator {}", unknown)),
        };
        if self.gates.add(&operators) {
            self.dead = false;
        }
        Ok(())
    }
}

impl From<&Node> for Qubit {
    fn from(node: &Node) -> Self {
        Self {
            id: node.node_id,
            dead: true,
            gates: OperatorMap::new(),
        }
    }
}

#[cfg(test)]
mod describe_qubit {
    use super::*;

    #[test]
    fn it_skips_serializing_dead_if_false() {
        let undead_qubit = Qubit {
            id: 0,
            dead: false,
            gates: OperatorMap::new(),
        };
        let dead_qubit = Qubit {
            id: 0,
            dead: true,
            gates: OperatorMap::new(),
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

/// Contains the post-filtered, categorized benchmarks for later usage
struct Benchmarks<'a> {
    frb_sim_1q: &'a Operation,
}

impl<'a> Benchmarks<'a> {
    fn try_from(ops: &'a [Operation]) -> Result<Benchmarks<'a>> {
        const BENCH_NAME: &str = "randomized_benchmark_simultaneous_1q";
        let frb_sim_1q = ops.iter().find(|op| op.name == BENCH_NAME).ok_or_else(|| {
            eyre!(
                "Parsing ISA requires a benchmark called '{}' which is missing",
                BENCH_NAME
            )
        })?;
        Ok(Self { frb_sim_1q })
    }
}

const DEFAULT_DURATION_RX: f64 = 50.0;

fn rx_gates(node_id: i32, frb_sim_1q: &Operation) -> Result<Vec<Operator>> {
    let fidelity = fidelity(frb_sim_1q, node_id)
        .wrap_err_with(|| format!("While adding RX gate to Qubit {}", node_id))?;

    let mut gates = Vec::with_capacity(5);
    let operator = "RX";
    gates.push(Operator::Gate {
        operator,
        parameters: Parameters::Float(0.0),
        arguments: Arguments::Int(node_id),
        fidelity: 1.0,
        duration: DEFAULT_DURATION_RX,
    });

    gates.extend(
        IntoIterator::into_iter([PI, -PI, FRAC_PI_2, -FRAC_PI_2]).map(|param| Operator::Gate {
            operator,
            parameters: Parameters::Float(param),
            arguments: Arguments::Int(node_id),
            fidelity,
            duration: DEFAULT_DURATION_RX,
        }),
    );
    Ok(gates)
}

#[cfg(test)]
mod describe_rx_gates {
    use qcs_api::models::OperationSite;

    use super::*;

    /// This data is copied from the pyQuil ISA integration test.
    #[test]
    fn it_passes_the_pyquil_aspen_8_test() {
        let node_id = 1;
        let frb_sim_1q = Operation {
            characteristics: vec![],
            name: "randomized_benchmark_simultaneous_1q".to_string(),
            node_count: Some(30),
            parameters: vec![],
            sites: vec![OperationSite {
                characteristics: vec![
                    Characteristic {
                        name: "fRB".to_string(),
                        value: 0.989821537688075,
                        error: Some(0.000699235456806402),
                        node_ids: Some(vec![0]),
                        parameter_values: None,
                        timestamp: "1970-01-01T00:00:00+00:00".to_string(),
                    },
                    Characteristic {
                        name: "fRB".to_string(),
                        value: 0.996832638579018,
                        error: Some(0.00010089678215399),
                        node_ids: Some(vec![1]),
                        timestamp: "1970-01-01T00:00:00+00:00".to_string(),
                        parameter_values: None,
                    },
                ],
                node_ids: vec![0, 1],
            }],
        };
        let gates = rx_gates(node_id, &frb_sim_1q).expect("Failed to create RX gates");
        let expected = vec![
            Operator::Gate {
                arguments: Arguments::Int(1),
                duration: 50.0,
                fidelity: 1.0,
                operator: "RX",
                parameters: Parameters::Float(0.0),
            },
            Operator::Gate {
                arguments: Arguments::Int(1),
                duration: 50.0,
                fidelity: 0.9968326091766357,
                operator: "RX",
                parameters: Parameters::Float(PI),
            },
            Operator::Gate {
                arguments: Arguments::Int(1),
                duration: 50.0,
                fidelity: 0.9968326091766357,
                operator: "RX",
                parameters: Parameters::Float(-PI),
            },
            Operator::Gate {
                arguments: Arguments::Int(1),
                duration: 50.0,
                fidelity: 0.9968326091766357,
                operator: "RX",
                parameters: Parameters::Float(FRAC_PI_2),
            },
            Operator::Gate {
                arguments: Arguments::Int(1),
                duration: 50.0,
                fidelity: 0.9968326091766357,
                operator: "RX",
                parameters: Parameters::Float(-FRAC_PI_2),
            },
        ];
        assert_eq!(gates, expected);
    }
}

fn rz_gates(node_id: i32) -> Vec<Operator> {
    vec![Operator::Gate {
        operator: "RZ",
        parameters: Parameters::Underscore,
        fidelity: PERFECT_FIDELITY,
        duration: PERFECT_DURATION,
        arguments: Arguments::Int(node_id),
    }]
}

#[cfg(test)]
mod describe_rz_gates {
    use qcs_api::models::OperationSite;

    use super::*;

    /// This data is copied from the pyQuil ISA integration test.
    #[test]
    fn it_passes_the_pyquil_aspen_8_test() {
        let node_id = 1;
        let frb_sim_1q = Operation {
            characteristics: vec![],
            name: "randomized_benchmark_simultaneous_1q".to_string(),
            node_count: Some(30),
            parameters: vec![],
            sites: vec![OperationSite {
                characteristics: vec![
                    Characteristic {
                        name: "fRB".to_string(),
                        value: 0.989821537688075,
                        error: Some(0.000699235456806402),
                        node_ids: Some(vec![0]),
                        parameter_values: None,
                        timestamp: "1970-01-01T00:00:00+00:00".to_string(),
                    },
                    Characteristic {
                        name: "fRB".to_string(),
                        value: 0.996832638579018,
                        error: Some(0.00010089678215399),
                        node_ids: Some(vec![1]),
                        timestamp: "1970-01-01T00:00:00+00:00".to_string(),
                        parameter_values: None,
                    },
                ],
                node_ids: vec![0, 1],
            }],
        };
        let gates = rz_gates(node_id, &frb_sim_1q).expect("Failed to create RZ gates");
        let expected = vec![Operator::Gate {
            arguments: Arguments::Int(1),
            duration: 0.01,
            fidelity: 0.9968326091766357,
            operator: "RZ",
            parameters: Parameters::Underscore,
        }];
        assert_eq!(gates, expected);
    }
}

fn fidelity(frb_sim_1q: &Operation, node_id: i32) -> Result<f64> {
    let site = frb_sim_1q
        .sites
        .get(0)
        .ok_or_else(|| eyre!("frb_sim_1q benchmark should have exactly 1 site, it has none."))?;
    site.characteristics
        .iter()
        .find(|characteristic| {
            characteristic.node_ids.as_ref().map_or(false, |node_ids| {
                node_ids.len() == 1 && node_ids[0] == node_id
            })
        })
        .map(|characteristic| characteristic.value.into())
        .ok_or_else(|| eyre!("No frb_sim_1q benchmark for qubit {}", node_id))
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
            operator: "MEASURE",
            duration: MEASURE_DEFAULT_DURATION,
            fidelity,
            qubit: node_id,
            target: Some("_"),
        },
        Operator::Measure {
            operator: "MEASURE",
            duration: MEASURE_DEFAULT_DURATION,
            fidelity,
            qubit: node_id,
            target: None,
        },
    ]
}

#[cfg(test)]
mod describe_measure {
    use super::*;

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
                operator: "MEASURE",
                duration: 2000.0,
                fidelity: 0.9810000061988831,
                qubit: 0,
                target: Some("_"),
            },
            Operator::Measure {
                operator: "MEASURE",
                duration: 2000.0,
                fidelity: 0.9810000061988831,
                qubit: 0,
                target: None,
            },
        ];
        assert_eq!(result, expected)
    }
}

fn wildcard(node_id: i32) -> Vec<Operator> {
    vec![Operator::Gate {
        operator: "_",
        duration: PERFECT_DURATION,
        fidelity: PERFECT_FIDELITY,
        parameters: Parameters::Underscore,
        arguments: Arguments::Int(node_id),
    }]
}
