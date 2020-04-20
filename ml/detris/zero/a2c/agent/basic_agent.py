from typing import List
from . import Agent, AgentParams, AgentCore, DoneCallback, LoadModelFunc
from ...batch import MultiBatch


class BasicAgent(Agent):
    core: AgentCore

    def __init__(self, model_loader: LoadModelFunc, batch_size: int, params: AgentParams):
        super(BasicAgent, self).__init__(model_loader, batch_size)
        self.core = AgentCore(params)

    def sync_model(self):
        self.core.set_model(self.load_model())

    def game_strs(self) -> List[str]:
        return [self.core.game_str()]

    def run_steps(self, on_done: DoneCallback) -> MultiBatch:
        multi_batch = MultiBatch.zeros(1, self.batch_size)
        self.core.run_steps(multi_batch.get(0), on_done)
        return multi_batch
