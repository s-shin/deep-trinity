from typing import List
import tensorflow as tf
from . import Agent, AgentParams, AgentCore, DoneCallback
from ...batch import MultiBatch


class BasicAgent(Agent):
    core: AgentCore

    def __init__(self, model: tf.keras.Model, params: AgentParams):
        self.core = AgentCore(model, params)

    def set_model(self, model: tf.keras.Model):
        self.core.set_model(model)

    def game_strs(self) -> List[str]:
        return [self.core.game_str()]

    def next_state_values(self) -> List[float]:
        return [self.core.next_state_value()]

    def run_steps(self, num_steps: int, on_done: DoneCallback) -> MultiBatch:
        multi_batch = MultiBatch.zeros(1, num_steps)
        self.core.run_steps(multi_batch.get(0), on_done)
        return multi_batch
