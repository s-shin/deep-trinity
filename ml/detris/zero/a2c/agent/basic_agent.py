from . import Agent, AgentParams, AgentCore, DoneCallback
from ...batch import MultiBatch
from ...predictor import Predictor


class BasicAgent(Agent):
    core: AgentCore

    def __init__(self, predictor: Predictor, batch_size: int, params: AgentParams):
        super(BasicAgent, self).__init__(batch_size)
        self.core = AgentCore(predictor, params)

    def sync_model(self):
        self.core.sync_model()

    def run_steps(self, on_done: DoneCallback) -> MultiBatch:
        multi_batch = MultiBatch.zeros(1, self.batch_size)
        self.core.run_steps(multi_batch.get(0), on_done)
        return multi_batch
