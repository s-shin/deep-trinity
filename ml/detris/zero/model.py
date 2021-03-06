from typing import List
import tensorflow as tf
from ..environment import Environment


# NOTE: AlphaZero's loss function is `(z-v)^2 - pi^sfT log bbp + c norm(sftheta)^2`.
# `(z-v)^2`           ... MeanSquaredError
# `pi^sfT log bbp`    ... CategoricalCrossentropy
# `c norm(sftheta)^2` ... l2(weight_decay) of each layer


def create_model_v1(hidden_layer_units: List[int], weight_decay: float):
    input = tf.keras.Input(shape=(Environment.OBSERVATION_SIZE,))
    x = input
    for units in hidden_layer_units:
        x = tf.keras.layers.Dense(
            units,
            kernel_initializer='he_normal',
            activation='relu',
            kernel_regularizer=tf.keras.regularizers.l2(weight_decay),
        )(x)
    action_probs = tf.keras.layers.Dense(
        Environment.NUM_ACTIONS,
        kernel_initializer='truncated_normal',
        activation='softmax',
        kernel_regularizer=tf.keras.regularizers.l2(weight_decay),
        name='action_probs'
    )(x)
    state_value = tf.keras.layers.Dense(
        1,
        kernel_initializer='truncated_normal',
        activation='tanh',
        kernel_regularizer=tf.keras.regularizers.l2(weight_decay),
        name='state_value'
    )(x)
    model = tf.keras.Model(inputs=[input], outputs=[action_probs, state_value])
    return model


def loss_v1():
    return {
        'action_probs': tf.keras.losses.CategoricalCrossentropy(from_logits=True),
        'state_value': tf.keras.losses.MeanSquaredError(),
    }
