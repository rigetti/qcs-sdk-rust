import asyncio
import numpy as np
import qcs

QPU_ID = "Aspen-11"

program = """
DECLARE ro BIT
DECLARE theta REAL
RX(theta) 0
MEASURE 0 ro[0]
"""

async def main():
    try:
        native_quil = await qcs.compile(program, QPU_ID)
        translated = await qcs.translate(native_quil, 1, QPU_ID)
        print(translated)
        job_id = await qcs.submit(translated['program'], {'theta': [np.pi]}, QPU_ID)
        print(job_id)
        results = await qcs.retrieve_results(job_id, QPU_ID)
        print(results)
    except Exception as e:
        print(e)


asyncio.run(main())
