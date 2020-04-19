from typing import List
from enum import Enum
import multiprocessing as mp
import tensorflow as tf
from . import Agent, AgentParams, AgentCore, DoneCallback
from ...batch import MultiBatch


class RequestType(Enum):
    EXIT = 0
    RUN_STEPS = 1


def worker(worker_i: int, req_queue: mp.Queue, res_queue: mp.Queue):
    while True:
        req_type = req_queue.get()
        if req_type == RequestType.EXIT:
            res_queue.put((worker_i, req_type))
            break
        elif req_type == RequestType.RUN_STEPS:
            res_queue.put((worker_i, req_type))


class MultiprocessAgent(Agent):
    core: AgentCore

    def __init__(self, model: tf.keras.Model, params: AgentParams):
        self.core = AgentCore(model, params)

    def set_model(self, model: tf.keras.Model):
        raise NotImplementedError()

    def game_strs(self) -> List[str]:
        raise NotImplementedError()

    def next_state_values(self) -> List[float]:
        raise NotImplementedError()

    def run_steps(self, num_steps: int, on_done: DoneCallback) -> MultiBatch:
        multi_batch = MultiBatch.zeros(1, num_steps)
        self.core.run_steps(multi_batch.get(0), on_done)
        return multi_batch
