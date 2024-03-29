from typing import List, Optional, Set


class Cell:
    EMPTY: Cell
    ANY: Cell
    S: Cell
    Z: Cell
    L: Cell
    J: Cell
    I: Cell
    T: Cell
    O: Cell
    GARBAGE: Cell

    @classmethod
    def from_id(cls, id: int) -> Cell: ...

    @classmethod
    def from_str(cls, s: str) -> Cell: ...

    @property
    def id(self) -> int: ...

    def __str__(self) -> str: ...


class Placement:
    def __init__(self, orientation: int, x: int, y: int): ...

    @property
    def orientation(self) -> int: ...

    @property
    def x(self) -> int: ...

    @property
    def y(self) -> int: ...


class MoveDecisionResource:
    def get_dst_candidates(self) -> Set[Placement]: ...


class Game:
    def __init__(self): ...

    def fast_mode(self): ...

    def should_supply_next_pieces(self) -> bool: ...

    def supply_next_pieces(self, piece_cell_ids: List[int]): ...

    def setup_falling_piece(self, piece_cell_id: Optional[int] = None): ...

    def drop(self, n: int): ...

    def firm_drop(self): ...

    def shift(self, n: int, to_end: bool): ...

    def rotate(self, n: int): ...

    def lock(self) -> bool: ...

    def hold(self) -> bool: ...

    def can_hold(self) -> bool: ...

    def get_move_decision_resource(self) -> MoveDecisionResource: ...

    def set_falling_piece_placement(self, placement: Placement): ...

    def set_hold_piece(self, piece_cell_id: Optional[int]): ...

    def set_next_pieces(self, piece_cell_ids: List[int]): ...

    def set_playfield_with_u64_rows(self, six_rows_x7: List[int]): ...

    def get_playfield_as_u64_rows(self) -> List[int]: ...

    def __str__(self) -> str: ...

    def __copy__(self) -> Game: ...
