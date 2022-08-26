import numpy as np
from pyquil.api import get_qc
from pyquil.quil import Program
from pyquil.gates import MEASURE, RX

import qcs
import asyncio

QPU_ID = "Aspen-11"

program = Program()
ro = program.declare('ro', 'BIT', 1)
theta = program.declare('theta', 'REAL')
program += RX(theta, 0)
program += MEASURE(0, ro[0])

qc = get_qc(QPU_ID)


async def main():
    try:
        native_quil = await qcs.compile(str(program), QPU_ID)
        qcs_program = await qcs.translate(native_quil, 1, QPU_ID)
        job_id = await qcs.submit(qcs_program, {'theta': [np.pi]}, QPU_ID)
        print(job_id)
        results = await qcs.retrieve_results(job_id, QPU_ID)
        print(results)
    except Exception as e:
        print(e)


asyncio.run(main())
