from typing import List, NamedTuple
import enum
import multiprocessing as mp
import ctypes
import logging
import numpy as np
from . import Agent, AgentParams, AgentCore, DoneCallback
from ...batch import MultiBatch
from ...predictor import Predictor
from ....environment import Environment

logger = logging.getLogger(__name__)


class SharedBuffers(NamedTuple):
    num_workers: int
    batch_size: int
    observations: mp.Array
    action_probs: mp.Array
    actions: mp.Array
    rewards: mp.Array
    dones: mp.Array

    @classmethod
    def zeros(cls, num_workers: int, batch_size: int) -> 'SharedBuffers':
        return cls(
            num_workers,
            batch_size,
            mp.Array(ctypes.c_float, num_workers * batch_size * Environment.OBSERVATION_SIZE),
            mp.Array(ctypes.c_float, num_workers * batch_size * Environment.NUM_ACTIONS),
            mp.Array(ctypes.c_uint32, num_workers * batch_size),
            mp.Array(ctypes.c_float, num_workers * batch_size),
            mp.Array(ctypes.c_uint8, num_workers * batch_size),
        )

    def as_multi_batch(self) -> MultiBatch:
        return MultiBatch(
            np.frombuffer(self.observations.get_obj(), dtype=np.float32)
                .reshape((self.num_workers, self.batch_size, -1)),
            np.frombuffer(self.action_probs.get_obj(), dtype=np.float32)
                .reshape((self.num_workers, self.batch_size, -1)),
            np.frombuffer(self.actions.get_obj(), dtype=np.uint32).reshape((self.num_workers, -1)),
            np.frombuffer(self.rewards.get_obj(), dtype=np.float32).reshape((self.num_workers, -1)),
            np.frombuffer(self.dones.get_obj(), dtype=np.uint8).reshape((self.num_workers, -1)),
        )


class RequestType(enum.Enum):
    EXIT = enum.auto()
    SYNC_MODEL = enum.auto()
    RUN_STEPS = enum.auto()


class EventType(enum.Enum):
    DID_EXIT = enum.auto()
    DID_SYNC_MODEL = enum.auto()
    DID_RUN_STEPS = enum.auto()
    EPISODE_DONE = enum.auto()


def worker(worker_i: int, predictor: Predictor, params: AgentParams,
           req_queue: mp.Queue, ev_queue: mp.Queue, bufs: SharedBuffers):
    logger.info(f'Worker#{worker_i} is started.')
    core = AgentCore(predictor, params)
    batch = bufs.as_multi_batch().get(worker_i)

    def on_done(episode_n: int, step_n: int, reward: float, game_str: str):
        ev_queue.put((worker_i, EventType.EPISODE_DONE, episode_n, step_n, reward, game_str))

    while True:
        req = req_queue.get()
        req_type = req[0]
        logger.debug(f'Worker#{worker_i}: {req_type} received.')
        if req_type == RequestType.EXIT:
            ev_queue.put((worker_i, EventType.DID_EXIT))
            break
        elif req_type == RequestType.SYNC_MODEL:
            core.sync_model()
            ev_queue.put((worker_i, EventType.DID_SYNC_MODEL))
        elif req_type == RequestType.RUN_STEPS:
            core.run_steps(batch, on_done)
            ev_queue.put((worker_i, EventType.DID_RUN_STEPS))
        elif req_type == RequestType.GET_GAME_STR:
            ev_queue.put((worker_i, EventType.DID_GET_GAME_STR, core.game_str()))

    logger.info(f'Worker#{worker_i} exits.')


class MultiprocessAgent(Agent):
    req_queues: List[mp.Queue]
    ev_queue: mp.Queue
    bufs: SharedBuffers
    workers: List[mp.Process]
    core: AgentCore

    def __init__(self, predictor: Predictor, batch_size: int, num_workers: int, params: AgentParams):
        super(MultiprocessAgent, self).__init__(batch_size)
        self.req_queues = [mp.Queue() for _ in range(num_workers)]
        self.ev_queue = mp.Queue()
        self.bufs = SharedBuffers.zeros(num_workers, batch_size)
        self.workers = [
            mp.Process(target=worker, args=(
                i, predictor, params,
                self.req_queues[i], self.ev_queue, self.bufs,
            ))
            for i in range(num_workers)
        ]
        for w in self.workers:
            w.start()

    def request(self, *args):
        for q in self.req_queues:
            q.put(*args)

    def wait_events(self, ev_type: EventType):
        for _ in range(len(self.workers)):
            _, t = self.ev_queue.get()
            assert t == ev_type

    def exit(self):
        self.request((RequestType.EXIT,))
        self.wait_events(EventType.DID_EXIT)
        for w in self.workers:
            w.join()

    def sync_model(self):
        self.request((RequestType.SYNC_MODEL,))
        self.wait_events(EventType.DID_SYNC_MODEL)

    def run_steps(self, on_done: DoneCallback) -> MultiBatch:
        self.request((RequestType.RUN_STEPS,))
        n = 0
        while True:
            worker_i, ev_type, *values = self.ev_queue.get()
            if ev_type == EventType.DID_RUN_STEPS:
                n += 1
                if n == len(self.workers):
                    break
            elif ev_type == EventType.EPISODE_DONE:
                on_done(*values)
        return self.bufs.as_multi_batch()
