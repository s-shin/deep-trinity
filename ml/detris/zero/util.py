import numpy as np


def softmax(arr1d):
    if len(arr1d) == 0:
        return []
    arr1d = np.asarray(arr1d)
    assert len(arr1d.shape) == 1
    t = np.exp(arr1d - np.max(arr1d))
    return t / t.sum()


def normalize_observation(observation) -> float:
    return np.array(observation) / np.iinfo(np.uint32).max
