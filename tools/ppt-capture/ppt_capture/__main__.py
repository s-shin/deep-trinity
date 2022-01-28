import time
import signal
import mss
import cv2
import numpy as np

REFERENCE_CAPTURE_SIZE = (556, 622)

REFERENCE_RECTS = {
    "hold": (4, 50, 4 + 103, 50 + 54),
    "playfield": (119, 0, 119 + 307, 0 + 622),
    "next1": (453, 24, 453 + 103, 24 + 54),
    "next2": (456, 119, 456 + 84, 119 + 44),
    "next3": (456, 203, 456 + 84, 203 + 44),
    "next4": (456, 288, 456 + 84, 288 + 44),
    "next5": (456, 372, 456 + 84, 372 + 44),
}


def get_normalized_rects():
    r = {}
    s = REFERENCE_CAPTURE_SIZE
    sizes = np.array([s[0], s[1], s[0], s[1]], dtype='f4')
    for k, v in REFERENCE_RECTS.items():
        r[k] = np.array(v, dtype='f4') / sizes
    return r


CAPTURE_ASPECT = float(REFERENCE_CAPTURE_SIZE[1]) / float(REFERENCE_CAPTURE_SIZE[0])
NORMALIZED_RECT = get_normalized_rects()

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
        "height": width * CAPTURE_ASPECT
    }


def get_rects(size):
    sizes = np.array([size[0], size[1], size[0], size[1]], dtype='f4')
    r = {}
    for k, v in NORMALIZED_RECT.items():
        r[k] = (v * sizes).astype('i4')
    return r


def detect_piece(img: np.ndarray, similarity=0.1, count=50):
    counts = {p: 0 for p in PIECE_COLORS.keys()}
    for y in range(img.shape[0]):
        for x in range(img.shape[1]):
            for p, c in PIECE_COLORS.items():
                if np.sum(np.abs(img[y, x, :3] - c)) / float(255 * 3) <= similarity:
                    counts[p] += 1
                    if counts[p] >= count:
                        return p
    return None


def main():
    should_stop = False

    def stop():
        nonlocal should_stop
        should_stop = True

    signal.signal(signal.SIGINT, should_stop)
    signal.signal(signal.SIGTERM, should_stop)

    monitor = get_monitor(96, 245, 277)
    with mss.mss() as sct:
        while not should_stop:
            img = np.array(sct.grab(monitor))

            rects = get_rects((img.shape[1], img.shape[0]))
            # for rect in rects.values():
            #     cv2.rectangle(img, (rect[0], rect[1]), (rect[2], rect[3]), (0, 0, 255), 2)
            # cv2.imshow("Test", img)

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
