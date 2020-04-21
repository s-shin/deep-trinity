import numpy as np
import tensorflow as tf
import json
from typing import Dict, Any


def load_json(file: str) -> Dict[str, Any]:
    with open(file, 'r') as fp:
        return json.load(fp)


def save_json(file: str, data: Dict[str, Any]):
    with open(file, 'w') as fp:
        json.dump(data, fp)


def softmax(arr1d):
    if len(arr1d) == 0:
        return arr1d
    arr1d = np.asarray(arr1d)
    assert len(arr1d.shape) == 1
    t = np.exp(arr1d - np.max(arr1d))
    return t / t.sum()


FLOAT32_EPS = np.finfo(np.float32).eps.item()


def standalize(arr1d):
    if len(arr1d) == 0:
        return arr1d
    arr1d = np.asarray(arr1d)
    assert len(arr1d.shape) == 1
    return (arr1d - arr1d.mean()) / (arr1d.std() + FLOAT32_EPS)


class LossLoggingCallback(tf.keras.callbacks.Callback):
    def __init__(self, episode_n, tb_summary_writer):
        super(LossLoggingCallback, self).__init__()
        self.episode_n = episode_n
        self.tb_summary_writer = tb_summary_writer

    def on_epoch_end(self, epoch, logs=None):
        with self.tb_summary_writer.as_default():
            tf.summary.scalar('Losses', logs['loss'], step=self.episode_n)
