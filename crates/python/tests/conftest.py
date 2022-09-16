import pytest


@pytest.fixture
def device_2q() -> str:
    import json

    return json.dumps(
        {
            "isa": {
                "1Q": {
                    "0": {
                        "id": 0,
                        "gates": [
                            {
                                "operator": "RX",
                                "duration": 50.0,
                                "fidelity": 1.0,
                                "parameters": [0.0],
                                "arguments": [0],
                                "operator_type": "gate",
                            },
                            {
                                "operator": "RX",
                                "duration": 50.0,
                                "fidelity": 0.9909074679565163,
                                "parameters": [3.141592653589793],
                                "arguments": [0],
                                "operator_type": "gate",
                            },
                            {
                                "operator": "RX",
                                "duration": 50.0,
                                "fidelity": 0.9909074679565163,
                                "parameters": [-3.141592653589793],
                                "arguments": [0],
                                "operator_type": "gate",
                            },
                            {
                                "operator": "RX",
                                "duration": 50.0,
                                "fidelity": 0.9909074679565163,
                                "parameters": [1.5707963267948966],
                                "arguments": [0],
                                "operator_type": "gate",
                            },
                            {
                                "operator": "RX",
                                "duration": 50.0,
                                "fidelity": 0.9909074679565163,
                                "parameters": [-1.5707963267948966],
                                "arguments": [0],
                                "operator_type": "gate",
                            },
                            {
                                "operator": "RZ",
                                "duration": 0.01,
                                "fidelity": 1.0,
                                "parameters": ["_"],
                                "arguments": [0],
                                "operator_type": "gate",
                            },
                            {
                                "operator": "MEASURE",
                                "duration": 2000.0,
                                "fidelity": 0.977,
                                "qubit": 0,
                                "target": "_",
                                "operator_type": "measure",
                            },
                            {
                                "operator": "MEASURE",
                                "duration": 2000.0,
                                "fidelity": 0.977,
                                "qubit": 0,
                                "target": None,
                                "operator_type": "measure",
                            },
                        ],
                    },
                    "1": {
                        "id": 1,
                        "gates": [
                            {
                                "operator": "RX",
                                "duration": 50.0,
                                "fidelity": 1.0,
                                "parameters": [0.0],
                                "arguments": [1],
                                "operator_type": "gate",
                            },
                            {
                                "operator": "RX",
                                "duration": 50.0,
                                "fidelity": 0.9967034552975036,
                                "parameters": [3.141592653589793],
                                "arguments": [1],
                                "operator_type": "gate",
                            },
                            {
                                "operator": "RX",
                                "duration": 50.0,
                                "fidelity": 0.9967034552975036,
                                "parameters": [-3.141592653589793],
                                "arguments": [1],
                                "operator_type": "gate",
                            },
                            {
                                "operator": "RX",
                                "duration": 50.0,
                                "fidelity": 0.9967034552975036,
                                "parameters": [1.5707963267948966],
                                "arguments": [1],
                                "operator_type": "gate",
                            },
                            {
                                "operator": "RX",
                                "duration": 50.0,
                                "fidelity": 0.9967034552975036,
                                "parameters": [-1.5707963267948966],
                                "arguments": [1],
                                "operator_type": "gate",
                            },
                            {
                                "operator": "RZ",
                                "duration": 0.01,
                                "fidelity": 1.0,
                                "parameters": ["_"],
                                "arguments": [1],
                                "operator_type": "gate",
                            },
                            {
                                "operator": "MEASURE",
                                "duration": 2000.0,
                                "fidelity": 0.9450000000000001,
                                "qubit": 1,
                                "target": "_",
                                "operator_type": "measure",
                            },
                            {
                                "operator": "MEASURE",
                                "duration": 2000.0,
                                "fidelity": 0.9450000000000001,
                                "qubit": 1,
                                "target": None,
                                "operator_type": "measure",
                            },
                        ],
                    },
                },
                "2Q": {
                    "0-1": {
                        "ids": [0, 1],
                        "gates": [
                            {
                                "operator": "CZ",
                                "duration": 200.0,
                                "fidelity": 0.95,
                                "parameters": [],
                                "arguments": ["_", "_"],
                                "operator_type": "gate",
                            },
                        ],
                    },
                },
            },
            "specs": {},
        }
    )


@pytest.fixture
def native_bitflip_program() -> str:
    return """
DECLARE ro BIT[0]
RX(pi) 0
MEASURE 0 ro[0]
"""


@pytest.fixture
def bell_program() -> str:
    return """
DECLARE ro BIT[2]
X 0
CNOT 0 1
MEASURE 0 ro[0]
MEASURE 1 ro[1]
"""
