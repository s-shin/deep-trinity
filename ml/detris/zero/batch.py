from typing import List, NamedTuple, Union
import numpy as np
from . import util
from ..environment import Environment


class Batch(NamedTuple):
    observations: np.ndarray
    action_probs: np.ndarray
    actions: np.ndarray
    rewards: np.ndarray
    dones: np.ndarray

    @property
    def batch_size(self) -> int:
        return self.observations.shape[0]

    @classmethod
    def zeros(cls, size: int) -> 'Batch':
        return cls(
            np.zeros((size, Environment.OBSERVATION_SIZE), dtype=np.float32),
            np.zeros((size, Environment.NUM_ACTIONS), dtype=np.float32),
            np.zeros((size,), dtype=np.uint32),
            np.zeros((size,), dtype=np.float32),
            np.zeros((size,), dtype=np.uint8),
        )

    def __len__(self) -> int:
        return self.observations.shape[0]

    def set(self, i: int, observation: np.ndarray, action_probs: np.ndarray, action: int, reward: float, done: bool):
        self.observations[i] = observation
        self.action_probs[i] = action_probs
        self.actions[i] = action
        self.rewards[i] = reward
        self.dones[i] = done

    def discounted_cumulative_rewards(self, discount_rate: float, next_state_value: float) -> np.ndarray:
        r = np.append(np.zeros_like(self.rewards, dtype=np.float32), next_state_value)
        for i in reversed(range(self.rewards.shape[0])):
            r[i] = self.rewards[i] + discount_rate * r[i + 1] * (1 - self.dones[i])
        return util.standalize(r[:-1])


class MultiBatch(NamedTuple):
    observations: np.ndarray
    action_probs: np.ndarray
    actions: np.ndarray
    rewards: np.ndarray
    dones: np.ndarray

    @property
    def num_multi(self) -> int:
        return self.observations.shape[0]

    @property
    def batch_size(self) -> int:
        return self.observations.shape[1]

    @classmethod
    def zeros(cls, n: int, size: int) -> 'MultiBatch':
        return cls(
            np.zeros((n, size, Environment.OBSERVATION_SIZE), dtype=np.float32),
            np.zeros((n, size, Environment.NUM_ACTIONS), dtype=np.float32),
            np.zeros((n, size), dtype=np.uint32),
            np.zeros((n, size), dtype=np.float32),
            np.zeros((n, size), dtype=np.uint8),
        )

    def get(self, i: int) -> Batch:
        return Batch(
            self.observations.view()[i],
            self.action_probs.view()[i],
            self.actions.view()[i],
            self.rewards.view()[i],
            self.dones.view()[i],
        )

    def discounted_cumulative_rewards(self, discount_rate: float,
                                      next_state_values: Union[List[float], np.ndarray]) -> np.ndarray:
        r = np.empty_like(self.rewards)
        for i in range(len(r)):
            r[i] = self.get(i).discounted_cumulative_rewards(discount_rate, next_state_values[i])
        return r
