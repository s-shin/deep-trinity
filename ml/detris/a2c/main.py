import os
import logging
import json
import numpy as np
import tensorflow as tf
import detris.core

logger = logging.getLogger(__name__)


def setup_logger(level: str = 'INFO'):
    log_format = '%(asctime)s %(levelname)s [%(name)s] %(message)s'
    logging.basicConfig(level=level.upper(), format=log_format)


def load_metadata(file: str):
    if os.path.exists(file):
        with open(file, 'r') as fp:
            return json.load(fp)
    return {
        'next_episode_n': 0,
        'next_update_n': 0,
    }


def save_metadata(file: str, data):
    with open(file, 'w') as fp:
        json.dump(data, fp)


def create_model(input_size, num_actions):
    input = tf.keras.Input(shape=(input_size,))
    x = input
    for _ in range(8):
        x = tf.keras.layers.Dense(512, activation='relu')(x)
    action_probs = tf.keras.layers.Dense(num_actions, activation='softmax')(x)
    state_value = tf.keras.layers.Dense(1)(x)
    model = tf.keras.Model(inputs=[input], outputs=[action_probs, state_value])
    return model


def decide_action(model, observation, legal_actions):
    x = tf.convert_to_tensor(observation[None, :])
    action_probs_batch, state_value_batch = model.predict_on_batch(x)
    probs = tf.gather(action_probs_batch, np.array(legal_actions, dtype=np.int32), axis=1)
    probs = tf.nn.softmax(probs)
    idx = tf.random.categorical(probs, 1)
    return legal_actions[np.squeeze(idx)], np.squeeze(state_value_batch)


def action_probs_loss_fn(actions_and_advantages, action_probs):
    actions, advantages = tf.split(actions_and_advantages, 2, axis=-1)
    actions = tf.cast(actions, tf.int32)
    cce = tf.keras.losses.SparseCategoricalCrossentropy(from_logits=True)
    return cce(actions, action_probs, sample_weight=advantages)


def state_value_loss_fn(returns, state_values):
    return tf.keras.losses.mean_squared_error(returns, tf.squeeze(state_values))


def train(max_updates=1000, batch_size=64, gamma=0.99, model_file='tmp/detris-a2c.h5', tb_log_dir='tmp/tb_log'):
    env = detris.core.Environment()
    env.reset()
    input_size = len(env.observation())

    metadata_file = model_file + '.meta'
    meta = load_metadata(metadata_file)

    if os.path.exists(model_file):
        model = tf.keras.models.load_model(model_file, custom_objects={
            'action_probs_loss_fn': action_probs_loss_fn,
            'state_value_loss_fn': state_value_loss_fn,
        })
    else:
        model = create_model(input_size, env.action_space())
        model.compile(
            optimizer=tf.optimizers.Adam(),
            loss=[action_probs_loss_fn, state_value_loss_fn],
        )

    tb_summary_writer = tf.summary.create_file_writer(tb_log_dir)

    env.reset()
    observation_space = len(env.observation())
    episode_n = meta['next_episode_n']
    update_n = meta['next_update_n']

    def save_model():
        model.save(model_file)
        logger.info('The model was saved to {}'.format(model_file))
        meta['next_episode_n'] = episode_n
        meta['next_update_n'] = update_n
        save_metadata(metadata_file, meta)

    episode_reward = 0
    for _ in range(max_updates):
        actions = np.empty((batch_size,), dtype=np.int32)
        rewards = np.empty((batch_size,), dtype=np.float)
        dones = np.empty((batch_size,), dtype=np.bool)
        state_values = np.empty((batch_size,), dtype=np.float)
        observations = np.empty((batch_size,) + (observation_space,))

        for i in range(batch_size):
            observations[i] = env.observation()
            actions[i], state_values[i] = decide_action(
                model, observations[i], env.legal_actions())
            logger.debug('Episode#{} (update={}, batch={}): action={}, state_value={}'.format(
                episode_n, update_n, i, actions[i], state_values[i]))
            env.step(actions[i])
            rewards[i] = env.last_reward()
            episode_reward += rewards[i]
            dones[i] = env.is_done()
            if dones[i]:
                logger.info('Episode#{} (update={}): reward={}'.format(episode_n, update_n, episode_reward))
                # logger.info('game:\n{}'.format(env.to_string()))
                with tb_summary_writer.as_default():
                    tf.summary.scalar('Rewards', episode_reward, step=episode_n)
                episode_n += 1
                episode_reward = 0
                env.reset()

        if dones[-1]:
            next_state_value = 0
        else:
            _, next_state_value = decide_action(
                model, observations[-1], env.legal_actions())
        returns = np.append(np.zeros_like(rewards), next_state_value)
        for i in reversed(range(rewards.shape[0])):
            returns[i] = rewards[i] + gamma * returns[i + 1] * (1 - dones[i])
        returns = returns[:-1]
        advantages = returns - state_values
        actions_and_advantages = np.concatenate([actions[:, None], advantages[:, None]], axis=-1)
        losses = model.train_on_batch(observations, [actions_and_advantages, returns])

        update_n += 1
        if update_n % 100 == 0:
            logger.info("[{}/{}] Losses: {}".format(update_n, next_state_value + max_updates, losses))
            with tb_summary_writer.as_default():
                tf.summary.scalar('Losses', losses[0], step=update_n)
            save_model()

    save_model()


def main():
    setup_logger()
    train()
