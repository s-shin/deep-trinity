import numpy as np

FLOAT32_EPS = np.finfo(np.float32).eps.item()


def softmax(arr1d):
    if len(arr1d) == 0:
        return arr1d
    arr1d = np.asarray(arr1d)
    assert len(arr1d.shape) == 1
    t = np.exp(arr1d - np.max(arr1d))
    return t / t.sum()


def normalize_observation(observation) -> float:
    return np.array(observation) / np.iinfo(np.uint32).max


def standalize(arr1d):
    if len(arr1d) == 0:
        return arr1d
    arr1d = np.asarray(arr1d)
    assert len(arr1d.shape) == 1
    return (arr1d - arr1d.mean()) / (arr1d.std() + FLOAT32_EPS)
