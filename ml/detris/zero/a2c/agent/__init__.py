from typing import Callable, NamedTuple, List, Optional
import tensorflow as tf
import numpy as np
from ....environment import Environment
from ...batch import Batch, MultiBatch
from ... import mcts

DoneCallback = Callable[
    [
        int,  # episode_n
        int,  # step_n
        float,  # episode_reward
        str,  # game_str
    ],
    None,
]


class AgentParams(NamedTuple):
    num_sample_actions: int
    mcts_run_params: mcts.RunParams


class AgentCore:
    model: Optional[tf.keras.Model]
    params: AgentParams
    env: Environment
    episode_n: int
    step_n: int
    episode_reward: float

    def __init__(self, params: AgentParams):
        self.model = None
        self.params = params
        self.env = Environment()
        self.episode_n = 1  # 1-indexed
        self.step_n = 1  # 1-indexed
        self.episode_reward = 0

    def set_model(self, model: tf.keras.Model):
        self.model = model

    def game_str(self) -> str:
        return self.env.game_str()

    def run_steps(self, out_batch: Batch, on_done: DoneCallback):
        assert self.model is not None
        for i in range(len(out_batch)):
            observation = self.env.observation

            should_sample_action = self.step_n <= self.params.num_sample_actions
            action, root = mcts.run(self.model, self.env, should_sample_action, self.params.mcts_run_params)
            _, reward, done = self.env.step(action)
            self.episode_reward += reward

            sum_visits = sum(np.array([child.num_visits for child in root.children.values()]))
            action_probs = np.array([
                root.children[a].num_visits / sum_visits if a in root.children else 0
                for a in range(Environment.NUM_ACTIONS)
            ], dtype=np.float32)

            out_batch.set(i, observation, action_probs, action, reward, done)

            if done:
                on_done(self.episode_n, self.step_n, self.episode_reward, self.env.game_str())
                self.episode_n += 1
                self.step_n = 1
                self.episode_reward = 0
                self.env.reset()
                continue

            self.step_n += 1


LoadModelFunc = Callable[[], tf.keras.Model]


class Agent:
    load_model: LoadModelFunc
    batch_size: int

    def __init__(self, load_model: LoadModelFunc, batch_size: int):
        self.load_model = load_model
        self.batch_size = batch_size

    def sync_model(self):
        raise NotImplementedError()

    def run_steps(self, on_done: DoneCallback) -> MultiBatch:
        raise NotImplementedError()

    def exit(self):
        pass
