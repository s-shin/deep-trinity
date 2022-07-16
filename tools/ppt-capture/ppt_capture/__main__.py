from typing import NamedTuple, Optional, List
from enum import Enum
import signal
import mss
import cv2
import numpy as np
from deep_trinity import Game, Cell


class Vec2:
    array: np.ndarray

    def __init__(self, x, y, dtype='f4'):
        self.array = np.array([x, y], dtype=dtype)

    def __repr__(self):
        return f"Vec2({self.array[0]:.3g}, {self.array[1]:.3g})"

    def __copy__(self):
        a = self.array
        return Vec2(a[0], a[1], a.dtype)

    def x(self):
        return self.array[0]

    def y(self):
        return self.array[1]

    def scale(self, ratio: float):
        v = self.__copy__()
        v.array *= ratio
        return v


class Rect:
    array: np.ndarray

    def __init__(self, x1, y1, x2, y2, dtype='f4'):
        self.array = np.array([x1, y1, x2, y2], dtype=dtype)

    def __repr__(self):
        a = self.array
        return f"Rect({a[0]:.3g}, {a[1]:.3g}, {a[2]:.3g}, {a[3]:.3g})"

    def __copy__(self):
        a = self.array
        return Rect(a[0], a[1], a[2], a[3], a.dtype)

    def x1(self):
        return self.array[0]

    def y1(self):
        return self.array[1]

    def p1(self):
        return Vec2(self.array[0], self.array[1])

    def x2(self):
        return self.array[2]

    def y2(self):
        return self.array[3]

    def p2(self):
        return Vec2(self.array[2], self.array[3])

    def width(self):
        return self.array[2] - self.array[0]

    def height(self):
        return self.array[3] - self.array[1]

    def size(self):
        return Vec2(self.width(), self.height())

    def scale(self, ratio: float):
        r = self.__copy__()
        r.array *= ratio
        return r


class Piece(Enum):
    # NONE = 0
    S = Cell.S.id
    Z = Cell.Z.id
    L = Cell.L.id
    J = Cell.J.id
    I = Cell.I.id
    T = Cell.T.id
    O = Cell.O.id


class Region(Enum):
    # NONE = 0
    HOLD_PIECE = 1
    PLAYFIELD = 2
    NEXT_PIECE_1 = 3
    NEXT_PIECE_2 = 4
    NEXT_PIECE_3 = 5
    NEXT_PIECE_4 = 6
    NEXT_PIECE_5 = 7


NEXT_PIECE_REGIONS: List[Region] = [getattr(Region, f"NEXT_PIECE_{i + 1}") for i in range(5)]


class RegionInfo(NamedTuple):
    hold: Rect = Rect(4, 50, 4 + 103, 50 + 54)
    playfield: Rect = Rect(119, 0, 119 + 307, 0 + 622)
    next1: Rect = Rect(453, 24, 453 + 103, 24 + 54)
    next2: Rect = Rect(456, 119, 456 + 84, 119 + 44)
    next3: Rect = Rect(456, 203, 456 + 84, 203 + 44)
    next4: Rect = Rect(456, 288, 456 + 84, 288 + 44)
    next5: Rect = Rect(456, 372, 456 + 84, 372 + 44)

    def __repr__(self):
        return "RegionInfo(hold={}, playfield={}, nexts=[{}, {}, {}, {}])".format(
            self.hold, self.playfield, self.next1, self.next2, self.next3, self.next4, self.next5)

    def get(self, r: Region) -> Rect:
        return getattr(self, r.name.lower())

    def scale(self, ratio):
        return RegionInfo(
            self.hold.scale(ratio),
            self.playfield.scale(ratio),
            self.next1.scale(ratio),
            self.next2.scale(ratio),
            self.next3.scale(ratio),
            self.next4.scale(ratio),
            self.next5.scale(ratio),
        )


class BlockSizeInfo(NamedTuple):
    hold: Vec2 = Vec2(24, 24)
    playfield: Vec2 = Vec2(30.75, 30.75)
    next1: Vec2 = Vec2(24, 24)
    next2: Vec2 = Vec2(19, 19)
    next3: Vec2 = Vec2(19, 19)
    next4: Vec2 = Vec2(19, 19)
    next5: Vec2 = Vec2(19, 19)

    def __repr__(self):
        return "BlockSizeInfo(hold={}, playfield={}, nexts=[{}, {}, {}, {}])".format(
            self.hold, self.playfield, self.next1, self.next2, self.next3, self.next4, self.next5)

    def get(self, r: Region) -> Vec2:
        return getattr(self, r.name.lower())

    def scale(self, ratio: float):
        return BlockSizeInfo(
            self.hold.scale(ratio),
            self.playfield.scale(ratio),
            self.next1.scale(ratio),
            self.next2.scale(ratio),
            self.next3.scale(ratio),
            self.next4.scale(ratio),
            self.next5.scale(ratio),
        )


class ScreenInfo(NamedTuple):
    size: Vec2 = Vec2(556, 622)
    regions: RegionInfo = RegionInfo()
    block_sizes: BlockSizeInfo = BlockSizeInfo()

    def __repr__(self):
        return f"ScreenInfo(size={self.size}, {self.regions}, {self.block_sizes})"

    def aspect(self):
        return self.size.y() / self.size.x()

    def scale(self, ratio: float):
        return ScreenInfo(
            self.size.scale(ratio),
            self.regions.scale(ratio),
            self.block_sizes.scale(ratio)
        )

    def resize_by_width(self, width):
        return self.scale(width / self.size.x())

    def get_upper_left_piece_block_center_point(self, region: Region, piece: Piece, relative=False):
        if region == Region.PLAYFIELD:
            raise ValueError()
        r = self.regions.get(region)
        s = self.block_sizes.get(region)
        if piece == Piece.I:
            nx, ny, bx, by = 4, 1, 0, 0
        elif piece == Piece.O:
            nx, ny, bx, by = 2, 2, 0, 0
        elif piece == Piece.Z:
            nx, ny, bx, by = 3, 2, 1, 0
        else:
            nx, ny, bx, by = 3, 2, 0, 1
        left_padding = (r.width() - s.x() * nx) * 0.5
        top_padding = (r.height() - s.y() * ny) * 0.5
        px = left_padding + s.x() * (0.5 + bx)
        py = top_padding + s.y() * (0.5 + by)
        return Vec2(px, py) if relative else Vec2(px + r.x1(), py + r.y1())

    def get_upper_left_playfield_block_center_point(self, relative=False):
        s = self.block_sizes.playfield
        p = Vec2(
            s.x() * 0.5,
            self.size.y() - s.y() * 19.5,
        )
        if not relative:
            p.array += self.regions.playfield.p1().array
        return p


REFERENCE_SCREEN_INFO = ScreenInfo()

# BGR
PIECE_COLORS = {
    Piece.L: np.array([49, 114, 228]),
    Piece.J: np.array([168, 86, 38]),
    Piece.S: np.array([75, 201, 131]),
    Piece.Z: np.array([47, 39, 178]),
    Piece.I: np.array([207, 168, 76]),
    Piece.T: np.array([135, 40, 126]),
    Piece.O: np.array([72, 204, 244]),
}


def detect_pieces_by_color(bgr_arr: np.ndarray, threshold=100):
    if bgr_arr.ndim != 2 and bgr_arr.shape[1] != 3:
        raise ValueError()
    r = np.zeros(bgr_arr.shape[0], dtype='u1')
    for piece, color in PIECE_COLORS.items():
        d = np.sum(np.abs(bgr_arr - color), axis=1) <= threshold
        r[d] = piece.value
    return r


class GameState(NamedTuple):
    hold_piece: Optional[Piece]
    next_pieces: List[Piece]
    playfield: np.ndarray
    """This playfield can be contain piece information."""

    def get_playfield_as_u64_rows(self):
        rows = []
        for i in range(7):
            y = i * 6
            row = sum(bool(b) << i for i, b in enumerate(self.playfield[y:y + 6].flatten()))
            rows.append(row)
        return rows

    def to_game(self):
        game = Game()
        game.fast_mode()
        game.set_hold_piece(self.hold_piece)
        game.set_next_pieces([p.value for p in self.next_pieces])
        game.set_playfield_with_u64_rows(self.get_playfield_as_u64_rows())
        return game


class GameStateRecognizer:
    prev: Optional[GameState] = None

    def update(self, hold_piece: Optional[Piece], next_pieces: List[Optional[Piece]], playfield: np.ndarray):
        game = Game()
        game.fast_mode()
        # TODO


def main():
    should_stop = False

    def stop():
        nonlocal should_stop
        should_stop = True

    signal.signal(signal.SIGINT, stop)
    signal.signal(signal.SIGTERM, stop)

    monitor = {
        "left": 96,
        "top": 245,
        "width": 277,
        "height": 277 * REFERENCE_SCREEN_INFO.aspect(),
    }

    debug1 = False
    debug2 = False
    scrren_info: Optional[ScreenInfo] = None
    hold_next_piece_pixels_mask: Optional[np.ndarray] = None
    masked_hold_next_piece_pixel_regions: Optional[np.ndarray] = None
    playfield_piece_pixels_mask: Optional[np.ndarray] = None
    recognizer = GameStateRecognizer()

    with mss.mss() as sct:
        while not should_stop:
            if cv2.waitKey(500) == ord("q"):
                cv2.destroyAllWindows()
                break
            print("---")

            # Capture screen.
            img = np.array(sct.grab(monitor))

            # Initialize screen_info and caches.
            if scrren_info is None:
                screen_info = REFERENCE_SCREEN_INFO.resize_by_width(img.shape[1])

                hold_next_piece_pixels_mask = np.zeros((img.shape[0], img.shape[1]), "?")
                regions = np.zeros(hold_next_piece_pixels_mask.shape, "u1")
                for region in Region:
                    if region == Region.PLAYFIELD:
                        continue
                    for piece in Piece:
                        p = screen_info.get_upper_left_piece_block_center_point(region, piece)
                        x = int(p.x())
                        y = int(p.y())
                        hold_next_piece_pixels_mask[y, x] = True
                        regions[y, x] = region.value
                masked_hold_next_piece_pixel_regions = regions[hold_next_piece_pixels_mask]
                del regions

                playfield_piece_pixels_mask = np.zeros((img.shape[0], img.shape[1]), "?")
                pf_block_pos = screen_info.get_upper_left_playfield_block_center_point()
                for y in range(20):
                    for x in range(10):
                        p = pf_block_pos.array + screen_info.block_sizes.playfield.array * [x, y]
                        playfield_piece_pixels_mask[int(p[1]), int(p[0])] = True

            if debug1:
                for region in Region:
                    r = screen_info.regions.get(region)
                    cv2.rectangle(img, r.p1().array.astype('u4'), r.p2().array.astype('u4'), (0, 0, 255), 2)
                    if region != Region.PLAYFIELD:
                        for piece in Piece:
                            pos = screen_info.get_upper_left_piece_block_center_point(region, piece)
                            p1 = pos.array - 1
                            p2 = pos.array + 1
                            cv2.rectangle(img, p1.astype('u4'), p2.astype('u4'), (255, 255, 255), 2)
                pf_block_pos = screen_info.get_upper_left_playfield_block_center_point()
                for y in range(20):
                    for x in range(10):
                        p = pf_block_pos.array + screen_info.block_sizes.playfield.array * [x, y]
                        cv2.rectangle(img, (p - 1).astype('u4'), (p + 1).astype('u4'), (255, 255, 255), 2)
                cv2.imshow("Debug1", img)

            # Detect hold and next pieces.
            region_pieces = {}
            piece_pixels = img[:, :, :3][hold_next_piece_pixels_mask]
            result = detect_pieces_by_color(piece_pixels)
            for region in Region:
                if region == Region.PLAYFIELD:
                    continue
                region_result = result[masked_hold_next_piece_pixel_regions == region.value]
                piece_result = region_result[region_result > 0]
                region_pieces[region] = None if len(piece_result) == 0 else Piece(piece_result[0])

            # Detect playfield pieces.
            piece_pixels = img[:, :, :3][playfield_piece_pixels_mask]
            result = detect_pieces_by_color(piece_pixels)
            playfield = np.flipud(result.reshape((20, 10)))

            if debug2:
                lines = [
                    "[{}]  (?) {}{}{}{}{}".format(*[
                        (region_pieces[t].name if region_pieces[t] is not None else "?")
                        for t in Region if t != Region.PLAYFIELD
                    ]),
                    "--+----------+",
                ]
                for i in range(20):
                    y = 19 - i
                    s = f"{y:02}|"
                    for x in range(10):
                        piece_value = playfield[y, x]
                        s += Piece(piece_value).name if piece_value > 0 else " "
                    s += "|"
                    lines.append(s)
                lines.append("--+----------+")
                lines.append("##|0123456789|")
                print("\n".join(lines))

            # Update recognizer.
            hold_piece = region_pieces[Region.HOLD_PIECE]
            next_pieces = [region_pieces[r] for r in NEXT_PIECE_REGIONS]
            recognizer.update(hold_piece, next_pieces, playfield)


main()
