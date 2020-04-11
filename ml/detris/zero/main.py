import os
import argparse
import logging
import json
import math
from typing import Dict
import multiprocessing as mp
import multiprocessing.sharedctypes as mpsc
import numpy as np
import tensorflow as tf
import detris.core

logger = logging.getLogger(__name__)


def setup_logger(level: str = 'INFO'):
    log_format = '%(asctime)s %(levelname)s [%(name)s] %(message)s'
    logging.basicConfig(level=level.upper(), format=log_format)


def load_metadata(file: str):
    data = {
        'next_episode_n': 1,
        'next_update_n': 1,
    }
    if os.path.exists(file):
        with open(file, 'r') as fp:
            data.update(json.load(fp))
    return data


def save_metadata(file: str, data):
    with open(file, 'w') as fp:
        json.dump(data, fp)


# ---

def create_model(input_size, num_actions):
    input = tf.keras.Input(shape=(input_size,))
    x = input
    for _ in range(8):
        x = tf.keras.layers.Dense(512, activation='relu')(x)
    action_probs = tf.keras.layers.Dense(num_actions)(x)
    state_value = tf.keras.layers.Dense(1)(x)
    model = tf.keras.Model(inputs=[input], outputs=[action_probs, state_value])
    return model


def action_probs_loss_fn(actions_and_advantages, action_probs):
    actions, advantages = tf.split(actions_and_advantages, 2, axis=-1)
    actions = tf.cast(actions, tf.int32)
    cce = tf.keras.losses.SparseCategoricalCrossentropy(from_logits=True)
    policy_loss = cce(actions, action_probs, sample_weight=advantages)
    probs = tf.nn.softmax(action_probs)
    entropy_loss = tf.keras.losses.categorical_crossentropy(probs, probs)
    return policy_loss - entropy_loss


def state_value_loss_fn(returns, state_values):
    return tf.keras.losses.mean_squared_error(returns, tf.squeeze(state_values))


def load_or_create_model(model_file):
    env = detris.core.Environment()
    observation_size = len(env.observation())
    if os.path.exists(model_file):
        model = tf.keras.models.load_model(model_file, custom_objects={
            'action_probs_loss_fn': action_probs_loss_fn,
            'state_value_loss_fn': state_value_loss_fn,
        })
    else:
        os.makedirs(os.path.dirname(model_file), exist_ok=True)
        model = create_model(observation_size, env.num_actions())
        model.compile(
            optimizer=tf.optimizers.Adam(),
            loss=[action_probs_loss_fn, state_value_loss_fn],
        )
    return model


# ---

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


def expand_and_simulate(model, env, node: Node) -> float:
    legal_actions = env.legal_actions()
    if len(legal_actions) == 0:
        return 0
    observation = np.array(env.observation())
    x = tf.convert_to_tensor(observation[None, :])
    action_probs_batch, state_value_batch = model.predict_on_batch(x)
    probs = tf.gather(action_probs_batch, tf.constant(legal_actions, dtype=tf.int32), axis=1)
    probs = tf.nn.softmax(probs)
    node.children = {
        legal_actions[i]: Node(float(probs[0][i]))
        for i in range(len(legal_actions))
    }
    return float(tf.squeeze(state_value_batch))


def add_exploration_noise(node: Node, dirichlet_alpha, exploration_fraction):
    actions = node.children.keys()
    noises = np.random.gamma(dirichlet_alpha, 1, len(actions))
    for action, noise in zip(actions, noises):
        node.children[action].action_prob = \
            node.children[action].action_prob * (1 - exploration_fraction) + noise * exploration_fraction


def calc_ucb_score(parent: Node, child: Node, pb_c_base, pb_c_init):
    pb_c = math.log(float(1 + parent.num_visits + pb_c_base) / pb_c_base) + pb_c_init
    return child.avg_state_value() + pb_c * child.action_prob * math.sqrt(parent.num_visits) / (1 + child.num_visits)


def run_mcts(model, env, should_sample_action, num_simulations, root_dirichlet_alpha, root_exploration_fraction,
             pb_c_base, pb_c_init):
    root = Node(0)
    expand_and_simulate(model, env, root)
    add_exploration_noise(root, root_dirichlet_alpha, root_exploration_fraction)

    for i in range(num_simulations):
        node = root
        sim_env = env.clone()
        path = []

        # select
        while not node.is_expanded():
            _, action, node = max(
                (calc_ucb_score(node, child, pb_c_base, pb_c_init), action, child)
                for action, child in node.children.items()
            )
            sim_env.step(action)
            path.append(node)

        # expand and simulate
        value = expand_and_simulate(model, sim_env, node)

        # backpropagate
        for node in path:
            node.sum_state_values += value
            node.num_visits += 1

    # select action
    actions = [(child.num_visits, action) for action, child in root.children.items()]
    if should_sample_action:
        num_visits_arr, action_arr = np.array(list(zip(*actions)))
        softmax_num_visits_arr = np.exp(num_visits_arr)
        softmax_num_visits_arr /= sum(softmax_num_visits_arr)
        mask = np.random.multinomial(1, softmax_num_visits_arr).astype(bool)
        action = action_arr[mask][0]
    else:
        _, action = max(actions)

    child = root.children[action]
    sum_visits = sum([child.num_visits for child in root.children.values()])
    action_prob = child.num_visits / sum_visits if sum_visits > 0 else 0
    state_value = child.sum_state_values / child.num_visits if child.num_visits > 0 else child.num_visits
    # TODO: OK?
    return action, action_prob, state_value


# ---

def model_weight_file(model_file):
    return model_file + '.weights.h5'


TRAIN_REQ_EXIT = 0
TRAIN_REQ_RUN_STEPS = 1


def train_worker(worker_i, args, next_episode_n: mp.Value, req_queue: mp.Queue, res_queue: mp.Queue,
                 out_observations, out_actions, out_action_probs, out_state_values):
    num_workers = args.num_workers
    batch_size = args.worker_batch_size
    weights_file = model_weight_file(args.model_file)

    model = load_or_create_model(args.model_file)
    tb_summary_writer = tf.summary.create_file_writer(args.tb_log_dir)

    env = detris.core.Environment()
    env.reset()
    with next_episode_n.get_lock():
        episode_n = next_episode_n.value
        next_episode_n.value += 1
    episode_reward = 0
    step_i = 0

    actions = np.asarray(out_actions).reshape((num_workers, -1))[worker_i]
    observations = np.asarray(out_observations).reshape((num_workers, batch_size, -1))[worker_i]
    action_probs = np.asarray(out_action_probs).reshape((num_workers, -1))[worker_i]
    state_values = np.asarray(out_state_values).reshape((num_workers, -1))[worker_i]

    while True:
        job = req_queue.get()
        if job == TRAIN_REQ_EXIT:
            res_queue.put((worker_i, TRAIN_REQ_EXIT))
            break
        elif job == TRAIN_REQ_RUN_STEPS:
            model.load_weights(weights_file)
            actions[:] = 0
            state_values[:] = 0
            observations[:] = 0

            for i in range(batch_size):
                observations[i] = env.observation()
                actions[i], action_probs[i], state_values[i] = run_mcts(
                    model, env, step_i < args.num_sampling_steps, args.num_simulations, args.root_dirichlet_alpha,
                    args.root_exploration_fraction, args.pb_c_base, args.pb_c_init)
                # logger.debug('Episode#{} (update={}, batch={}): action={}, state_value={}'.format(
                #     episode_n, update_n, i, actions[i], state_values[i]))
                env.step(actions[i])
                episode_reward += env.last_reward()
                step_i += 1
                if env.is_done():
                    with next_episode_n.get_lock():
                        logger.info('Episode#{}: reward={:.03f}'.format(episode_n, episode_reward))
                        logger.info('game:\n{}'.format(env.game_str()))
                        with tb_summary_writer.as_default():
                            tf.summary.scalar('Rewards', episode_reward, step=episode_n)
                        episode_n = next_episode_n.value
                        next_episode_n.value += 1
                    env.reset()
                    episode_reward = 0
                    step_i = 0

            res_queue.put((worker_i, TRAIN_REQ_RUN_STEPS))


def train(args):
    env = detris.core.Environment()
    env.reset()
    observation_size = len(env.observation())

    os.makedirs(args.tb_log_dir, exist_ok=True)

    metadata_file = args.model_file + '.meta'
    meta = load_metadata(metadata_file)
    update_n = meta['next_update_n']

    batch_size = args.num_workers * args.worker_batch_size
    episode_n = mp.Value('L', lock=True)
    episode_n.value = meta['next_episode_n']
    req_queues = [mp.Queue() for _ in range(args.num_workers)]
    res_queue = mp.Queue()
    out_observations = mpsc.RawArray('f', batch_size * observation_size)
    out_actions = mpsc.RawArray('L', batch_size)
    out_action_probs = mpsc.RawArray('f', batch_size)
    out_state_values = mpsc.RawArray('f', batch_size)
    workers = [mp.Process(target=train_worker, args=(
        i, args, episode_n, req_queues[i], res_queue,
        out_observations, out_actions, out_action_probs, out_state_values,
    )) for i in range(args.num_workers)]

    # Subprocesses must be started before using TensorFlow APIs.
    for w in workers:
        w.start()

    model = load_or_create_model(args.model_file)
    tb_summary_writer = tf.summary.create_file_writer(args.tb_log_dir)

    def save():
        model.save(args.model_file)
        logger.info('The model was saved to {}'.format(args.model_file))
        meta['next_episode_n'] = episode_n.value
        meta['next_update_n'] = update_n
        save_metadata(metadata_file, meta)

    def sync():
        model.save_weights(model_weight_file(args.model_file))

    sync()

    observations = np.asarray(out_observations).reshape((batch_size, -1))
    actions = np.asarray(out_actions)
    action_probs = np.asarray(out_action_probs)
    state_values = np.asarray(out_state_values)

    for _ in range(args.max_updates):
        for q in req_queues:
            q.put(TRAIN_REQ_RUN_STEPS)

        n = 0
        while n != args.num_workers:
            worker_i, req_id = res_queue.get()
            if req_id == TRAIN_REQ_RUN_STEPS:
                n += 1

        actions_and_probs = np.concatenate([actions[:, None], action_probs[:, None]], axis=-1)
        losses = model.train_on_batch(observations, [actions_and_probs, state_values])
        logger.info("Update#{}: losses={}".format(update_n, losses))
        with tb_summary_writer.as_default():
            tf.summary.scalar('Losses', losses[0], step=update_n)

        if update_n % args.model_save_interval == 0:
            save()

        sync()
        update_n += 1

    save()

    for q in req_queues:
        q.put(TRAIN_REQ_EXIT)

    for w in workers:
        w.join()


def test(args):
    # env = detris.core.Environment()
    # model = tf.keras.models.load_model(args.model_file, custom_objects={
    #     'action_probs_loss_fn': lambda: 'dummy',
    #     'state_value_loss_fn': lambda: 'dummy',
    # })
    #
    # rewards = np.zeros((args.num_episodes,))
    # steps = np.zeros((args.num_episodes,))
    # for episode_i in range(args.num_episodes):
    #     episode_n = episode_i + 1
    #     env.reset()
    #     for step_i in range(args.num_steps):
    #         steps[episode_i] += 1
    #         action = decide_action(model, np.array(env.observation()), env.legal_actions())
    #         env.step(action)
    #         rewards[episode_i] += env.last_reward()
    #         if env.is_done():
    #             break
    #     logger.info('Episode#{}: steps={}, rewards={:.03f}, game={}\n'.format(
    #         episode_n, steps[episode_i], rewards[episode_i], env.to_string()))
    pass


def main(arguments=None):
    parser = argparse.ArgumentParser(prog='PROG')
    parser.add_argument('--log_level', default='INFO')
    sub_parser = parser.add_subparsers()

    p = sub_parser.add_parser('train')
    p.add_argument('--model_file', default='tmp/zero/model.h5')
    p.add_argument('--model_save_interval', default=100, type=int)
    p.add_argument('--tb_log_dir', default='tmp/zero/tb_log')
    p.add_argument('--num_sampling_steps', default=30, type=int)
    p.add_argument('--max_steps', default=100, type=int)
    p.add_argument('--num_simulations', default=500, type=int)
    p.add_argument('--root_dirichlet_alpha', default=0.15, type=float)
    p.add_argument('--root_exploration_fraction', default=0.25, type=float)
    p.add_argument('--pb_c_base', default=19652, type=int)
    p.add_argument('--pb_c_init', default=1.25, type=float)
    p.add_argument('--num_workers', default=2, type=int)
    p.add_argument('--worker_batch_size', default=16, type=int)
    p.add_argument('--max_updates', default=1, type=int)
    p.set_defaults(func=train)

    p = sub_parser.add_parser('test')
    p.add_argument('--model_file', default='tmp/zero/model.h5')
    p.add_argument('--num_episodes', default=10, type=int)
    p.add_argument('--num_steps', default=100, type=int)
    p.set_defaults(func=test)

    args = parser.parse_args(arguments)
    setup_logger(args.log_level)
    args.func(args)
