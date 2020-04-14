from typing import List
import tensorflow as tf


def create_basic_model(env, hidden_layer_units: List[int]):
    input = tf.keras.Input(shape=(len(env.observation()),))
    x = input
    for units in hidden_layer_units:
        x = tf.keras.layers.Dense(units, activation='relu')(x)
    action_probs = tf.keras.layers.Dense(env.num_actions())(x)
    state_value = tf.keras.layers.Dense(1)(x)
    model = tf.keras.Model(inputs=[input], outputs=[action_probs, state_value])
    return model
