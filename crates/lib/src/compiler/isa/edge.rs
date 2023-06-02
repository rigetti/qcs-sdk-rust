use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize, Serializer};

use qcs_api_client_openapi::models::{self, Characteristic};

use super::operator::{wildcard, Argument, Operator, Parameter};

/// Represents a connection between two qubits.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct Edge {
    #[serde(rename = "ids")]
    id: Id,
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

impl Edge {
    /// Construct an edge with no gates
    fn empty(id: Id) -> Self {
        Self {
            id,
            dead: true,
            gates: vec![],
        }
    }

    pub(crate) fn add_operation(
        &mut self,
        op_name: &str,
        characteristics: &[Characteristic],
    ) -> Result<(), Error> {
        let operator = match GATE_PARAMS.get_key_value(op_name) {
            Some((key, params)) => basic_gates(key.clone(), params, characteristics),
            _ => {
                if op_name == "WILDCARD" {
                    wildcard(None)
                } else {
                    return Err(Error::UnknownOperator(String::from(op_name)));
                }
            }
        };

        // If an edge has an operation, it's not dead.
        self.dead = false;

        self.gates.push(operator);

        Ok(())
    }
}

/// All the error which can occur from within this module.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unknown operator: {0}")]
    UnknownOperator(String),
    #[error("Edges should have exactly 2 nodes, got {0}")]
    EdgeSize(usize),
}

#[cfg(test)]
mod describe_edge {
    use super::{Edge, Id};

    #[test]
    fn it_skips_serializing_dead_if_false() {
        let undead_qubit = Edge {
            id: Id::new([1, 2]),
            dead: false,
            gates: vec![],
        };
        let dead_qubit = Edge {
            id: Id::new([1, 2]),
            dead: true,
            gates: vec![],
        };

        let expected_dead = serde_json::json!({
            "ids": [1, 2],
            "dead": true,
            "gates": []
        });
        let expected_undead = serde_json::json!({
            "ids": [1, 2],
            "gates": []
        });

        let undead = serde_json::to_value(undead_qubit).unwrap();
        let dead = serde_json::to_value(dead_qubit).unwrap();

        assert_eq!(undead, expected_undead);
        assert_eq!(dead, expected_dead);
    }
}

/// The unique identifier of an [`Edge`] is defined by the sorted combination
/// of the nodes that make it up. When used as a key in the map that will be sent to `quilc`, the
/// key should look like `"{node_id_1}-{node_id_2}"`
///
/// This struct enforces those things to make looking up of Edges easier when converting ISAs.
#[derive(Deserialize, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Id([i64; 2]);

impl Id {
    pub fn new(mut node_ids: [i64; 2]) -> Self {
        node_ids.sort_unstable();
        Self(node_ids)
    }
}

impl TryFrom<&Vec<i64>> for Id {
    type Error = Error;

    fn try_from(node_ids: &Vec<i64>) -> Result<Self, Error> {
        let node_ids: [i64; 2] = node_ids
            .as_slice()
            .try_into()
            .map_err(|_| Error::EdgeSize(node_ids.len()))?;
        Ok(Self::new(node_ids))
    }
}

impl Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_seq(self.0)
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.0[0], self.0[1])
    }
}

#[cfg(test)]
mod describe_edge_id {
    use std::convert::TryFrom;

    use super::Id;

    #[test]
    fn it_serializes_as_an_int_list() {
        let edge_id = Id::new([1, 2]);
        let serialized = serde_json::to_value(edge_id).expect("Could not serialize EdgeId");
        assert_eq!(serialized, serde_json::json!([1, 2]));
    }

    #[test]
    fn it_displays_as_quilc_style_string() {
        let edge_id = Id::new([1, 2]);
        assert_eq!(edge_id.to_string(), "1-2".to_string());
    }

    #[test]
    fn it_sorts_ids_when_constructed() {
        let edge_id = Id::new([2, 1]);
        assert_eq!(edge_id.to_string(), "1-2".to_string());
    }

    #[test]
    fn it_fails_to_convert_from_vec_when_wrong_size() {
        let inputs = vec![
            // No nodes
            vec![],
            // Only 1 node
            vec![1],
            // Too many nodes
            vec![1, 2, 3],
        ];
        for input in inputs {
            let result = Id::try_from(&input);
            assert!(result.is_err());
        }
    }

    #[test]
    fn it_successfully_converts_from_correct_vec() {
        let input = vec![2, 1];
        let result = Id::try_from(&input).expect("Failed to convert valid Vec to EdgeId");
        assert_eq!(result.to_string(), "1-2".to_string());
    }
}

/// Convert the QCS ISA representation of edges to the quilc form [`Edge`]
pub(crate) fn convert_edges(edges: &[models::Edge]) -> Result<HashMap<Id, Edge>, Error> {
    edges
        .iter()
        .map(|edge| {
            let id = Id::try_from(&edge.node_ids)?;
            let edge = Edge::empty(id);
            Ok((id, edge))
        })
        .collect()
}

#[cfg(test)]
mod describe_convert_edges {
    use qcs_api_client_openapi::models;

    use super::{convert_edges, Id};

    #[test]
    fn it_converts_valid_edges() {
        let input = vec![
            models::Edge {
                node_ids: vec![1, 2],
            },
            models::Edge {
                node_ids: vec![2, 3],
            },
            models::Edge {
                node_ids: vec![3, 1],
            },
        ];

        let result = convert_edges(&input).expect("Could not convert valid inputs");

        assert_eq!(result.len(), 3);
        let expected_ids = [Id::new([1, 2]), Id::new([2, 3]), Id::new([1, 3])];
        for expected in expected_ids {
            assert_eq!(result[&expected].id, expected);
        }
    }
}

lazy_static::lazy_static! {
    static ref GATE_PARAMS: HashMap<String, GateParams> = {
        let mut m = HashMap::new();
        m.insert("CZ".to_string(), GateParams{
            default_fidelity: 0.89,
            duration: 200.0,
            characteristic_name: "fCZ".to_string(),
            parameters: vec![],
        });
        m.insert("ISWAP".to_string(), GateParams{
            default_fidelity: 0.90,
            duration: 200.0,
            characteristic_name: "fISWAP".to_string(),
            parameters: vec![],
        });
        m.insert("CPHASE".to_string(), GateParams{
            default_fidelity: 0.85,
            duration: 200.0,
            characteristic_name: "fCPHASE".to_string(),
            parameters: vec![Parameter::String("theta".to_owned())],
        });
        m.insert("XY".to_string(), GateParams{
            default_fidelity: 0.86,
            duration: 200.0,
            characteristic_name: "fXY".to_string(),
            parameters: vec![Parameter::String("theta".to_owned())],
        });
        m
    };
}

/// Contains everything you need to know to parse a gate on an edge for a particular op
struct GateParams {
    default_fidelity: f64,
    duration: f64,
    characteristic_name: String,
    parameters: Vec<Parameter>,
}

fn basic_gates(
    op_name: String,
    params: &GateParams,
    characteristics: &[Characteristic],
) -> Operator {
    let GateParams {
        default_fidelity,
        duration,
        characteristic_name,
        parameters,
    } = params;
    let fidelity = characteristics
        .iter()
        .find(|characteristic| &characteristic.name == characteristic_name)
        .map_or(*default_fidelity, |characteristic| characteristic.value);

    Operator::Gate {
        operator: op_name,
        parameters: parameters.clone(),
        arguments: vec![Argument::String("_".to_string()); 2],
        fidelity,
        duration: *duration,
    }
}
