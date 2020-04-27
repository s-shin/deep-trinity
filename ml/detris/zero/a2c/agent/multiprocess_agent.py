from typing import List, NamedTuple, Tuple, Optional
import enum
import multiprocessing as mp
import ctypes
import logging
import numpy as np
import tensorflow as tf
from . import Agent, AgentParams, AgentCore, DoneCallback
from ...util.ipc import MasterWorkerIPC, MasterClient, WorkerClient
from ...batch import MultiBatch
from ...predictor import Predictor, LoadModelFunc, SinglePredictor
from ....environment import Environment

logger = logging.getLogger(__name__)


class EventType(enum.Enum):
    EXIT = enum.auto()
    SYNC_MODEL = enum.auto()
    RUN_STEPS = enum.auto()
    DID_EXIT = enum.auto()
    DID_SYNC_MODEL = enum.auto()
    DID_RUN_STEPS = enum.auto()
    PREDICT = enum.auto()
    DID_PREDICT = enum.auto()
    EPISODE_DONE = enum.auto()


class SharedPredictionBuffers(NamedTuple):
    num_workers: int
    observations: mp.Array
    action_probs: mp.Array
    state_values: mp.Array

    @classmethod
    def zeros(cls, num_workers: int) -> 'PredictionBuffers':
        return cls(
            num_workers,
            mp.Array(ctypes.c_float, num_workers * Environment.OBSERVATION_SIZE),
            mp.Array(ctypes.c_float, num_workers * Environment.NUM_ACTIONS),
            mp.Array(ctypes.c_float, num_workers),
        )

    def as_numpy(self) -> Tuple[np.ndarray, np.ndarray, np.ndarray]:
        return (
            np.frombuffer(self.observations.get_obj(), dtype=np.float32).reshape((self.num_workers, -1)),
            np.frombuffer(self.action_probs.get_obj(), dtype=np.float32).reshape((self.num_workers, -1)),
            np.frombuffer(self.state_values.get_obj(), dtype=np.float32).reshape((self.num_workers,)),
        )


class WorkerPredictor(Predictor):
    worker_i: int
    ipc_client: WorkerClient
    bufs: SharedPredictionBuffers

    def __init__(self, worker_i: int, ipc_client: WorkerClient, bufs: SharedPredictionBuffers):
        self.worker_i = worker_i
        self.ipc_client = ipc_client
        self.bufs = bufs

    def predict(self, observation: np.ndarray) -> (np.ndarray, float):
        observations, action_probs, state_values = self.bufs.as_numpy()
        observations[self.worker_i] = observation
        self.ipc_client.send((self.worker_i, EventType.PREDICT))
        event = self.ipc_client.receive()
        assert event[0] == EventType.DID_PREDICT
        return action_probs[self.worker_i], state_values[self.worker_i]

    def reload_model(self):
        pass


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


def workerMain(worker_i: int, ipc_client: WorkerClient, predictor: Predictor,
               params: AgentParams, bufs: SharedBuffers):
    logger.info(f'Worker#{worker_i} is started.')
    core = AgentCore(predictor, params)
    batch = bufs.as_multi_batch().get(worker_i)

    def on_done(episode_n: int, step_n: int, reward: float, game_str: str):
        ipc_client.send((worker_i, EventType.EPISODE_DONE, episode_n, step_n, reward, game_str))

    while True:
        event = ipc_client.receive()
        event_type = event[0]
        logger.debug(f'Worker#{worker_i}: {event_type} received.')
        if event_type == EventType.EXIT:
            ipc_client.send((worker_i, EventType.DID_EXIT))
            break
        elif event_type == EventType.SYNC_MODEL:
            core.sync_model()
            ipc_client.send((worker_i, EventType.DID_SYNC_MODEL))
        elif event_type == EventType.RUN_STEPS:
            core.run_steps(batch, on_done)
            ipc_client.send((worker_i, EventType.DID_RUN_STEPS))
        elif event_type == EventType.GET_GAME_STR:
            ipc_client.send((worker_i, EventType.DID_GET_GAME_STR, core.game_str()))

    logger.info(f'Worker#{worker_i} exits.')


class MultiprocessAgent1(Agent):
    ipc_client: MasterClient
    bufs: SharedBuffers
    workers: List[mp.Process]
    core: AgentCore

    def __init__(self, load_model_from_file_system: LoadModelFunc,
                 batch_size: int, num_workers: int, params: AgentParams):
        super(MultiprocessAgent1, self).__init__(batch_size)
        ipc = MasterWorkerIPC(num_workers)
        self.ipc_client = ipc.get_master_client()
        self.bufs = SharedBuffers.zeros(num_workers, batch_size)
        self.workers = []
        for i in range(num_workers):
            client = ipc.get_worker_client(i)
            predictor = SinglePredictor(load_model_from_file_system)
            worker = mp.Process(target=workerMain, args=(i, client, predictor, params, self.bufs))
            self.workers.append(worker)
            worker.start()

    def wait_events(self, ev_type: EventType, n: int = -1):
        if n < 0:
            n = len(self.workers)
        for _ in range(n):
            _, t = self.ipc_client.receive()
            assert t == ev_type

    def exit(self):
        self.ipc_client.send((EventType.EXIT,))
        self.wait_events(EventType.DID_EXIT)
        for w in self.workers:
            w.join()

    def sync_model(self):
        self.ipc_client.send((EventType.SYNC_MODEL,))
        self.wait_events(EventType.DID_SYNC_MODEL)

    def run_steps(self, on_done: DoneCallback) -> MultiBatch:
        self.ipc_client.send((EventType.RUN_STEPS,))
        n = 0
        while True:
            worker_i, event_type, *values = self.ipc_client.receive()
            if event_type == EventType.DID_RUN_STEPS:
                n += 1
                if n == len(self.workers):
                    break
            elif event_type == EventType.EPISODE_DONE:
                on_done(*values)
            else:
                raise RuntimeError(f'Unexpected event_type: {event_type}')
        return self.bufs.as_multi_batch()


class MultiprocessAgent2(Agent):
    load_model: LoadModelFunc
    model: Optional[tf.keras.Model]
    ipc_client: MasterClient
    bufs: SharedBuffers
    pred_bufs: Optional[SharedPredictionBuffers]
    workers: List[mp.Process]
    core: AgentCore

    def __init__(self, load_model: LoadModelFunc, batch_size: int, num_workers: int, params: AgentParams):
        super(MultiprocessAgent2, self).__init__(batch_size)
        self.load_model = load_model
        ipc = MasterWorkerIPC(num_workers)
        self.ipc_client = ipc.get_master_client()
        self.bufs = SharedBuffers.zeros(num_workers, batch_size)
        self.workers = []
        self.pred_bufs = SharedPredictionBuffers.zeros(num_workers)
        for i in range(num_workers):
            client = ipc.get_worker_client(i)
            predictor = WorkerPredictor(i, client, self.pred_bufs)
            worker = mp.Process(target=workerMain, args=(i, client, predictor, params, self.bufs))
            self.workers.append(worker)
            worker.start()

    def wait_events(self, ev_type: EventType, n: int = -1):
        if n < 0:
            n = len(self.workers)
        for _ in range(n):
            _, t = self.ipc_client.receive()
            assert t == ev_type

    def exit(self):
        self.ipc_client.send((EventType.EXIT,))
        self.wait_events(EventType.DID_EXIT)
        for w in self.workers:
            w.join()

    def sync_model(self):
        self.model = self.load_model()

    def run_steps(self, on_done: DoneCallback) -> MultiBatch:
        self.ipc_client.send((EventType.RUN_STEPS,))
        n = 0
        num_predict_events = 0
        while True:
            worker_i, event_type, *values = self.ipc_client.receive()
            # logger.debug(f'Master: {event_type} received.')
            if event_type == EventType.DID_RUN_STEPS:
                assert num_predict_events == 0
                n += 1
                if n == len(self.workers):
                    break
            elif event_type == EventType.EPISODE_DONE:
                on_done(*values)
            elif event_type == EventType.PREDICT:
                num_predict_events += 1
                if num_predict_events < len(self.workers):
                    continue
                num_predict_events = 0
                observations, action_probs, state_values = self.pred_bufs.as_numpy()
                action_probs_batch, state_value_batch = self.model.predict_on_batch(
                    tf.convert_to_tensor(observations))
                action_probs[:] = action_probs_batch.numpy()
                state_values[:] = state_value_batch.numpy().squeeze()
                self.ipc_client.send((EventType.DID_PREDICT,))
        return self.bufs.as_multi_batch()
