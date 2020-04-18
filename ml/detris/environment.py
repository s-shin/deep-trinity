import numpy as np
from . import core
from typing import ClassVar, List
import copy


def normalize_observation(observation) -> np.ndarray:
    return np.array(observation) / np.iinfo(np.uint32).max


class Environment:
    num_actions: ClassVar[int] = core.Environment.num_actions()
    observation_size: ClassVar[int] = len(core.Environment().observation())
    base_step_reward: float
    env: core.Environment
    observation: np.ndarray
    reward: float
    done: bool

    def __init__(self, base_step_reward=0.01):
        self.base_step_reward = base_step_reward
        self.env = core.Environment()
        self.observation = normalize_observation(self.env.observation())
        self.reward = 0
        self.done = False

    def reset(self) -> np.ndarray:
        self.env.reset()
        self.observation = normalize_observation(self.env.observation())
        return self.observation

    def step(self, action: int) -> (np.ndarray, float, bool):
        self.env.step(action)
        self.observation = np.array(self.env.observation())
        self.reward = self.env.last_reward() + self.base_step_reward
        self.done = self.env.is_done()
        return self.observation, self.reward, self.done

    def legal_actions(self) -> List[int]:
        return self.env.legal_actions()

    def game_str(self) -> str:
        return self.env.game_str()

    def clone(self) -> 'Environment':
        env = copy.copy(self)
        env.env = self.env.clone()
        return env
