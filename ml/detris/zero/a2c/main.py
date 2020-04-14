import argparse
import sys
from typing import NamedTuple, Dict, Any
import logging
import json
import os
import shutil
from ..model import create_basic_model
from ...core import Environment

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


class Hyperparams(NamedTuple):
    num_sampling_steps: int = 30
    num_simulations: int = 800
    root_dirichlet_alpha: float = 0.15
    root_exploration_fraction: float = 0.25
    pb_c_base: int = 19652
    pb_c_init: float = 1.25
    batch_size: int = 512


class RunState(NamedTuple):
    num_updates: int = 0


# --- init ---

def register_init(p: argparse.ArgumentParser):
    p.add_argument('--project_dir', default='tmp/detris-zero-a2c')
    p.add_argument('--clean', action='store_true')
    p.add_argument('--num_hidden_layers', default=10, type=int)
    p.add_argument('--num_hidden_layer_units', default=512, type=int)
    for (k, v) in Hyperparams._field_defaults.items():
        p.add_argument(f'--{k}', default=v, type=type(v))
    p.set_defaults(func=init)


def init(args):
    if os.path.exists(args.project_dir):
        if args.clean:
            logger.info(f'Clean {args.project_dir}.')
            shutil.rmtree(args.project_dir)
        else:
            logger.error(f'The specified project directory already exists: {args.project_dir}')
            exit(1)
    os.makedirs(os.path.join(args.project_dir, TB_LOG_DIR), exist_ok=True)

    hyperparams = Hyperparams(**{k: args.__dict__[k] for k in Hyperparams._fields})
    save_json(os.path.join(args.project_dir, HYPERPARAMS_FILE), hyperparams._asdict())

    run_state = RunState()
    save_json(os.path.join(args.project_dir, RUN_STATE_FILE), run_state._asdict())

    model = create_basic_model(Environment(), [args.num_hidden_layer_units] * args.num_hidden_layers)
    model.save(os.path.join(args.project_dir, MODEL_FILE))
    model.summary()

    logger.info('Done!')


# --- train ---

def register_train(p: argparse.ArgumentParser):
    p.set_defaults(func=train)


def train(args):
    print(args)


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
