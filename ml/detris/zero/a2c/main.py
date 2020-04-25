import argparse
import sys
from typing import NamedTuple, List
import logging
import os
import shutil
import numpy as np
import tensorflow as tf
from .. import mcts, util, model as M
from ..predictor import BasicPredictor
from .agent import AgentParams, basic_agent, multiprocess_agent

logger = logging.getLogger(__name__)


def setup_logger(level: str = 'INFO'):
    logging.basicConfig(level=level.upper(), stream=sys.stdout,
                        format='%(asctime)s %(levelname)s [%(name)s] %(message)s')


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
        return Hyperparams(**util.load_json(self.hyperparams_file()))

    def save_hyperparams(self, hyperparams: Hyperparams):
        util.save_json(self.hyperparams_file(), hyperparams._asdict())

    def load_run_state(self) -> RunState:
        return RunState(**util.load_json(self.run_state_file()))

    def save_run_state(self, run_state: RunState):
        util.save_json(self.run_state_file(), run_state._asdict())


# --- init ---

def register_init(p: argparse.ArgumentParser):
    p.add_argument('--project_dir', default=DEFAULT_PROJECT_DIR)
    p.add_argument('--clean', action='store_true')
    p.add_argument('--num_hidden_layers', default=10, type=int)
    p.add_argument('--num_hidden_layer_units', default=1024, type=int)
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

def register_train(p: argparse.ArgumentParser):
    p.add_argument('--project_dir', default=DEFAULT_PROJECT_DIR)
    p.add_argument('--num_updates', default=100, type=int)
    p.add_argument('--num_workers', default=0, type=int)  # hyperparam?
    p.set_defaults(func=train)


def train(args):
    project = Project(args.project_dir)
    model = None
    hyperparams = project.load_hyperparams()
    params = AgentParams(
        hyperparams.num_sampling_steps,
        mcts.RunParams(
            hyperparams.num_simulations,
            hyperparams.root_dirichlet_alpha,
            hyperparams.root_exploration_fraction,
            hyperparams.pb_c_base,
            hyperparams.pb_c_init,
        )
    )

    is_mp = args.num_workers > 0
    if not is_mp:
        agent = basic_agent.BasicAgent(BasicPredictor(lambda: model), hyperparams.batch_size, params)
    else:
        if hyperparams.batch_size % args.num_workers != 0:
            logger.error('Invalid num_workers.')
            exit(1)
        agent = multiprocess_agent.MultiprocessAgent(
            BasicPredictor(lambda: tf.keras.models.load_model(project.model_file(), custom_objects=M.loss_v1())),
            hyperparams.batch_size // args.num_workers, args.num_workers, params,
        )

    model = tf.keras.models.load_model(project.model_file(), custom_objects=M.loss_v1())
    run_state = project.load_run_state()
    tb_summary_writer = tf.summary.create_file_writer(project.tb_log_dir())

    def save():
        model.save(project.model_file())
        project.save_run_state(run_state)

    def on_done(_episode_n, step_n, episode_reward, game_str):
        nonlocal run_state
        n = run_state.episode_n
        logger.info(f'Episode#{n}: steps={step_n}, reward={episode_reward:.3f}, game=\n{game_str}')
        with tb_summary_writer.as_default():
            tf.summary.scalar('Rewards', episode_reward, step=n)
        run_state = run_state._replace(episode_n=n + 1)

    logger.info('Training started.')

    for _ in range(args.num_updates):
        agent.sync_model()
        multi_batch = agent.run_steps(on_done)

        mask = np.where(multi_batch.dones[:, -1] == 0)
        next_state_values = np.zeros((multi_batch.batch_size,))
        if len(mask) > 0:
            last_observations = multi_batch.observations[:, -1][mask]
            _, state_values = model.predict_on_batch(tf.convert_to_tensor(last_observations))
            next_state_values[mask] = state_values.numpy().reshape((-1,))
        returns = multi_batch.discounted_cumulative_rewards(hyperparams.discount_rate, next_state_values)

        logger.info(f'Update#{run_state.update_n}:')
        model.fit(
            multi_batch.observations.reshape((-1, multi_batch.observations.shape[-1])),
            [
                multi_batch.action_probs.reshape((-1, multi_batch.action_probs.shape[-1])),
                returns.reshape((-1,)),
            ],
            batch_size=hyperparams.batch_size,
            callbacks=[util.LossLoggingCallback(run_state.update_n, tb_summary_writer)],
        )

        run_state = run_state._replace(update_n=run_state.update_n + 1)
        save()

    agent.exit()
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
