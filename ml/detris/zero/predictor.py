import tensorflow as tf
import numpy as np
from typing import Callable, Optional


class Predictor:
    def predict(self, observation: np.ndarray) -> (np.ndarray, float):
        raise NotImplementedError()

    def reload_model(self):
        raise NotImplementedError()


LoadModelFunc = Callable[[], tf.keras.Model]


class BasicPredictor(Predictor):
    load_model: LoadModelFunc
    model: Optional[tf.keras.Model]

    def __init__(self, load_model: LoadModelFunc):
        self.load_model = load_model
        self.model = None

    def predict(self, observation: np.ndarray) -> (np.ndarray, float):
        assert self.model is not None
        action_probs_batch, state_value_batch = self.model.predict_on_batch(tf.convert_to_tensor(observation[None, :]))
        return action_probs_batch[0].numpy(), float(state_value_batch[0][0])

    def reload_model(self):
        self.model = self.load_model()
