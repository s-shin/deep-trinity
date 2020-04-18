from typing import Callable, List
import tensorflow as tf
import numpy as np
from ...environment import Environment
from ..batch import MultiBatch
from .. import mcts

DoneCallback = Callable[
    [
        int,  # episode_n
        int,  # step_n
        float,  # episode_reward
    ],
    None,
]


class Agent:
    model: tf.keras.Model
    num_sample_actions: int
    params: mcts.RunParams
    env: Environment
    episode_n: int
    step_n: int
    episode_reward: float

    def __init__(self, model: tf.keras.Model, num_sample_actions: int, params: mcts.RunParams):
        self.model = model
        self.num_sample_actions = num_sample_actions
        self.params = params
        self.env = Environment()
        self.episode_n = 1  # 1-indexed
        self.step_n = 1  # 1-indexed
        self.episode_reward = 0

    def game_strs(self) -> List[str]:
        return [self.env.game_str()]

    def set_model(self, model: tf.keras.Model):
        self.model = model

    def next_state_values(self) -> List[float]:
        if self.env.done:
            return [0]
        x = tf.convert_to_tensor(self.env.observation[None, :])
        _, state_value_batch = self.model.predict_on_batch(x)
        return [float(state_value_batch[0][0])]

    # TODO: about on_done
    def run_steps(self, num_steps: int, on_done: DoneCallback) -> MultiBatch:
        multi_batch = MultiBatch(1, num_steps)
        batch = multi_batch.get(0)
        for i in range(num_steps):
            observation = self.env.observation

            should_sample_action = self.step_n <= self.num_sample_actions
            action, root = mcts.run(self.model, self.env, should_sample_action, self.params)
            _, reward, done = self.env.step(action)
            self.episode_reward += reward

            sum_visits = sum(np.array([child.num_visits for child in root.children.values()]))
            action_probs = np.array([
                root.children[a].num_visits / sum_visits if a in root.children else 0
                for a in range(Environment.num_actions)
            ], dtype=np.float)

            batch.set(i, observation, action_probs, action, reward, done)

            if done:
                on_done(self.episode_n, self.step_n, self.episode_reward)
                self.episode_n += 1
                self.step_n = 1
                self.env.reset()
                continue

            self.step_n += 1

        return multi_batch
