from typing import Optional, List
import numpy as np
from . import util
from ..environment import Environment


class Batch:
    observations: np.ndarray
    action_probs: np.ndarray
    actions: np.ndarray
    rewards: np.ndarray
    dones: np.ndarray

    def __init__(self, size: int, observations: Optional[np.ndarray] = None,
                 action_probs: Optional[np.ndarray] = None, actions: Optional[np.ndarray] = None,
                 rewards: Optional[np.ndarray] = None, dones: Optional[np.ndarray] = None):
        self.observations = observations if observations is not None \
            else np.zeros((size, Environment.observation_size), dtype=np.float)
        self.action_probs = action_probs if action_probs is not None \
            else np.zeros((size, Environment.num_actions), dtype=np.float)
        self.actions = actions if actions is not None \
            else np.zeros((size,), dtype=np.uint32)
        self.rewards = rewards if rewards is not None \
            else np.zeros((size,), dtype=np.float)
        self.dones = dones if dones is not None \
            else np.zeros((size,), dtype=np.uint8)

    def set(self, i: int, observation: np.ndarray, action_probs: np.ndarray, action: int, reward: float, done: bool):
        self.observations[i] = observation
        self.action_probs[i] = action_probs
        self.actions[i] = action
        self.rewards[i] = reward
        self.dones[i] = done

    def discounted_cumulative_rewards(self, discount_rate: float, next_state_value: float) -> np.ndarray:
        r = np.append(np.zeros_like(self.rewards, dtype=np.float), next_state_value)
        for i in reversed(range(self.rewards.shape[0])):
            r[i] = self.rewards[i] + discount_rate * r[i + 1] * (1 - self.dones[i])
        return util.standalize(r[:-1])


class MultiBatch:
    observations: np.ndarray
    action_probs: np.ndarray
    actions: np.ndarray
    rewards: np.ndarray
    dones: np.ndarray

    def __init__(self, n: int, size: int):
        self.observations = np.zeros((n, size, Environment.observation_size), dtype=np.float)
        self.action_probs = np.zeros((n, size, Environment.num_actions), dtype=np.float)
        self.actions = np.zeros((n, size), dtype=np.uint32)
        self.rewards = np.zeros((n, size), dtype=np.float)
        self.dones = np.zeros((n, size), dtype=np.uint8)

    def get(self, i: int) -> Batch:
        return Batch(
            0,
            self.observations.view()[i],
            self.action_probs.view()[i],
            self.actions.view()[i],
            self.rewards.view()[i],
            self.dones.view()[i],
        )

    def discounted_cumulative_rewards(self, discount_rate: float, next_state_values: List[float]) -> np.ndarray:
        r = np.empty_like(self.rewards)
        for i in range(len(r)):
            r[i] = self.get(i).discounted_cumulative_rewards(discount_rate, next_state_values[i])
        return r
