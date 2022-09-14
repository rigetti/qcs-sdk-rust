import os
import asyncio
import numpy as np
import qcs

QPID = os.getenv("QPID", "Aspen-11")

program = """
DECLARE ro BIT
DECLARE theta REAL
RX(theta) 0
MEASURE 0 ro[0]
"""

async def main():
    print(f"Executing program on {QPID}.")
    try:
        native_quil = await qcs.compile(program, QPID)
        translated = await qcs.translate(native_quil, 1, QPID)
        print(translated)
        job_id = await qcs.submit(translated['program'], {'theta': [np.pi]}, QPID)
        print(job_id)
        results = await qcs.retrieve_results(job_id, QPID)
        print(results)
    except Exception as e:
        print(e)


asyncio.run(main())
