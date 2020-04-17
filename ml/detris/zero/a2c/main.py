import argparse
import sys
from typing import NamedTuple, Dict, Any, List
import logging
import json
import os
import shutil
import tensorflow as tf
import numpy as np
from ...core import Environment
from .. import mcts, util, model as M
from .agent import Agent

logger = logging.getLogger(__name__)


def setup_logger(level: str = 'INFO'):
    logging.basicConfig(level=level.upper(), stream=sys.stdout,
                        format='%(asctime)s %(levelname)s [%(name)s] %(message)s')


def load_json(file: str) -> Dict[str, Any]:
    with open(file, 'r') as fp:
        return json.load(fp)


def save_json(file: str, data: Dict[str, Any]):
    with open(file, 'w') as fp:
        json.dump(data, fp)


# ---

HYPERPARAMS_FILE = 'hyperparams.json'
RUN_STATE_FILE = 'run_state.json'
MODEL_FILE = 'model.h5'
TB_LOG_DIR = 'tb_log'
DEFAULT_PROJECT_DIR = 'tmp/detris-zero-a2c'


class Hyperparams(NamedTuple):
    num_sampling_steps: int = 30
    num_simulations: int = 800
    root_dirichlet_alpha: float = 0.15
    root_exploration_fraction: float = 0.25
    pb_c_base: int = 19652
    pb_c_init: float = 1.25
    batch_size: int = 512
    max_steps: int = 200
    optimizer: str = 'adam'
    weight_decay: float = 1e-4
    learning_rate_boundaries: List[int] = [100_000, 300_000, 500_000]
    learning_rate_values: List[float] = [2e-1, 2e-2, 2e-3, 2e-4]
    discount_rate: float = 0.9


class RunState(NamedTuple):
    update_n: int = 1
    episode_n: int = 1


class Project:
    dir: str

    def __init__(self, dir: str):
        self.dir = dir

    def exists(self) -> bool:
        return os.path.exists(self.dir)

    def rm_rf(self):
        shutil.rmtree(self.dir)

    def mkdir_p(self):
        os.makedirs(self.tb_log_dir(), exist_ok=True)

    def tb_log_dir(self) -> str:
        return os.path.join(self.dir, TB_LOG_DIR)

    def hyperparams_file(self) -> str:
        return os.path.join(self.dir, HYPERPARAMS_FILE)

    def run_state_file(self) -> str:
        return os.path.join(self.dir, RUN_STATE_FILE)

    def model_file(self) -> str:
        return os.path.join(self.dir, MODEL_FILE)

    def load_hyperparams(self) -> Hyperparams:
        return Hyperparams(**load_json(self.hyperparams_file()))

    def save_hyperparams(self, hyperparams: Hyperparams):
        save_json(self.hyperparams_file(), hyperparams._asdict())

    def load_run_state(self) -> RunState:
        return RunState(**load_json(self.run_state_file()))

    def save_run_state(self, run_state: RunState):
        save_json(self.run_state_file(), run_state._asdict())


# --- init ---

def register_init(p: argparse.ArgumentParser):
    p.add_argument('--project_dir', default=DEFAULT_PROJECT_DIR)
    p.add_argument('--clean', action='store_true')
    p.add_argument('--num_hidden_layers', default=10, type=int)
    p.add_argument('--num_hidden_layer_units', default=512, type=int)
    types = {
        'learning_rate_boundaries': lambda s: list(map(int, s.split(','))),
        'learning_rate_values': lambda s: list(map(float, s.split(','))),
    }
    for (k, v) in Hyperparams._field_defaults.items():
        p.add_argument(f'--{k}', default=v, type=types[k] if k in types else type(v))
    p.set_defaults(func=init)


def init(args):
    project = Project(args.project_dir)
    if project.exists():
        if args.clean:
            logger.info(f'Clean {args.project_dir}.')
            project.rm_rf()
            project.mkdir_p()
        else:
            logger.error(f'The specified project directory already exists: {args.project_dir}')
            exit(1)
    else:
        logger.info(f'Create {args.project_dir}.')
        project.mkdir_p()

    logger.info(f'Create {HYPERPARAMS_FILE}.')
    hyperparams = Hyperparams(**{k: args.__dict__[k] for k in Hyperparams._fields})
    project.save_hyperparams(hyperparams)

    logger.info(f'Create {RUN_STATE_FILE}.')
    run_state = RunState()
    project.save_run_state(run_state)

    logger.info(f'Create {MODEL_FILE}.')
    model = M.create_model_v1(
        Environment(),
        [args.num_hidden_layer_units] * args.num_hidden_layers,
        hyperparams.weight_decay,
    )
    optimizer = None
    if args.optimizer == 'adam':
        optimizer = tf.keras.optimizers.Adam(
            tf.keras.optimizers.schedules.PiecewiseConstantDecay(
                hyperparams.learning_rate_boundaries,
                hyperparams.learning_rate_values,
            )
        )
    assert optimizer is not None
    model.compile(optimizer, M.loss_v1())
    model.save(project.model_file())
    model.summary()

    logger.info('Done!')


# --- train ---

class LossLoggingCallback(tf.keras.callbacks.Callback):
    def __init__(self, episode_n, tb_summary_writer):
        super(LossLoggingCallback, self).__init__()
        self.episode_n = episode_n
        self.tb_summary_writer = tb_summary_writer

    def on_epoch_end(self, epoch, logs=None):
        with self.tb_summary_writer.as_default():
            tf.summary.scalar('Losses', logs['loss'], step=self.episode_n)


def register_train(p: argparse.ArgumentParser):
    p.add_argument('--project_dir', default=DEFAULT_PROJECT_DIR)
    p.add_argument('--num_updates', default=100, type=int)
    p.set_defaults(func=train)


def train(args):
    train_in_this_process(args)


def train_with_multiprocess():
    pass


def train_in_this_process(args):
    project = Project(args.project_dir)
    hyperparams = project.load_hyperparams()
    model = tf.keras.models.load_model(project.model_file(), custom_objects=M.loss_v1())
    agent = Agent(model, hyperparams.num_sampling_steps, mcts.RunParams(
        hyperparams.num_simulations,
        hyperparams.root_dirichlet_alpha,
        hyperparams.root_exploration_fraction,
        hyperparams.pb_c_base,
        hyperparams.pb_c_init,
    ))
    run_state = project.load_run_state()
    tb_summary_writer = tf.summary.create_file_writer(project.tb_log_dir())

    def save():
        model.save(project.model_file())
        project.save_run_state(run_state)

    observation_batch = np.empty((hyperparams.batch_size, agent.env_observation_size()), dtype=np.uint32)
    action_probs_batch = np.empty((hyperparams.batch_size, agent.env_num_actions()), dtype=np.float)
    reward_batch = np.empty((hyperparams.batch_size,), dtype=np.float)
    done_batch = np.empty((hyperparams.batch_size,), dtype=np.bool)

    def store(i, observation, action_probs, _action, reward, is_done):
        observation_batch[i] = util.normalize_observation(observation)
        action_probs_batch[i] = action_probs
        reward_batch[i] = reward
        done_batch[i] = is_done

    def on_done(_episode_n, step_n, episode_reward):
        nonlocal run_state
        n = run_state.episode_n
        logger.info(f'Episode#{n}: steps={step_n}, reward={episode_reward}')
        logger.info(f'game:\n{agent.env_game_str()}')
        with tb_summary_writer.as_default():
            tf.summary.scalar('Rewards', episode_reward, step=n)
        run_state = run_state._replace(episode_n=n + 1)

    for _ in range(args.num_updates):
        agent.run_steps(hyperparams.batch_size, store, on_done)

        return_batch = np.append(np.zeros_like(reward_batch), agent.next_state_value())
        for i in reversed(range(reward_batch.shape[0])):
            return_batch[i] = reward_batch[i] + hyperparams.discount_rate * return_batch[i + 1] * (1 - done_batch[i])
        return_batch = return_batch[:-1]
        # Standarize
        return_batch = (return_batch - return_batch.mean()) / (return_batch.std() + np.finfo(np.float32).eps.item())

        logger.info(f'Update#{run_state.update_n}:')
        model.fit(
            observation_batch,
            [action_probs_batch, return_batch],
            batch_size=hyperparams.batch_size,
            callbacks=[LossLoggingCallback(run_state.update_n, tb_summary_writer)]
        )

        run_state = run_state._replace(update_n=run_state.update_n + 1)
        save()

    logger.info('Done!')


# --- main ---

def main(arguments=None):
    parser = argparse.ArgumentParser(prog='PROG')
    parser.add_argument('--log_level', default='INFO')
    sub_parsers = parser.add_subparsers()
    register_init(sub_parsers.add_parser('init'))
    register_train(sub_parsers.add_parser('train'))
    args = parser.parse_args(arguments)
    setup_logger(args.log_level)
    args.func(args)
