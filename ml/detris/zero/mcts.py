import math
import random
from typing import Dict, NamedTuple
import numpy as np
from ..environment import Environment
from . import util
from .predictor import Predictor


class Node:
    num_visits: int
    sum_state_values: float
    action_prob: float
    children: Dict[int, 'Node']

    def __init__(self, action_prob: float):
        self.num_visits = 0
        self.sum_state_values = 0
        self.action_prob = action_prob
        self.children = {}

    def avg_state_value(self) -> float:
        if self.num_visits == 0:
            return 0
        return self.sum_state_values / self.num_visits

    def is_expanded(self) -> bool:
        return len(self.children) == 0


def expand(node: Node, predictor: Predictor, env: Environment) -> float:
    all_action_probs, state_value = predictor.predict(env.observation)
    legal_actions = env.legal_actions()
    action_probs = util.softmax([all_action_probs[a] for a in legal_actions])
    for (i, action) in enumerate(legal_actions):
        node.children[action] = Node(action_probs[i])
    return state_value


def add_exploration_noise(node: Node, dirichlet_alpha: float, exploration_fraction: float):
    actions = node.children.keys()
    noises = np.random.gamma(dirichlet_alpha, 1, len(actions))
    for action, noise in zip(actions, noises):
        node.children[action].action_prob = \
            node.children[action].action_prob * (1 - exploration_fraction) + noise * exploration_fraction


def calc_ucb_score(parent: Node, child: Node, pb_c_base: int, pb_c_init: float) -> float:
    pb_c = math.log((1 + parent.num_visits + pb_c_base) / pb_c_base) + pb_c_init
    return child.avg_state_value() + pb_c * child.action_prob * math.sqrt(parent.num_visits) / (1 + child.num_visits)


def select_action(node: Node, should_sample_action: bool) -> int:
    actions = [(child.num_visits, action) for action, child in node.children.items()]
    if should_sample_action:
        num_visits_arr, action_arr = np.array(list(zip(*actions)))
        mask = np.random.multinomial(1, util.softmax(num_visits_arr)).astype(bool)
        action = action_arr[mask][0]
    else:
        _, action = max(actions)
    return action


class RunParams(NamedTuple):
    num_simulations: int
    root_dirichlet_alpha: float
    root_exploration_fraction: float
    pb_c_base: int
    pb_c_init: float


def run(predictor: Predictor, env: Environment, should_sample_action: bool, params: RunParams) -> (int, Node):
    root = Node(0)
    expand(root, predictor, env)
    add_exploration_noise(root, params.root_dirichlet_alpha, params.root_exploration_fraction)

    for _ in range(params.num_simulations):
        node = root
        sim_env = env.clone()
        path = []

        # select
        while not node.is_expanded():
            children = sorted(
                (calc_ucb_score(node, child, params.pb_c_base, params.pb_c_init), action, child)
                for action, child in node.children.items()
            )
            _, action, node = children[-1] if children[-1][0] > 0 else random.choice(children)
            # _, action, node = max(
            #     (calc_ucb_score(node, child, params.pb_c_base, params.pb_c_init), action, child)
            #     for action, child in node.children.items()
            # )
            sim_env.step(action)
            path.append(node)

        value = expand(node, predictor, sim_env)

        # backpropagate
        for node in path:
            node.sum_state_values += value
            node.num_visits += 1

    # select action
    action = select_action(root, should_sample_action)

    # for (a, c) in sorted(root.children.items()):
    #     print('{:4d} => num_visits={:<4d} action_prob={:.3f} avg_state_value={}'.format(
    #         a, c.num_visits, c.action_prob, c.avg_state_value()))

    return action, root
