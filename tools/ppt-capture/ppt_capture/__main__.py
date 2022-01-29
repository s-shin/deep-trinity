from typing import NamedTuple
from enum import Enum
import time
import signal
import mss
import cv2
import numpy as np


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

    def p1(self):
        return Vec2(self.array[0], self.array[1])

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
    L = "L"
    J = "J"
    S = "S"
    Z = "Z"
    I = "I"
    T = "T"
    O = "O"


class ScreenTarget(Enum):
    HOLD = "hold"
    PLAYFIELD = "playfield"
    NEXT1 = "next1"
    NEXT2 = "next2"
    NEXT3 = "next3"
    NEXT4 = "next4"
    NEXT5 = "next5"


class RectInfo(NamedTuple):
    hold: Rect = Rect(4, 50, 4 + 103, 50 + 54)
    playfield: Rect = Rect(119, 0, 119 + 307, 0 + 622)
    next1: Rect = Rect(453, 24, 453 + 103, 24 + 54)
    next2: Rect = Rect(456, 119, 456 + 84, 119 + 44)
    next3: Rect = Rect(456, 203, 456 + 84, 203 + 44)
    next4: Rect = Rect(456, 288, 456 + 84, 288 + 44)
    next5: Rect = Rect(456, 372, 456 + 84, 372 + 44)

    def __repr__(self):
        return "RectInfo(hold={}, playfield={}, nexts=[{}, {}, {}, {}])".format(
            self.hold, self.playfield, self.next1, self.next2, self.next3, self.next4, self.next5)

    def get(self, t: ScreenTarget) -> Rect:
        return getattr(self, t.value)

    def scale(self, ratio):
        return RectInfo(
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
    playfield: Vec2 = Vec2(30.7, 30.7)
    next1: Vec2 = Vec2(24, 24)
    next2: Vec2 = Vec2(19, 19)
    next3: Vec2 = Vec2(19, 19)
    next4: Vec2 = Vec2(19, 19)
    next5: Vec2 = Vec2(19, 19)

    def __repr__(self):
        return "BlockSizeInfo(hold={}, playfield={}, nexts=[{}, {}, {}, {}])".format(
            self.hold, self.playfield, self.next1, self.next2, self.next3, self.next4, self.next5)

    def get(self, t: ScreenTarget) -> Vec2:
        return getattr(self, t.value)

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
    rects: RectInfo = RectInfo()
    block_sizes: BlockSizeInfo = BlockSizeInfo()

    def __repr__(self):
        return f"ScreenInfo(size={self.size}, {self.rects}, {self.block_sizes})"

    def aspect(self):
        return self.size.y() / self.size.x()

    def scale(self, ratio: float):
        return ScreenInfo(
            self.size.scale(ratio),
            self.rects.scale(ratio),
            self.block_sizes.scale(ratio)
        )

    def resize_by_width(self, width):
        return self.scale(width / self.size.x())

    def get_a_block_center_position_in_piece_rect(self, target: ScreenTarget, piece: Piece):
        if target == ScreenTarget.PLAYFIELD:
            raise ValueError()
        r = self.rects.get(target)
        s = self.block_sizes.get(target)
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
        return Vec2(px, py)

    def get_top_left_block_center_position_in_playfield(self):
        s = self.block_sizes.playfield
        return Vec2(
            s.x() * 0.5,
            self.size.y() - s.y() * 19.5,
        )


REFERENCE_SCREEN_INFO = ScreenInfo()

# BGR
PIECE_COLORS = {
    "L": np.array([49, 114, 228]),
    "J": np.array([168, 86, 38]),
    "S": np.array([75, 201, 131]),
    "Z": np.array([47, 39, 178]),
    "I": np.array([207, 168, 76]),
    "T": np.array([135, 40, 126]),
    "O": np.array([72, 204, 244]),
}


def get_monitor(x, y, width):
    return {
        "left": x,
        "top": y,
        "width": width,
        "height": width * REFERENCE_SCREEN_INFO.aspect()
    }


def get_rects(size):
    sizes = np.array([size[0], size[1], size[0], size[1]], dtype='f4')
    r = {}
    for k, v in NORMALIZED_RECT.items():
        r[k] = (v * sizes).astype('i4')
    return r


def to_binary(img):
    # gray_img = cv2.cvtColor(img, cv2.COLOR_BGR2GRAY)
    # gray_img = np.average(img[:, :, :3], axis=-1).astype('u1')
    gray_img = (0.35 * img[:, :, 0] + 0.15 * img[:, :, 1] + 0.5 * img[:, :, 2]).astype('u1')
    _, bin_img = cv2.threshold(gray_img, 63, 255, cv2.THRESH_BINARY_INV)
    return bin_img


def detect_piece_v1(bgr_img: np.ndarray, similarity=0.1, count=50):
    counts = {p: 0 for p in PIECE_COLORS.keys()}
    for y in range(bgr_img.shape[0]):
        for x in range(bgr_img.shape[1]):
            for p, c in PIECE_COLORS.items():
                if np.sum(np.abs(bgr_img[y, x, :3] - c)) / float(255 * 3) <= similarity:
                    counts[p] += 1
                    if counts[p] >= count:
                        return p
    return None


def detect_piece_v2(bin_img: np.ndarray):
    pass


def main():
    should_stop = False

    def stop():
        nonlocal should_stop
        should_stop = True

    signal.signal(signal.SIGINT, should_stop)
    signal.signal(signal.SIGTERM, should_stop)

    scrren_info = None

    monitor = get_monitor(96, 245, 277)
    with mss.mss() as sct:
        while not should_stop:
            # Capture screen.
            img = np.array(sct.grab(monitor))
            # cv2.imshow("Test", img)
            # if cv2.waitKey(500) == ord("q"):
            #     cv2.destroyAllWindows()
            #     break
            # continue

            # Initialize screen_info.
            if scrren_info is None:
                screen_info = REFERENCE_SCREEN_INFO.resize_by_width(img.shape[1])
            for t in ScreenTarget:
                r = screen_info.rects.get(t)
                cv2.rectangle(img, r.p1().array.astype('u4'), r.p2().array.astype('u4'), (0, 0, 255), 2)
                if t != ScreenTarget.PLAYFIELD:
                    for piece in Piece:
                        pos = screen_info.get_a_block_center_position_in_piece_rect(t, piece)
                        p1 = r.p1().array + pos.array - 1
                        p2 = r.p1().array + pos.array + 1
                        cv2.rectangle(img, p1.astype('u4'), p2.astype('u4'), (255, 255, 255), 2)
            pf_block_pos = screen_info.get_top_left_block_center_position_in_playfield()
            for y in range(20):
                for x in range(10):
                    p = screen_info.rects.playfield.p1().array + pf_block_pos.array + \
                        screen_info.block_sizes.playfield.array * [x, y]
                    cv2.rectangle(img, (p - 1).astype('u4'), (p + 1).astype('u4'), (255, 255, 255), 2)
            cv2.imshow("Test", img)
            if cv2.waitKey(500) == ord("q"):
                cv2.destroyAllWindows()
                break
            continue

            # img = to_binary(img)
            # cv2.imshow("Test", img)
            # if cv2.waitKey(500) == ord("q"):
            #     cv2.destroyAllWindows()
            #     break
            # continue

            r = rects["hold"]
            img = img[r[1]:r[3], r[0]:r[2]]
            padding = int((img.shape[1] - HOLD_BLOCK_SIZE * 4) * 0.5)
            img = img[:, padding:(img.shape[1] - padding)]
            cv2.imshow("Hold", img)
            if cv2.waitKey(500) == ord("q"):
                cv2.destroyAllWindows()
                break
            continue

            pieces = {}
            for t, r in rects.items():
                if t == "playfield":
                    continue
                p_img = img[r[1]:r[3], r[0]:r[2]]
                print("{}".format(p_img.shape[0] * p_img.shape[1]))
                piece = detect_piece(p_img)
                pieces[t] = piece if piece is not None else "?"

            print("[{}] {}{}{}{}{}".format(pieces["hold"], pieces["next1"], pieces["next2"], pieces["next3"],
                                           pieces["next4"], pieces["next5"]))

            time.sleep(0.25)
            # if cv2.waitKey(500) == ord("q"):
            #     cv2.destroyAllWindows()
            #     break


main()
