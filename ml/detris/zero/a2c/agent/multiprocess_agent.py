from typing import List, NamedTuple
import enum
import multiprocessing as mp
import ctypes
import numpy as np
from . import Agent, AgentParams, AgentCore, DoneCallback, ModelLoader
from ...batch import MultiBatch
from ....environment import Environment


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
            np.asarray(self.observations.get_obj(), dtype=np.float).reshape((self.num_workers, self.batch_size, -1)),
            np.asarray(self.action_probs.get_obj(), dtype=np.float).reshape((self.num_workers, self.batch_size, -1)),
            np.asarray(self.actions, dtype=np.uint32).reshape((self.num_workers, -1)),
            np.asarray(self.rewards, dtype=np.float).reshape((self.num_workers, -1)),
            np.asarray(self.dones, dtype=np.uint8).reshape((self.num_workers, -1)),
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


def worker(worker_i: int, model_loader: ModelLoader, params: AgentParams,
           req_queue: mp.Queue, ev_queue: mp.Queue, bufs: SharedBuffers):
    core = AgentCore(params)
    batch = bufs.as_multi_batch().get(worker_i)

    def on_done(episode_n: int, step_n: int, reward: float):
        ev_queue.put((worker_i, EventType.EPISODE_DONE, episode_n, step_n, reward))

    while True:
        req = req_queue.get()
        req_type = req[0]
        if req_type == RequestType.EXIT:
            ev_queue.put((worker_i, EventType.DID_EXIT))
            break
        elif req_type == RequestType.SYNC_MODEL:
            core.set_model(model_loader.load())
            ev_queue.put((worker_i, EventType.DID_SYNC_MODEL))
        elif req_type == RequestType.RUN_STEPS:
            core.run_steps(batch, on_done)
            ev_queue.put((worker_i, EventType.DID_RUN_STEPS))


class MultiprocessAgent(Agent):
    req_queues: List[mp.Queue]
    ev_queue: mp.Queue
    bufs: SharedBuffers
    workers: List[mp.Process]
    core: AgentCore

    def __init__(self, model_loader: ModelLoader, batch_size: int, num_workers: int, params: AgentParams):
        super(MultiprocessAgent, self).__init__(model_loader, batch_size)
        self.req_queues = [mp.Queue() for _ in range(num_workers)]
        self.ev_queue = mp.Queue()
        self.bufs = SharedBuffers.zeros(num_workers, batch_size)
        self.workers = [
            mp.Process(target=worker, args=(
                i, model_loader, params,
                self.req_queues[i], self.ev_queue, self.bufs,
            ))
            for i in range(num_workers)
        ]

    def request(self, *args):
        for q in self.req_queues:
            q.put(*args)

    def wait_events(self, ev_type: EventType):
        n = 0
        for ev in self.ev_queue.get():
            assert ev[1] == ev_type
            n += 1
            if n == len(self.workers):
                break

    def exit(self):
        self.request((RequestType.EXIT,))
        self.wait_events(EventType.DID_EXIT)

    def sync_model(self):
        self.request((RequestType.SYNC_MODEL,))
        self.wait_events(EventType.DID_SYNC_MODEL)

    def game_strs(self) -> List[str]:
        return ['TODO']

    def run_steps(self, on_done: DoneCallback) -> MultiBatch:
        self.request((RequestType.RUN_STEPS,))
        n = 0
        for ev in self.ev_queue.get():
            ev_type = ev[1]
            if ev_type == EventType.DID_RUN_STEPS:
                n += 1
                if n == len(self.workers):
                    break
            elif ev_type == EventType.EPISODE_DONE:
                on_done(ev[2], ev[3], ev[4])
        return self.bufs.as_multi_batch()
