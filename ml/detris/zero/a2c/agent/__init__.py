from typing import Callable, NamedTuple, List
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
    ],
    None,
]


class AgentParams(NamedTuple):
    num_sample_actions: int
    mcts_run_params: mcts.RunParams


class AgentCore:
    model: tf.keras.Model
    params: AgentParams
    env: Environment
    episode_n: int
    step_n: int
    episode_reward: float

    def __init__(self, model: tf.keras.Model, params: AgentParams):
        self.model = model
        self.params = params
        self.env = Environment()
        self.episode_n = 1  # 1-indexed
        self.step_n = 1  # 1-indexed
        self.episode_reward = 0

    def set_model(self, model: tf.keras.Model):
        self.model = model

    def game_str(self) -> str:
        return self.env.game_str()

    def next_state_value(self) -> float:
        if self.env.done:
            return 0
        x = tf.convert_to_tensor(self.env.observation[None, :])
        _, state_value_batch = self.model.predict_on_batch(x)
        return float(state_value_batch[0][0])

    def run_steps(self, out_batch: Batch, on_done: DoneCallback):
        for i in range(len(out_batch)):
            observation = self.env.observation

            should_sample_action = self.step_n <= self.params.num_sample_actions
            action, root = mcts.run(self.model, self.env, should_sample_action, self.params.mcts_run_params)
            _, reward, done = self.env.step(action)
            self.episode_reward += reward

            sum_visits = sum(np.array([child.num_visits for child in root.children.values()]))
            action_probs = np.array([
                root.children[a].num_visits / sum_visits if a in root.children else 0
                for a in range(Environment.num_actions)
            ], dtype=np.float)

            out_batch.set(i, observation, action_probs, action, reward, done)

            if done:
                on_done(self.episode_n, self.step_n, self.episode_reward)
                self.episode_n += 1
                self.step_n = 1
                self.env.reset()
                continue

            self.step_n += 1


class Agent:
    def set_model(self, model: tf.keras.Model):
        raise NotImplementeyydError()

    def game_strs(self) -> List[str]:
        raise NotImplementedError()

    def next_state_values(self) -> List[float]:
        raise NotImplementedError()

    def run_steps(self, num_steps: int, on_done: DoneCallback) -> MultiBatch:
        raise NotImplementedError()
