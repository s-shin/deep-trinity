import multiprocessing as mp
from typing import List, Tuple, Any, Optional


class MasterWorkerIPC:
    txs: List[mp.Queue]
    rx: mp.Queue

    def __init__(self, num_workers: int):
        self.txs = [mp.Queue() for _ in range(num_workers)]
        self.rx = mp.Queue()

    def send_to_worker(self, args: Any, worker_i=None):
        if worker_i is None:
            for tx in self.txs:
                tx.put(args)
        else:
            self.txs[worker_i].put(args)

    def send_to_master(self, args: Tuple):
        self.rx.put(args)

    def receive_from_worker(self) -> Any:
        return self.rx.get()

    def receive_from_master(self, worker_i) -> Any:
        return self.txs[worker_i].get()

    def get_master_client(self):
        return MasterClient(self)

    def get_worker_client(self, worker_i: int):
        return WorkerClient(self, worker_i)


class MasterClient:
    ipc: MasterWorkerIPC

    def __init__(self, ipc: MasterWorkerIPC):
        self.ipc = ipc

    def send(self, args: Any, worker_i: Optional[int] = None):
        self.ipc.send_to_worker(args, worker_i)

    def receive(self) -> Any:
        return self.ipc.receive_from_worker()


class WorkerClient:
    ipc: MasterWorkerIPC
    worker_i: int

    def __init__(self, ipc: MasterWorkerIPC, worker_i: int):
        self.ipc = ipc
        self.worker_i = worker_i

    def send(self, args: Any):
        self.ipc.send_to_master(args)

    def receive(self) -> Any:
        return self.ipc.receive_from_master(self.worker_i)
