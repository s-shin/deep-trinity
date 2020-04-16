from typing import Callable, List
import tensorflow as tf
import numpy as np
from ...core import Environment
from .. import mcts

StoreFunc = Callable[
    [
        int,  # step index of current update
        List[int],  # observations
        List[float],  # action_probs[num_actions]
        int,  # action
        float,  # reward
        bool  # is_done
    ],
    None,
]

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
    observation_size: int
    episode_n: int
    step_n: int
    episode_reward: float

    def __init__(self, model, num_sample_actions: int, params: mcts.RunParams):
        self.model = model
        self.num_sample_actions = num_sample_actions
        self.params = params
        self.env = Environment()
        self.observation_size = len(self.env.observation())
        self.episode_n = 1  # 1-indexed
        self.step_n = 1  # 1-indexed
        self.episode_reward = 0

    def env_observation_size(self) -> int:
        return self.observation_size

    def env_num_actions(self) -> int:
        return self.env.num_actions()

    def env_game_str(self) -> str:
        return self.env.game_str()

    def set_model(self, model):
        self.model = model

    def should_sample_action(self) -> bool:
        return self.step_n <= self.num_sample_actions

    def next_state_value(self) -> float:
        if self.env.is_done():
            return 0
        observation = np.array(self.env.observation())
        x = tf.convert_to_tensor(observation[None, :])
        _, state_value_batch = self.model.predict_on_batch(x)
        return float(state_value_batch[0][0])

    def run_steps(self, num_steps: int, store: StoreFunc, on_done: DoneCallback):
        assert self.model is not None
        for i in range(num_steps):
            observation = self.env.observation()

            action, root = mcts.run(self.model, self.env, self.should_sample_action(), self.params)
            self.env.step(action)

            reward = self.env.last_reward()
            self.episode_reward += reward
            is_done = self.env.is_done()

            sum_visits = sum([child.num_visits for child in root.children.values()])
            action_probs = [
                root.children[a].num_visits / sum_visits if a in root.children else 0
                for a in range(self.env.num_actions())
            ]
            store(i, observation, action_probs, action, reward, is_done)

            if is_done:
                on_done(self.episode_n, self.step_n, self.episode_reward)
                self.episode_n += 1
                self.step_n = 1
                self.env.reset()
                continue

            self.step_n += 1
