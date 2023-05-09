# See the following documentation for why this file is necessary:
# https://pyo3.rs/v0.18.0/python_typing_hints#__init__py-content
import signal

# def handle(signum, farme):
#     print("caught")
#     raise OSError("Interrupted")

# signal.signal(signal.SIGINT, handle)

from .qcs_sdk import *
