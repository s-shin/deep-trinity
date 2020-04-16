import numpy as np


def softmax(arr):
    if len(arr) == 0:
        return []
    t = np.exp(arr - np.max(arr))
    return t / t.sum(axis=0)
